use crate::commands::{
    Executor, Parser, Result, Value,
    parser::{make_err, skip, str_lit},
};

pub struct Join {
    on: String,
}

impl Parser for Join {
    const CHAR: char = 'J';

    fn parse(inp: &str) -> Result<(Self, &str)> {
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
        let mut parts: Vec<&str> = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Value::String(s) => parts.push(s),
                Value::List(_) => return Some(Err("J: list element is not a String".into())),
            }
        }
        Some(Ok(Value::String(parts.join(&self.on))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string_arg() {
        let (j, rest) = Join::parse("\"-\"after").unwrap();
        assert_eq!(j.on, "-");
        assert_eq!(rest, "after");
    }

    #[test]
    fn parse_missing_arg() {
        assert!(Join::parse("/regex/").is_err());
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
}
