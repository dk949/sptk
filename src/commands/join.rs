use std::fmt::Write;

use crate::commands::{
    Executor, ParseCtx, Parser, Result, Value,
    parser::{make_err, skip, str_lit},
};

pub struct Join {
    on: String,
}

impl Parser for Join {
    const CHAR: char = 'J';

    fn parse<'a>(inp: &'a str, _ctx: &ParseCtx) -> Result<(Self, &'a str)> {
        let inp = skip(inp);
        if let Some(res) = str_lit(inp) {
            let (s, next) = res?;
            return Ok((Join { on: s }, next));
        }
        make_err::<_, Join>(inp, "Expected \"String\"")
    }
}

impl Executor for Join {
    fn apply_list(&self, items: &[Value]) -> Option<Result<Value>> {
        let mut out = String::new();
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                out.push_str(&self.on);
            }
            match item {
                Value::String(s) => out.push_str(s),
                Value::Int(n) => write!(out, "{n}").unwrap(),
                Value::Float(n) => write!(out, "{n}").unwrap(),
                Value::List(_) => {
                    return Some(Err("J: nested list cannot be joined".into()));
                }
            }
        }
        Some(Ok(Value::String(out)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string_arg() {
        let (j, rest) = Join::parse("\"-\"after", &ParseCtx::new()).unwrap();
        assert_eq!(j.on, "-");
        assert_eq!(rest, "after");
    }

    #[test]
    fn parse_missing_arg() {
        assert!(Join::parse("/regex/", &ParseCtx::new()).is_err());
    }

    #[test]
    fn apply_list_happy() {
        let j = Join { on: "-".into() };
        let out = j
            .apply_list(&[
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
            ])
            .unwrap()
            .unwrap();
        match out {
            Value::String(s) => assert_eq!(s, "a-b-c"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn apply_list_rejects_nested() {
        let j = Join { on: "-".into() };
        assert!(
            j.apply_list(&[Value::String("a".into()), Value::List(vec![])])
                .unwrap()
                .is_err()
        );
    }

    #[test]
    fn does_not_accept_str() {
        let j = Join { on: "-".into() };
        assert!(j.apply_str("xyz").is_none());
    }

    #[test]
    fn apply_list_stringifies_ints() {
        let j = Join { on: " ".into() };
        let out = j
            .apply_list(&[Value::Int(1), Value::Int(2), Value::Int(3)])
            .unwrap()
            .unwrap();
        match out {
            Value::String(s) => assert_eq!(s, "1 2 3"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn apply_list_stringifies_mixed() {
        let j = Join { on: ",".into() };
        let out = j
            .apply_list(&[Value::String("a".into()), Value::Int(2), Value::Float(3.5)])
            .unwrap()
            .unwrap();
        match out {
            Value::String(s) => assert_eq!(s, "a,2,3.5"),
            _ => panic!("expected String"),
        }
    }
}
