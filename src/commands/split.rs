use regex::Regex;

use crate::commands::{
    Executor, Parser, Result, Value,
    parser::{regex_lit, skip, str_lit},
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
        let preview = inp.chars().next().map_or(String::new(), |c| c.to_string());
        Err(format!(
            "Failed to parse S args at {preview:?}: expected \"string\" or /regex/"
        ))
    }
}

impl Executor for Split {
    fn apply(&self, s: &str) -> Result<Value> {
        let parts: Vec<Value> = match &self.on {
            On::String(sep) => s
                .split(sep.as_str())
                .map(|x| Value::String(x.to_owned()))
                .collect(),
            On::Regex(r) => r.split(s).map(|x| Value::String(x.to_owned())).collect(),
        };
        Ok(Value::List(parts))
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

    #[test]
    fn apply_string() {
        let s = Split {
            on: On::String(" ".into()),
        };
        let out = s.apply("foo bar baz").unwrap();
        match out {
            Value::List(v) => {
                let strs: Vec<&str> = v
                    .iter()
                    .map(|x| match x {
                        Value::String(s) => s.as_str(),
                        _ => panic!("inner not String"),
                    })
                    .collect();
                assert_eq!(strs, vec!["foo", "bar", "baz"]);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn apply_regex() {
        let s = Split {
            on: On::Regex(Regex::new("\\s+").unwrap()),
        };
        let out = s.apply("a  b   c").unwrap();
        match out {
            Value::List(v) => {
                let strs: Vec<&str> = v
                    .iter()
                    .map(|x| match x {
                        Value::String(s) => s.as_str(),
                        _ => panic!(),
                    })
                    .collect();
                assert_eq!(strs, vec!["a", "b", "c"]);
            }
            _ => panic!("expected List"),
        }
    }
}
