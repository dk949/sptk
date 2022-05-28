use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;

struct Args {
    pub program: Option<String>,
    pub input: Option<String>,
}

fn help() -> ! {
    todo!("help");
    //process::exit(0)
}

fn version() -> ! {
    todo!("version");
    //process::exit(0)
}

fn parse() -> io::Result<Args> {
    let mut args = Args {
        program: None,
        input: None,
    };

    let mut iter = env::args();
    iter.next();
    loop {
        if let Some(arg) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => help(),
                "-v" | "--version" => version(),
                "-f" | "--file" => {
                    if args.program.is_none() {
                        args.program = Some(fs::read_to_string(
                            iter.next().expect("expected file name after -f|--file"),
                        )?);
                    } else {
                        panic!("too many programs");
                    }
                }
                s => {
                    if args.program.is_none() {
                        args.program = Some(s.to_string());
                    } else {
                        if args.input.is_none() {
                            args.input = Some(fs::read_to_string(s)?)
                        } else {
                            panic!("Too many input files");
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    if args.program.is_none() {
        panic!("No program provided");
    }
    return Ok(args);
}

fn get_input() -> io::Result<String> {
    let mut buf = vec![];
    io::stdin().read_to_end(&mut buf)?;
    String::from_utf8(buf).or(Err(io::Error::from(io::ErrorKind::Other)))
}

pub fn input_and_program() -> io::Result<(String, String)> {
    let args = parse()?;
    Ok((
        if let Some(input) = args.input {
            input
        } else {
            get_input()?
        },
        args.program.expect("Internal error: no program found"),
    ))
}
