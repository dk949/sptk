use regex::Regex;

use crate::commands::{
    Executor, ParseCtx, Parser, Result, Value,
    arg::Arg,
    parser::{make_err, regex_lit, skip, str_lit},
    store::StepStore,
};

pub enum On {
    String(Arg<String>),
    Regex(Arg<Regex>),
}

pub struct Split {
    on: On,
}

impl Parser for Split {
    const CHAR: char = 'S';

    fn parse<'a>(inp: &'a str, _ctx: &ParseCtx) -> Result<(Self, &'a str)> {
        let inp = skip(inp);
        if let Some(res) = str_lit(inp) {
            let (s, next) = res?;
            return Ok((
                Split {
                    on: On::String(Arg::Lit(s)),
                },
                next,
            ));
        }
        if let Some(res) = regex_lit(inp) {
            let (r, next) = res?;
            return Ok((
                Split {
                    on: On::Regex(Arg::Lit(r)),
                },
                next,
            ));
        }
        // NOTE: `${N}` not accepted here yet: a bare sub doesn't say whether
        // the target is a string separator or a regex. Awaiting a
        // disambiguation syntax.
        make_err::<_, Split>(inp, "Expected \"string\" or /regex/")
    }
}

impl Executor for Split {
    fn resolve(&self, store: &StepStore) -> Result<Option<Self>> {
        match &self.on {
            On::String(a @ Arg::Sub(_)) => Ok(Some(Split {
                on: On::String(Arg::Lit(a.resolve(store)?)),
            })),
            On::Regex(a @ Arg::Sub(_)) => Ok(Some(Split {
                on: On::Regex(Arg::Lit(a.resolve(store)?)),
            })),
            _ => Ok(None),
        }
    }

    fn refs(&self, out: &mut Vec<usize>) {
        match &self.on {
            On::String(a) => a.add_refs(out),
            On::Regex(a) => a.add_refs(out),
        }
    }

    fn apply_str(&self, s: &str) -> Option<Result<Value>> {
        let parts: Vec<Value> = match &self.on {
            On::String(Arg::Lit(sep)) => s
                .split(sep.as_str())
                .map(|x| Value::String(x.to_owned()))
                .collect(),
            On::Regex(Arg::Lit(r)) => r.split(s).map(|x| Value::String(x.to_owned())).collect(),
            On::String(Arg::Sub(_)) | On::Regex(Arg::Sub(_)) => {
                return Some(Err(
                    "S: unresolved ${N} arg (internal: resolve not called)".into()
                ));
            }
        };
        Some(Ok(Value::List(parts)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_split(sep: &str) -> Split {
        Split {
            on: On::String(Arg::Lit(sep.into())),
        }
    }

    fn re_split(pat: &str) -> Split {
        Split {
            on: On::Regex(Arg::Lit(Regex::new(pat).unwrap())),
        }
    }

    #[test]
    fn parse_string_arg() {
        let (s, rest) = Split::parse("\" \"after", &ParseCtx::new()).unwrap();
        assert!(matches!(s.on, On::String(Arg::Lit(ref x)) if x == " "));
        assert_eq!(rest, "after");
    }

    #[test]
    fn parse_regex_arg() {
        let (s, rest) = Split::parse("/\\s+/after", &ParseCtx::new()).unwrap();
        assert!(matches!(s.on, On::Regex(Arg::Lit(_))));
        assert_eq!(rest, "after");
    }

    #[test]
    fn parse_missing_arg() {
        assert!(Split::parse("xyz", &ParseCtx::new()).is_err());
    }

    #[test]
    fn parse_rejects_pipeline_ref() {
        // Until S has a disambiguation syntax for string vs regex subs,
        // bare ${N} is not accepted.
        let ctx = ParseCtx { next_step: 3 };
        assert!(Split::parse("${1}", &ctx).is_err());
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
        let out = str_split(" ").apply_str("foo bar baz").unwrap().unwrap();
        assert_eq!(strs_of(out), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn apply_regex() {
        let out = re_split("\\s+").apply_str("a  b   c").unwrap().unwrap();
        assert_eq!(strs_of(out), vec!["a", "b", "c"]);
    }

    #[test]
    fn does_not_accept_list() {
        assert!(str_split(" ").apply_list(&[]).is_none());
    }
}
