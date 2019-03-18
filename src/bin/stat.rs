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
NAME
    stat - display file status

SYNOPSIS
    stat [-x] [file ...]

DESCRIPTION
     The stat utility displays information about the file pointed to by	file.
     Read, write, or execute permissions of the	named file are not required,
     but all directories listed	in the pathname	leading	to the file must be
     searchable.  If no	argument is given, stat	displays information about the
     file descriptor for standard input.

OPTIONS
     -x	        Display information in a more verbose way as known from some
	            Linux distributions.
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

fn main() {
	
    //create Output options
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    
    //create help page
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"])
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

    for path in &parser.args[0..] {
        let meta = fs::symlink_metadata(path).unwrap();
        let file_type = if meta.file_type().is_symlink() {
            "symbolic link"
        } else if meta.is_file() {
            "regular file"
        } else if meta.is_dir() {
            "directory"
        } else {
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
		}
		//Standard Output
        else
        {
			print!("{} {} {} {} {} {} ", meta.dev(), meta.ino(), Perms(meta.mode()), meta.nlink(), meta.uid(), meta.gid());
			print!("{}", meta.rdev());
			print!("{} ", meta.size());
			print!("\"{}\" ", time::at(Timespec::new(meta.atime(), meta.atime_sec() as i32)).strftime(TIME_FMT).unwrap());
			print!("\"{}\" ", time::at(Timespec::new(meta.mtime(), meta.mtime_sec() as i32)).strftime(TIME_FMT).unwrap()); 
			print!("\"{}\" ", time::at(Timespec::new(meta.ctime(), meta.ctime_sec() as i32)).strftime(TIME_FMT).unwrap());
			//TODO: %SB (Birth time of Inode) option from BSD stat(1)
			println!("{}", path);
        }
		return;
    }
}
