use std::fmt::Write;

use crate::commands::{
    Executor, ParseCtx, Parser, Result, Value,
    arg::Arg,
    parser::{make_err, pipeline_ref, skip, str_lit},
    store::StepStore,
};

pub struct Join {
    on: Arg<String>,
}

impl Parser for Join {
    const CHAR: char = 'J';

    fn parse<'a>(inp: &'a str, ctx: &ParseCtx) -> Result<(Self, &'a str)> {
        let inp = skip(inp);
        if let Some(res) = str_lit(inp) {
            let (s, next) = res?;
            return Ok((Join { on: Arg::Lit(s) }, next));
        }
        if let Some(res) = pipeline_ref(inp, ctx) {
            let (n, next) = res?;
            return Ok((Join { on: Arg::Sub(n) }, next));
        }
        make_err::<_, Join>(inp, "Expected \"String\" or ${N}")
    }
}

impl Executor for Join {
    fn resolve(&self, store: &StepStore) -> Result<Option<Self>> {
        match &self.on {
            Arg::Lit(_) => Ok(None),
            Arg::Sub(_) => Ok(Some(Join {
                on: Arg::Lit(self.on.resolve(store)?),
            })),
        }
    }

    fn refs(&self, out: &mut Vec<usize>) {
        self.on.add_refs(out);
    }

    fn apply_list(&self, items: &[Value]) -> Option<Result<Value>> {
        let sep = match &self.on {
            Arg::Lit(s) => s.as_str(),
            Arg::Sub(_) => {
                return Some(Err(
                    "J: unresolved ${N} arg (internal: resolve not called)".into()
                ));
            }
        };
        let mut out = String::new();
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                out.push_str(sep);
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

    fn lit(s: &str) -> Join {
        Join {
            on: Arg::Lit(s.into()),
        }
    }

    #[test]
    fn parse_string_arg() {
        let (j, rest) = Join::parse("\"-\"after", &ParseCtx::new()).unwrap();
        assert!(matches!(j.on, Arg::Lit(ref s) if s == "-"));
        assert_eq!(rest, "after");
    }

    #[test]
    fn parse_pipeline_ref_arg() {
        let ctx = ParseCtx { next_step: 3 };
        let (j, rest) = Join::parse("${1}rest", &ctx).unwrap();
        assert!(matches!(j.on, Arg::Sub(1)));
        assert_eq!(rest, "rest");
    }

    #[test]
    fn parse_forward_ref_errors() {
        let ctx = ParseCtx { next_step: 1 };
        assert!(Join::parse("${1}", &ctx).is_err());
    }

    #[test]
    fn parse_missing_arg() {
        assert!(Join::parse("/regex/", &ParseCtx::new()).is_err());
    }

    #[test]
    fn apply_list_happy() {
        let out = lit("-")
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
        assert!(
            lit("-")
                .apply_list(&[Value::String("a".into()), Value::List(vec![])])
                .unwrap()
                .is_err()
        );
    }

    #[test]
    fn does_not_accept_str() {
        assert!(lit("-").apply_str("xyz").is_none());
    }

    #[test]
    fn apply_list_stringifies_ints() {
        let out = lit(" ")
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
        let out = lit(",")
            .apply_list(&[Value::String("a".into()), Value::Int(2), Value::Float(3.5)])
            .unwrap()
            .unwrap();
        match out {
            Value::String(s) => assert_eq!(s, "a,2,3.5"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn resolve_replaces_sub_with_lit() {
        let mut s = StepStore::new(2);
        s.set(1, Value::String("--".into()));
        let j = Join { on: Arg::Sub(1) };
        let resolved = j.resolve(&s).unwrap().expect("resolve produces a clone");
        let out = resolved
            .apply_list(&[Value::String("a".into()), Value::String("b".into())])
            .unwrap()
            .unwrap();
        match out {
            Value::String(s) => assert_eq!(s, "a--b"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn resolve_lit_returns_none() {
        let store = StepStore::new(1);
        let j = lit("-");
        assert!(j.resolve(&store).unwrap().is_none());
    }

    #[test]
    fn refs_reports_sub_index() {
        let mut out = Vec::new();
        Join { on: Arg::Sub(3) }.refs(&mut out);
        assert_eq!(out, vec![3]);
        let mut out2 = Vec::new();
        lit("-").refs(&mut out2);
        assert!(out2.is_empty());
    }
}
