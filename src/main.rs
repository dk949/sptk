use std::error::Error;

use clap::Parser;

use crate::{
    commands::{Pipeline, State, Value, apply_to_value, parse_dispatch},
    io::{read_file, strip_trailing_newline, write_file},
};

mod commands;
mod io;

// echo "some string" | sptk 'S/\s+/ c /{(f,s),a= $a+$f}'

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "PROGRAM")]
    program: String,

    #[arg(short, long, value_name = "FILE", default_value_t = ("-").to_string())]
    input: String,

    #[arg(short, long, value_name = "FILE", default_value_t = ("-").to_string())]
    output: String,

    #[arg(short, long, value_name = "REGEX")]
    split_input: Option<String>,

    /// Don't strip a trailing newline from input or append one on output.
    #[arg(long)]
    raw: bool,
}

fn parse_commands(input: &str) -> Result<Pipeline, commands::Error> {
    let mut input = input.trim_start();
    let mut out = Vec::new();
    while !input.is_empty() {
        let ch = input.chars().next().unwrap();
        let rest = &input[ch.len_utf8()..];
        match parse_dispatch(ch, rest) {
            Some(Ok((cmd, next))) => {
                out.push(cmd);
                input = next.trim_start();
            }
            Some(Err(e)) => return Err(e),
            None => return Err(format!("Unknown command: {ch:?}")),
        }
    }
    Ok(out)
}

fn run_commands(pipeline: Pipeline, input: String) -> Result<State, commands::Error> {
    let mut state = State::new(input);
    for cmd in pipeline {
        let v = apply_to_value(&cmd, state.last())?;
        state.push(v);
    }
    Ok(state)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut data = read_file(&cli.input)?;
    if !cli.raw {
        let trimmed = strip_trailing_newline(&data).len();
        data.truncate(trimmed);
    }
    let pipeline = parse_commands(&cli.program)?;
    let state = run_commands(pipeline, data)?;
    let mut s = match state.into_last() {
        Value::String(s) => s,
        Value::List(_) => {
            return Err("output is not a string; render cmd not yet implemented".into());
        }
    };
    if !cli.raw {
        s.push('\n');
    }
    write_file(&cli.output, &s)?;
    Ok(())
}
