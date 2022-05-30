mod compiler;
mod funcs;
#[cfg(test)]
mod tests;
mod types;

use super::error::*;
use compiler::*;
use types::*;

fn parse(program: String) -> StringResult<Program> {
    let mut iter = program.as_bytes().iter().peekable();
    let mut out: Program = vec![];
    let mut state: State = State::Normal;
    loop {
        if let Some(ch) = iter.peek() {
            match state {
                State::Normal => match **ch as char {
                    '"' => state = State::StrLiteral,
                    '\'' => state = State::CharLiteral,
                    '0'..='9' => state = State::NumLiteral,
                    '`' => state = State::FuncLiteral,
                    _ => state = State::CallLiteral,
                },
                _ => {
                    out.push(make_literal(&mut iter, state)?);
                    state = State::Normal;
                }
            }
        } else {
            break;
        }
    }
    Ok(out)
}

pub fn run((input, program): (String, String)) -> StringResult<()> {
    let mut input = Input::Str(input.strip_nl());
    let program = parse(filter_chars(program))?;
    let mut stack = Stack::new();
    for instruction in program {
        if let Token::Call(func) = instruction {
            func(&mut input, &mut stack)?;
        } else {
            stack.push_back(instruction);
        }
    }
    println!("{}", input);
    //print_stack(&stack);
    Ok(())
}
