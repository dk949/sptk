use crate::common::iota;
use crate::error::*;
use crate::interpreter::funcs;
use crate::interpreter::types::*;
use std::iter;
use std::slice;

pub trait StripNl {
    fn strip_nl(self) -> Self;
}
impl StripNl for String {
    fn strip_nl(mut self) -> Self {
        if self.ends_with('\n') {
            unsafe {
                self.as_mut_vec().pop();
            }
            self
        } else {
            self
        }
    }
}

pub trait CharOrU8 {
    fn as_char(&self) -> char;
    fn as_u8(&self) -> u8;
}

impl CharOrU8 for char {
    fn as_char(&self) -> char {
        self.clone()
    }
    fn as_u8(&self) -> u8 {
        self.clone() as u8
    }
}

impl CharOrU8 for u8 {
    fn as_char(&self) -> char {
        self.clone() as char
    }
    fn as_u8(&self) -> u8 {
        self.clone()
    }
}

pub fn func_map<T: CharOrU8>(f: T) -> StringResult<InterpFunc> {
    match f.as_char() {
        '+' => Ok(funcs::plus),
        _ => Err(format!("'{}' is not usable as a function", f.as_char())),
    }
}

pub fn get_string(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<String> {
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

pub fn get_number(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<f64> {
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

pub fn get_char(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<char> {
    iter.next()
        .into_string_result("Internal Error: Expected start of char literal".to_string())?;
    return Ok(*iter
        .next()
        .into_string_result("Unexpected end of character literal".to_string())?
        as char);
}

pub fn get_func(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<InterpFunc> {
    iter.next().into_string_result(
        "Internal Error: Expected start of function pointer literal".to_string(),
    )?;
    func_map(
        *iter
            .next()
            .into_string_result("Unexpected end of function pointer literal".to_string())?,
    )
}

pub fn get_call(iter: &mut iter::Peekable<slice::Iter<u8>>) -> StringResult<InterpFunc> {
    func_map(*iter.next().into_string_result(
        "Internal Error: Expected start of function call literal".to_string(),
    )?)
}

pub fn filter_chars(mut s: String) -> String {
    s.retain(|c| !iota![32].contains(&(c as usize)));
    return s;
}

pub fn make_literal(
    mut iter: &mut iter::Peekable<slice::Iter<u8>>,
    state: State,
) -> StringResult<Token> {
    match state {
        State::StrLiteral => Ok(Token::Str(get_string(&mut iter)?)),
        State::CharLiteral => Ok(Token::Char(get_char(&mut iter)?)),
        State::NumLiteral => Ok(Token::Num(get_number(&mut iter)?)),
        State::FuncLiteral => Ok(Token::Func(get_func(&mut iter)?)),
        State::CallLiteral => Ok(Token::Call(get_call(&mut iter)?)),
        _ => Err(format!(
            "Internal Error: Unexpected state '{:?}' in make_literal",
            state
        )),
    }
}
