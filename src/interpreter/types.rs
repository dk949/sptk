use crate::error::StringResult;
use std::cmp;
use std::collections::VecDeque;
use std::fmt;

pub type InterpFunc = fn(&mut Input, &mut Stack) -> StringResult<()>;
pub enum Token {
    Char(char),
    Str(String),
    Num(f64),
    Call(InterpFunc),
    Func(InterpFunc),
}

#[derive(Debug)]
pub enum Input {
    Str(String),
    Num(f64),
    List(Vec<Input>),
}

pub type Program = Vec<Token>;
pub type Stack = VecDeque<Token>;

pub fn print_stack(stack: &Stack) {
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
                format!("{:?}", item),
                pad = cmp::max(longest, min_width)
            );
        }
    } else {
        println!("|    |")
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Char(c) => c.to_string(),
                Token::Str(s) => s.to_string(),
                Token::Num(n) => n.to_string(),
                Token::Call(_) => "[Call]".to_string(),
                Token::Func(_) => "[Function]".to_string(),
            }
        )
    }
}
impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Input::Str(s) => write!(f, "{}", s),
            Input::Num(n) => write!(f, "{}", n),
            Input::List(l) => {
                write!(f, "[")?;
                write!(f, "{}", l[0])?;
                for i in l.iter().skip(1) {
                    write!(f, ", {}", i)?;
                }
                write!(f, "]")?;
                Ok(())
            }
        }
        //
    }
}

#[derive(Debug)]
pub enum State {
    Normal,
    StrLiteral,
    CharLiteral,
    NumLiteral,
    FuncLiteral,
    CallLiteral,
}
