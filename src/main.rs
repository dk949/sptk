use std::error::Error;

use clap::Parser;

use crate::{
    commands::{
        Executor, ParseCtx, Pipeline, State, Value, apply_to_value, debug, parse_dispatch,
        store::StepStore,
    },
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

    /// Debug-print the final pipeline value (any shape) instead of requiring a String.
    #[arg(short, long)]
    debug: bool,
}

fn parse_commands(input: &str) -> Result<Pipeline, commands::Error> {
    let mut input = input.trim_start();
    let mut out = Vec::new();
    let mut ctx = ParseCtx::new();
    while !input.is_empty() {
        let ch = input.chars().next().unwrap();
        let rest = &input[ch.len_utf8()..];
        match parse_dispatch(ch, rest, &ctx) {
            Some(Ok((cmd, next))) => {
                if cmd.produces_step() {
                    ctx.next_step += 1;
                }
                out.push(cmd);
                input = next.trim_start();
            }
            Some(Err(e)) => return Err(e),
            None => return Err(format!("Unknown command: {ch:?}")),
        }
    }
    Ok(out)
}

fn compute_needed(pipeline: &Pipeline) -> Vec<bool> {
    let n_steps = pipeline.iter().filter(|c| c.produces_step()).count();
    let mut needed = vec![false; n_steps + 1];
    let mut refs = Vec::new();
    for cmd in pipeline {
        cmd.refs(&mut refs);
    }
    for r in refs {
        if r < needed.len() {
            needed[r] = true;
        }
    }
    needed
}

fn run_commands(pipeline: Pipeline, input: String) -> Result<State, commands::Error> {
    let needed = compute_needed(&pipeline);
    let mut store = StepStore::new(needed.len());
    let mut current = Value::String(input);
    if needed[0] {
        store.set(0, current.clone());
    }
    let mut step = 0;
    for cmd in &pipeline {
        let resolved = cmd.resolve(&store)?;
        let cmd_ref: &commands::Command = resolved.as_ref().unwrap_or(cmd);
        let next = if let Some(res) = cmd_ref.apply_pipeline(&store) {
            res?
        } else {
            apply_to_value(cmd_ref, &current)?
        };
        if cmd_ref.produces_step() {
            step += 1;
            if needed[step] {
                store.set(step, next.clone());
            }
        }
        current = next;
    }
    Ok(State::new(current))
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
    let final_value = state.into_last();
    if cli.debug {
        let mut buf = String::new();
        debug::print(&final_value, &mut buf).expect("fmt::Write on String never fails");
        write_file(&cli.output, &buf)?;
        return Ok(());
    }
    let mut s = match final_value {
        Value::String(s) => s,
        Value::List(_) | Value::Int(_) | Value::Float(_) => {
            return Err("output is not a string; render cmd not yet implemented".into());
        }
    };
    if !cli.raw {
        s.push('\n');
    }
    write_file(&cli.output, &s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(prog: &str, input: &str) -> Result<Value, commands::Error> {
        let pipeline = parse_commands(prog)?;
        let state = run_commands(pipeline, input.into())?;
        Ok(state.into_last())
    }

    fn as_str(v: Value) -> String {
        match v {
            Value::String(s) => s,
            _ => panic!("expected String, got {v:?}"),
        }
    }

    #[test]
    fn pipe_ref_replays_step_one() {
        // Plan example: S" " J "," $1 J "." on "1 2 3" -> "1.2.3"
        let out = run("S\" \" J \",\" $1 J \".\"", "1 2 3").unwrap();
        assert_eq!(as_str(out), "1.2.3");
    }

    #[test]
    fn pipe_ref_step_zero_is_original_input() {
        // S splits, J joins (mutating), $0 restores original, J"." rebuilds.
        let out = run("S\" \" J \",\" $0", "a b c").unwrap();
        assert_eq!(as_str(out), "a b c");
    }

    #[test]
    fn pipe_ref_zero_alone() {
        // `$0` with nothing else: returns input verbatim.
        let out = run("$0", "hello").unwrap();
        assert_eq!(as_str(out), "hello");
    }

    #[test]
    fn pipe_ref_forward_ref_errors() {
        // $1 before any step-producing cmd is a forward ref.
        assert!(parse_commands("$1").is_err());
        // $2 after only one step-producing cmd is also forward.
        assert!(parse_commands("S\" \" $2").is_err());
    }

    #[test]
    fn pipe_ref_self_ref_errors() {
        // $1 at the position of step 1 itself: forward ref.
        assert!(parse_commands("$1 S\" \"").is_err());
    }

    #[test]
    fn pipe_ref_does_not_count_as_step() {
        // S=step1, $0=replace, S=step2: only steps 0..=2 exist. $3 errors.
        // (If `$` counted as a step we'd have 3 steps and $3 would be valid.)
        assert!(parse_commands("S\" \" $0 S\" \" $3").is_err());
        // $2 references the second S (step 2); valid.
        assert!(parse_commands("S\" \" $0 S\" \" $2").is_ok());
    }

    #[test]
    fn pipe_ref_multiple_refs() {
        // Two `$` cmds referencing the same step: both load step 1's output.
        let out = run("S\" \" J \",\" $1 J \"-\" $1 J \"+\"", "a b c").unwrap();
        assert_eq!(as_str(out), "a+b+c");
    }

    // `${N}` arg substitution -------------------------------------

    #[test]
    fn arg_subst_join_with_string_step() {
        // Step 0 = "-" (input). Step 1 = split into chars. Then J ${0}: use
        // step 0's value as the sep.
        // Use $0 + S to set things up: input is the sep, get a separate list.
        // Simpler form: feed list-of-chars to a Join whose sep is ${0}.
        // input = "-", S"" splits to chars "-", then $0 replaces with "-",
        // then S"" again splits to chars again... too convoluted.
        //
        // Direct: S " " on "a b c" -> ["a","b","c"] (step 1). $0 puts back
        // the original input "a b c" (a String). J ${0} on step 1 list?
        // No — $0 made current value the string. Pipeline order matters.
        //
        // Cleanest: split input by spaces, then join with the original input
        // string as the separator.
        let out = run("S\" \" J ${0}", "X").unwrap();
        // input "X" splits on space -> ["X"] (single elem), joining anything
        // single-element produces just "X". Not interesting; use real input.
        assert_eq!(as_str(out), "X");
    }

    #[test]
    fn arg_subst_join_uses_step_output_as_sep() {
        // Step 1 = J"-" on input split. Step 2 = J ${1} on step 1's output?
        // J needs a list input. Step 1's output is a string. So Join can't
        // apply to that. Use $1 to recover the list-of-tokens (output of S).
        //
        // Pipeline: S" " (step1=list) J"," (step2=string "a,b,c") $1 (back to
        // list) J ${2} (join list with step 2's "a,b,c" as separator).
        let out = run("S\" \" J\",\" $1 J ${2}", "a b c").unwrap();
        assert_eq!(as_str(out), "aa,b,cba,b,cc");
    }

    #[test]
    fn arg_subst_step_zero_as_sep() {
        // $0 = original input. Use it as Join's separator after splitting.
        let out = run("S\" \" J ${0}", "a b c").unwrap();
        // Original input is "a b c"; sep is the whole "a b c".
        assert_eq!(as_str(out), "aa b cba b cc");
    }

    #[test]
    fn arg_subst_forward_ref_errors() {
        // J ${1} as first cmd: only step 0 exists.
        assert!(parse_commands("J ${1}").is_err());
    }

    #[test]
    fn arg_subst_braces_required() {
        // Bare $1 is NOT a Join arg (J only accepts "str" or ${N}).
        // J $1 should fail to parse as a Join arg.
        assert!(parse_commands("S\" \" J $1").is_err());
    }

    #[test]
    fn arg_subst_string_in_quotes_is_literal() {
        // "${1}" inside a string literal is just text, not interpolation.
        // Pipeline: S" " J"${1}" on "a b c": uses literal "${1}" as sep.
        let out = run("S\" \" J\"${1}\"", "a b c").unwrap();
        assert_eq!(as_str(out), "a${1}b${1}c");
    }

    #[test]
    fn arg_subst_list_to_string_errors() {
        // J ${1} where step 1 is a List: coerce error at run time.
        // S" " produces list, then J ${1} would try to coerce step 1 (list) to
        // a string separator.
        let err = run("S\" \" J ${1}", "a b c").unwrap_err();
        assert!(
            err.contains("List"),
            "expected list-coerce error, got: {err}"
        );
    }
}
