use crate::error::*;
use crate::file_io;
use std::env;

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

fn parse() -> StringResult<Args> {
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
                        args.program =
                            Some(file_io::read_to_string(iter.next().into_string_result(
                                "expected file name after -f|--file".to_string(),
                            )?)?);
                    } else {
                        return Err("too many programs".to_string());
                    }
                }
                s => {
                    if args.program.is_none() {
                        args.program = Some(s.to_string());
                    } else {
                        if args.input.is_none() {
                            args.input = Some(file_io::read_to_string(s)?)
                        } else {
                            return Err("Too many input files".to_string());
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    if args.program.is_none() {
        Err("No program provided".to_string())
    } else {
        Ok(args)
    }
}

pub fn get_input_and_program() -> StringResult<(String, String)> {
    let args = parse()?;
    Ok((
        if let Some(input) = args.input {
            input
        } else {
            file_io::get_input()?
        },
        args.program
            .into_string_result("Internal error: no program found".to_string())?,
    ))
}
