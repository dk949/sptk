use regex::Regex;

use crate::commands::{
    Executor, Parser, Result, Value,
    parser::{make_err, regex_lit, skip, str_lit},
};

pub enum On {
    String(String),
    Regex(Regex),
}

pub struct Split {
    on: On,
}

impl Parser for Split {
    const CHAR: char = 'S';

    fn parse(inp: &str) -> Result<(Self, &str)> {
        let inp = skip(inp);
        if let Some(res) = str_lit(inp) {
            let (s, next) = res?;
            return Ok((Split { on: On::String(s) }, next));
        }
        if let Some(res) = regex_lit(inp) {
            let (r, next) = res?;
            return Ok((Split { on: On::Regex(r) }, next));
        }
        make_err::<_, Split>(inp, "Expected \"string\" or /regex/")
    }
}

impl Executor for Split {
    fn apply_str(&self, s: &str) -> Option<Result<Value>> {
        let parts: Vec<Value> = match &self.on {
            On::String(sep) => s
                .split(sep.as_str())
                .map(|x| Value::String(x.to_owned()))
                .collect(),
            On::Regex(r) => r.split(s).map(|x| Value::String(x.to_owned())).collect(),
        };
        Some(Ok(Value::List(parts)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string_arg() {
        let (s, rest) = Split::parse("\" \"after").unwrap();
        assert!(matches!(s.on, On::String(ref x) if x == " "));
        assert_eq!(rest, "after");
    }

    #[test]
    fn parse_regex_arg() {
        let (s, rest) = Split::parse("/\\s+/after").unwrap();
        assert!(matches!(s.on, On::Regex(_)));
        assert_eq!(rest, "after");
    }

    #[test]
    fn parse_missing_arg() {
        assert!(Split::parse("xyz").is_err());
    }

    fn strs_of(v: Value) -> Vec<String> {
        match v {
            Value::List(items) => items
                .into_iter()
                .map(|x| match x {
                    Value::String(s) => s,
                    _ => panic!("inner not String"),
                })
                .collect(),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn apply_string() {
        let s = Split {
            on: On::String(" ".into()),
        };
        let out = s.apply_str("foo bar baz").unwrap().unwrap();
        assert_eq!(strs_of(out), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn apply_regex() {
        let s = Split {
            on: On::Regex(Regex::new("\\s+").unwrap()),
        };
        let out = s.apply_str("a  b   c").unwrap().unwrap();
        assert_eq!(strs_of(out), vec!["a", "b", "c"]);
    }

    #[test]
    fn does_not_accept_list() {
        let s = Split {
            on: On::String(" ".into()),
        };
        assert!(s.apply_list(&[]).is_none());
    }
}
