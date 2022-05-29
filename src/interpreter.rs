use crate::common::iota;
use crate::error::*;
use std::cmp;
use std::fmt;
use std::iter;
use std::slice;

type InterpFunc = fn(&mut Input, &mut Stack) -> StringResult<()>;
enum Token {
    Char(char),
    Str(String),
    Num(f64),
    Call(InterpFunc),
    Func(InterpFunc),
}

#[derive(Debug)]
enum Input {
    Str(String),
    Num(f64),
    List(Vec<Input>),
}

type Program = Vec<Token>;
type Stack = Vec<Token>;

fn print_stack(stack: &Stack) {
    let min_width = 4;
    if let Some(longest) = stack
        .iter()
        .map(|t| match t {
            Token::Str(s) => s.len(),
            Token::Num(n) => n.to_string().len(),
            Token::Call(_) => 10,
            _ => 0,
        })
        .reduce(cmp::max)
    {
        for item in stack.iter().rev() {
            println!(
                "|{:<pad$}|",
                match item {
                    Token::Char(c) => c.to_string(),
                    Token::Str(s) => s.to_string(),
                    Token::Num(n) => n.to_string(),
                    Token::Call(_) => "[Call]".to_string(),
                    Token::Func(_) => "[Function]".to_string(),
                },
                pad = cmp::max(longest, min_width)
            );
        }
    } else {
        println!("|    |")
    }
}

fn func_form_char(f: char) -> StringResult<InterpFunc> {
    match f {
        _ => Err(format!("'{}' is not usable as a function", f)),
    }
}

fn func_form_u8(f: u8) -> StringResult<InterpFunc> {
    func_form_char(f as char)
}

fn get_string(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<String> {
    let mut out: Vec<u8> = vec![];
    iter.next()
        .into_string_result("Internal Error: Expected start of string literal".to_string())?;
    loop {
        if let Some(ch) = iter.next() {
            match *ch as char {
                '"' => break,
                c => out.push(c as u8),
            }
        } else {
            return Err("Unterminated string literal".to_string());
        }
    }
    String::from_utf8(out)
        .into_string_result("Internal Error: Failed to create string from u8".to_string())
}

fn get_number(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<f64> {
    let mut out: Vec<u8> = vec![];
    loop {
        if let Some(ch) = iter.peek() {
            match **ch as char {
                c @ ('0'..='9' | '.') => {
                    out.push(c as u8);
                    iter.next();
                }
                _ => {
                    break;
                }
            }
        } else {
            break;
        }
    }
    String::from_utf8(out)
        .into_string_result("Internal Error: Failed to create string from u8".to_string())?
        .parse::<f64>()
        .into_string_result_msg()
}

fn get_char(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<char> {
    iter.next()
        .into_string_result("Internal Error: Expected start of char literal".to_string())?;
    return Ok(*iter
        .next()
        .into_string_result("Unexpected end of character literal".to_string())?
        as char);
}

fn get_func(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<InterpFunc> {
    iter.next().into_string_result(
        "Internal Error: Expected start of function pointer literal".to_string(),
    )?;
    func_form_u8(
        *iter
            .next()
            .into_string_result("Unexpected end of function pointer literal".to_string())?,
    )
}

fn filter_chars(mut s: String) -> String {
    s.retain(|c| !iota![32].contains(&(c as usize)));
    return s;
}

#[derive(Debug)]
enum State {
    Normal,
    StrLiteral,
    CharLiteral,
    NumLiteral,
    FuncLiteral,
}

fn make_literal(
    mut iter: &mut iter::Peekable<slice::Iter<u8>>,
    state: State,
) -> StringResult<Token> {
    match state {
        State::StrLiteral => Ok(Token::Str(get_string(&mut iter)?)),
        State::CharLiteral => Ok(Token::Char(get_char(&mut iter)?)),
        State::NumLiteral => Ok(Token::Num(get_number(&mut iter)?)),
        State::FuncLiteral => Ok(Token::Func(get_func(&mut iter)?)),
        _ => Err(format!(
            "Internal Error: Unexpected state '{:?}' in make_literal",
            state
        )),
    }
}

fn parse(program: String) -> StringResult<Program> {
    let mut iter = program.as_bytes().iter().peekable();
    let mut out: Program = vec![];
    let mut state: State = State::Normal;
    loop {
        if let Some(ch) = iter.peek() {
            println!("ch = {:?}; state = {:?}", **ch as char, state);
            print_stack(&out);
            println!("______________________________________");
            match state {
                State::Normal => match **ch as char {
                    '"' => state = State::StrLiteral,
                    '\'' => state = State::CharLiteral,
                    '0'..='9' => state = State::NumLiteral,
                    '`' => state = State::FuncLiteral,
                    _ => todo!(),
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
    let mut input = Input::Str(input);
    let program = parse(filter_chars(program))?;
    let mut stack = Stack::new();
    for instruction in program {
        if let Token::Call(func) = instruction {
            func(&mut input, &mut stack)?;
        } else {
            stack.push(instruction);
        }
    }
    print_stack(&stack);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestInput<T> = (&'static str, &'static [u8], bool, T, Option<&'static u8>);
    fn run<
        T: fmt::Debug + std::cmp::PartialEq,
        F: FnOnce(&mut iter::Peekable<slice::Iter<u8>>) -> StringResult<T>,
    >(
        func: F,
        (name, input, is_ok, expected, remaining): TestInput<T>,
    ) {
        let mut iter = input.iter().peekable();
        let res = func(&mut iter);
        assert_eq!(
            res.is_ok(),
            is_ok,
            "is_ok, test: {}, err: {}",
            name,
            res.unwrap_err()
        );
        if !is_ok {
            return;
        }
        assert_eq!(res.unwrap(), expected);
        assert_eq!(iter.next(), remaining, "remaining, test: {}", name);
    }
    #[test]
    fn get_number_test() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        for test in [
            ("1234"     , "1234".as_bytes()     , true  , 1234.0 , None)               ,
            ("12.34"    , "12.34".as_bytes()    , true  , 12.340 , None)               ,
            (""         , "".as_bytes()         , false , 0.0    , None)               ,
            ("1234."    , "1234.".as_bytes()    , true  , 1234.0 , None)               ,
            ("1234abc"  , "1234abc".as_bytes()  , true  , 1234.0 , Some(&('a' as u8))) ,
            ("abc1234"  , "abc1234".as_bytes()  , false , 0.0    , Some(&('a' as u8))) ,
            ("12.34abc" , "12.34abc".as_bytes() , true  , 12.340 , Some(&('a' as u8))) ,
            ("abc12.34" , "abc12.34".as_bytes() , false , 0.0    , Some(&('a' as u8))) ,

        ] {
            run(get_number, test);
        }
    }

    #[test]
    fn get_string_test() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        for test in [
            ("hello\""  , "hello\"".as_bytes()  , true  , "hello".to_string() , None)               ,
            ("\""       , "\"".as_bytes()       , true  , "".to_string()      , None)               ,
            ("hello"    , "hello".as_bytes()    , false , "".to_string()      , None)               ,
            (""         , "".as_bytes()         , false , "".to_string()      , None)               ,
            ("hello\"a" , "hello\"a".as_bytes() , true  , "hello".to_string() , Some(&('a' as u8))) ,
            ("\"a"      , "\"a".as_bytes()      , true  , "".to_string()      , Some(&('a' as u8))) ,
        ] {
            run(get_string, test);
        }
    }
}
