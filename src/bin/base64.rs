#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate base64;
#[macro_use]
extern crate coreutils;

use std::io::{self, Write, Read, Stderr};
use std::path::Path;
use std::fs::File;
use std::process;
use arg_parser::ArgParser;
use coreutils::arg_parser::ArgParserExt;
use extra::option::OptionalExt;
use extra::io::fail;

const MAN_PAGE: &'static str = /* @MANSTART{base64} */ r#"
NAME
    base64 - encode / decode binary file as RFC 1341 MIME base64

SYNOPSIS
    base64 -d|-e [ -h ] [ infile [ outfile ] ]

DESCRIPTION
    The MIME (Multipurpose Internet Mail Extensions) specification (RFC
    1341 and successors) defines a mechanism for encoding arbitrary binary
    information for transmission by electronic mail.  Triplets of 8-bit
    octets are encoded as groups of four characters, each representing 6
    bits of the source 24 bits.  Only characters present in all variants of
    ASCII and EBCDIC are used, avoiding incompatibilities in other forms of
    encoding such as uuencode/uudecode.

    base64 is a command line utility which encodes and decodes files in
    this format.  It can be used within a pipeline as an encoding or
    decoding filter, and is most commonly used in this manner as part of an
    automated mail processing system.

OPTIONS
    -d, --decode
        Decodes the input, previously created by base64, to recover
        the original input file.

    -e, --encode
        Encodes the input into an output text file containing its
        base64 encoding.

    -h
    --help
        Display this help and exit.

EXIT STATUS
    base64 returns status 0 if processing was completed without errors and
    1 if an I/O error occurred or errors were detected in decoding a file
    which indicate it is incorrect or incomplete or if processing could
    not be performed at all due, for example, to a nonexistent input file.

AUTHOR
    Written by Jose Narvaez.
"#; /* @MANEND */

fn main() {
    let mut parser = ArgParser::new(3)
        .add_flag(&["d", "decode"])
        .add_flag(&["e", "encode"])
        .add_flag(&["h", "help"]);
    parser.process_common(help_info!("base64"), MAN_PAGE);

    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut stderr = io::stderr();
    let stdin = io::stdin();

    let mut input: Option<Box<Read>> = None;
    let mut output: Option<Box<Write>> = None;

    if parser.args.is_empty() {
        input = Some(Box::new(stdin));
        output = Some(Box::new(stdout));
    } else if parser.args.len() == 1 {
        let input_file_path = Path::new(&parser.args[0]);
        input = Some(Box::new(File::open(input_file_path).try(&mut stderr)));
        output = Some(Box::new(stdout));
    } else if parser.args.len() > 1 {
        let input_file_path = Path::new(&parser.args[0]);
        input = Some(Box::new(File::open(input_file_path).try(&mut stderr)));
        let output_file_path = Path::new(&parser.args[1]);
        output = Some(Box::new(File::create(output_file_path).try(&mut stderr)));
    }

    let mut input = match input {
        Some(input) => input,
        None => fail("error obtaining the input file.", &mut stderr),
    };

    let mut output = match output {
        Some(output) => output,
        None => fail("error obtaining the output file.", &mut stderr),
    };

    if parser.found("encode") {
        encode(input.as_mut(), output.as_mut(), &mut stderr);
    } else if parser.found("decode") {
        decode(input.as_mut(), output.as_mut(), &mut stderr);
    } else {
        encode(input.as_mut(), output.as_mut(), &mut stderr);
    }

    process::exit(0);
}

fn encode<I: ?Sized, O: ?Sized>(input: &mut I, output: &mut O, stderr: &mut Stderr)
    where I: Read,
          O: Write
{
    let mut data = String::new();
    input.read_to_string(&mut data).try(stderr);
    let encoded_data = base64::encode(data.as_bytes());
    output.write_all(encoded_data.as_bytes()).try(stderr);
    output.flush().try(stderr);
}

fn decode<I: ?Sized, O: ?Sized>(input: &mut I, output: &mut O, stderr: &mut Stderr)
    where I: Read,
          O: Write
{
    let mut data = String::new();
    input.read_to_string(&mut data).try(stderr);
    let decoded_data = base64::decode(data.as_bytes()).try(stderr);
    output.write_all(&decoded_data).try(stderr);
    output.flush().try(stderr);
}
