#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate time;
extern crate redox_users;

use std::{env, fmt, fs};
use std::io::{stdout, stderr, Write};
use std::os::unix::fs::MetadataExt;
use std::process::exit;

use arg_parser::ArgParser;
use extra::option::OptionalExt;
use redox_users::{AllUsers, AllGroups, Config, All};

use time::Timespec;

const MAN_PAGE: &'static str = /* @MANSTART{stat} */ r#"
Usage: stat [OPTION]... FILE...
Display file or file system status.

Mandatory arguments to long options are mandatory for short options too.
  -L, --dereference     follow links
  -f, --file-system     display file system status instead of file status
  -c  --format=FORMAT   use the specified FORMAT instead of the default;
                          output a newline after each use of FORMAT 
      --printf=FORMAT   like --format, but interpret backslash escapes,
                          and do not output a mandatory trailing newline;
                          if you want a newline, include \n in FORMAT
  -t, --terse           print the information in terse form
      --help     display this help and exit
      --version  output version information and exit

The valid format sequences for files (without --file-system):

  %a   access rights in octal (note '#' and '0' printf flags)
  %A   access rights in human readable form
  %b   number of blocks allocated (see %B)
  %B   the size in bytes of each block reported by %b
  %C   SELinux security context string
  %d   device number in decimal
  %D   device number in hex
  %f   raw mode in hex
  %F   file type
  %g   group ID of owner
  %G   group name of owner
  %h   number of hard links
  %i   inode number
  %m   mount point
  %n   file name
  %N   quoted file name with dereference if symbolic link
  %o   optimal I/O transfer size hint
  %s   total size, in bytes
  %t   major device type in hex, for character/block device special files
  %T   minor device type in hex, for character/block device special files
  %u   user ID of owner
  %U   user name of owner
  %w   time of file birth, human-readable; - if unknown
  %W   time of file birth, seconds since Epoch; 0 if unknown
  %x   time of last access, human-readable
  %X   time of last access, seconds since Epoch
  %y   time of last data modification, human-readable
  %Y   time of last data modification, seconds since Epoch
  %z   time of last status change, human-readable
  %Z   time of last status change, seconds since Epoch

Valid format sequences for file systems:

  %a   free blocks available to non-superuser
  %b   total data blocks in file system
  %c   total file nodes in file system
  %d   free file nodes in file system
  %f   free blocks in file system
  %i   file system ID in hex
  %l   maximum length of filenames
  %n   file name
  %s   block size (for faster transfers)
  %S   fundamental block size (for block counts)
  %t   file system type in hex
  %T   file system type in human readable form

NOTE: your shell may have its own version of stat, which usually supersedes
the version described here.  Please refer to your shell's documentation
for details about the options it supports.

NOTE: The current Redox shell supports only the -x option.

GNU coreutils online help: <http://www.gnu.org/software/coreutils/>
Report stat translation bugs to <http://translationproject.org/team/>
Full documentation at: <http://www.gnu.org/software/coreutils/stat>
or available locally via: info '(coreutils) stat invocation'
"#; /* @MANEND */

const TIME_FMT: &'static str = "%Y-%m-%d %H:%M:%S.%f %z";

struct Perms(u32);

impl fmt::Display for Perms {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(0{:o}/", self.0 & 0o777)?;
        let perm = |i, c| {
            if self.0 & ((1 << i) as u32) != 0 {
                c
            } else {
                "-"
            }
        };
        write!(f, "{}{}{}", perm(8, "r"), perm(7, "w"), perm(6, "x"))?;
        write!(f, "{}{}{}", perm(5, "r"), perm(4, "w"), perm(3, "x"))?;
        write!(f, "{}{}{}", perm(2, "r"), perm(1, "w"), perm(0, "x"))?;
        write!(f, ")")?;
        Ok(())
    }
}

fn main() 
{
    //create Output options
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    
    //create help page
    let mut parser = ArgParser::new(1)
        .add_flag(&["help"])
        .add_flag(&["x"]);
    parser.parse(env::args());

    //show manpage or return error
    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }
    
    //Try to get User and Group rights, throw an error upon failure
    let (all_users, all_groups) = match (AllUsers::new(Config::default()), AllGroups::new(Config::default())) {
        (Ok(all_users), Ok(all_groups)) => (all_users, all_groups),
        (Err(_), Ok(_)) => {
            eprintln!("Unable to access password file");
            exit(1);
        },
        (Ok(_), Err(_)) => {
            eprintln!("Unable to access group file");
            exit(1);
        }
        _ => {
            eprintln!("Unable to access password and group file");
            exit(1);
        }
    };
    
    //Exit if no operand is entered
    match &parser.args.len() {
        0 => 
        {
            println!("stat: missing operand");
            println!("Try 'stat --help' for more information.");
            exit(1);
        },
        _ => {}
    };

    //Show stat for all operands
    for path in &parser.args[0..] 
    {
        //Exit if operand does not exist
        let meta = fs::metadata(path);
        let meta = match meta 
        {
            Ok(metadata) => metadata,
            Err(_) => 
            {
                eprintln!("stat: cannot stat '{}': No such file or directory", path);
                exit(-1);
            }
        };
        
        //detect type of operand
        let file_type = if meta.file_type().is_symlink()
        {
            "symbolic link" 
        }
        else if meta.is_file() 
        {
            "regular file"
        }  
        else if meta.is_dir()
        {
            "directory"
        }
        else
        {
            ""
        };
        
        //Get username and groupname of file
        let username = all_users.get_by_id(meta.uid() as usize)
            .map(|x| x.user.to_string())
            .unwrap_or("UNKNOWN".to_string());
        let groupname = all_groups.get_by_id(meta.gid() as usize)
            .map(|x| x.group.to_string())
            .unwrap_or("UNKNOWN".to_string());

		//Verbose Output
		if parser.found(&'x') 
		{
			if meta.file_type().is_symlink()
			{
				println!("File: {} -> {}", path, fs::read_link(path).unwrap().display());
			}
			else
			{
				println!("File: {}", path);
            }
			println!("Size: {}  Blocks: {}  IO Block: {} {}", meta.size(), meta.blocks(), meta.blksize(), file_type);
			println!("Device: {}  Inode: {}  Links: {}", meta.dev(), meta.ino(), meta.nlink());
			println!("Access: {}", Perms(meta.mode()));
			println!("Uid: ({}/{}) Gid: ({}/{})", meta.uid(), username, meta.gid(), groupname);
			println!("Access: {}", time::at(Timespec::new(meta.atime(), meta.atime_nsec() as i32)).strftime(TIME_FMT).unwrap());
			println!("Modify: {}", time::at(Timespec::new(meta.mtime(), meta.mtime_nsec() as i32)).strftime(TIME_FMT).unwrap());
			println!("Change: {}", time::at(Timespec::new(meta.ctime(), meta.ctime_nsec() as i32)).strftime(TIME_FMT).unwrap());
            //TODO: %SB (Birth time of Inode) option from BSD stat(1)	
		}
		//Standard Output
        else
        {
			print!("{} {} {} {} {} {} ", meta.dev(), meta.ino(), Perms(meta.mode()), meta.nlink(), meta.uid(), meta.gid());
			//TODO: print!("{}", meta.rdev()); Fails to compile, not found?
			print!("{} ", meta.size());
            //TODO: Cannot be changed from nsec to sec, not found?
			print!("\"{}\" ", time::at(Timespec::new(meta.atime(), meta.atime_nsec() as i32)).strftime(TIME_FMT).unwrap());
			print!("\"{}\" ", time::at(Timespec::new(meta.mtime(), meta.mtime_nsec() as i32)).strftime(TIME_FMT).unwrap()); 
			print!("\"{}\" ", time::at(Timespec::new(meta.ctime(), meta.ctime_nsec() as i32)).strftime(TIME_FMT).unwrap());
			//TODO: %SB (Birth time of Inode) option from BSD stat(1)
			println!("{}", path);
        }
    }
    return;
}
