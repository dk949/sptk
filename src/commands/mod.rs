pub mod join;
pub mod parser;
pub mod split;

use split::Split;

use crate::commands::join::Join;

pub type Error = String;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    List(Vec<Value>),
}

#[derive(Debug)]
pub struct State {
    last: Value,
}

impl State {
    pub fn new(s: String) -> Self {
        Self {
            last: Value::String(s),
        }
    }

    pub fn push(&mut self, v: Value) {
        self.last = v;
    }

    pub fn last(&self) -> &Value {
        &self.last
    }

    pub fn into_last(self) -> Value {
        self.last
    }
}

pub trait Parser: Sized {
    const CHAR: char;
    fn parse(inp: &str) -> Result<(Self, &str)>;
}

pub trait Executor {
    /// Default `None` = cmd does not accept this shape. The framework then tries
    /// the other variants and/or promotes (String → List<char>) before giving up.
    /// `Some(Ok|Err)` = cmd accepts the shape and ran (successfully or not).
    fn apply_str(&self, _s: &str) -> Option<Result<Value>> {
        None
    }
    fn apply_list(&self, _items: &[Value]) -> Option<Result<Value>> {
        None
    }
}

pub type Pipeline = Vec<Command>;

macro_rules! cmds {
    ( $( $ty:ident ),+ $(,)? ) => {
        pub enum Command {
            $( $ty($ty), )+
        }

        $(
            impl From<$ty> for Command {
                fn from(v: $ty) -> Self { Command::$ty(v) }
            }
        )+

        impl Executor for Command {
            fn apply_str(&self, s: &str) -> Option<Result<Value>> {
                match self {
                    $( Command::$ty(inner) => inner.apply_str(s), )+
                }
            }
            fn apply_list(&self, items: &[Value]) -> Option<Result<Value>> {
                match self {
                    $( Command::$ty(inner) => inner.apply_list(items), )+
                }
            }
        }

        pub fn parse_dispatch(ch: char, rest: &str)
            -> Option<Result<(Command, &str)>>
        {
            match ch {
                $( <$ty as Parser>::CHAR =>
                    Some(<$ty>::parse(rest).map(|(v, n)| (v.into(), n))), )+
                _ => None,
            }
        }
    };
}

cmds! { Split, Join }

pub fn apply_to_value(cmd: &Command, v: &Value) -> Result<Value> {
    match v {
        Value::String(s) => {
            if let Some(res) = cmd.apply_str(s) {
                return res;
            }
            // Promote String → List of single-char Strings, then retry as list.
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            cmd.apply_list(&chars)
                .unwrap_or_else(|| Err("Cmd accepts neither String nor List input".into()))
        }
        Value::List(items) => {
            let all_strings = items.iter().all(|i| matches!(i, Value::String(_)));
            if all_strings && let Some(res) = cmd.apply_list(items) {
                return res;
            }
            items
                .iter()
                .map(|item| apply_to_value(cmd, item))
                .collect::<Result<Vec<_>>>()
                .map(Value::List)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> Value {
        Value::String(x.into())
    }
    fn l(xs: Vec<Value>) -> Value {
        Value::List(xs)
    }

    fn eq(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::String(x), Value::String(y)) => x == y,
            (Value::List(x), Value::List(y)) => {
                x.len() == y.len() && x.iter().zip(y).all(|(a, b)| eq(a, b))
            }
            _ => false,
        }
    }

    // A test-only str-accepting cmd that wraps the string in a single-elem list.
    struct WrapStr;
    impl Executor for WrapStr {
        fn apply_str(&self, s: &str) -> Option<Result<Value>> {
            Some(Ok(Value::List(vec![Value::String(s.to_owned())])))
        }
    }

    // A test-only list-accepting cmd that concats inner strings.
    struct ConcatList;
    impl Executor for ConcatList {
        fn apply_list(&self, items: &[Value]) -> Option<Result<Value>> {
            let out: String = items
                .iter()
                .filter_map(|x| match x {
                    Value::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            Some(Ok(Value::String(out)))
        }
    }

    // Drive these test cmds through the same dispatch shape apply_to_value uses,
    // but without going through Command. Mirrors framework logic exactly.
    fn drive<E: Executor>(cmd: &E, v: &Value) -> Result<Value> {
        match v {
            Value::String(s) => {
                if let Some(res) = cmd.apply_str(s) {
                    return res;
                }
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                cmd.apply_list(&chars)
                    .unwrap_or_else(|| Err("neither".into()))
            }
            Value::List(items) => {
                let all_strings = items.iter().all(|i| matches!(i, Value::String(_)));
                if all_strings && let Some(res) = cmd.apply_list(items) {
                    return res;
                }
                items
                    .iter()
                    .map(|item| drive(cmd, item))
                    .collect::<Result<Vec<_>>>()
                    .map(Value::List)
            }
        }
    }

    #[test]
    fn str_cmd_on_string() {
        let out = drive(&WrapStr, &s("foo")).unwrap();
        assert!(eq(&out, &l(vec![s("foo")])));
    }

    #[test]
    fn str_cmd_leaf_maps_over_lists() {
        let input = l(vec![l(vec![s("a"), s("b")]), l(vec![s("c")])]);
        let out = drive(&WrapStr, &input).unwrap();
        let want = l(vec![
            l(vec![l(vec![s("a")]), l(vec![s("b")])]),
            l(vec![l(vec![s("c")])]),
        ]);
        assert!(eq(&out, &want));
    }

    #[test]
    fn list_cmd_on_innermost_list() {
        let out = drive(&ConcatList, &l(vec![s("a"), s("b"), s("c")])).unwrap();
        assert!(eq(&out, &s("abc")));
    }

    #[test]
    fn list_cmd_leaf_maps_over_outer_lists() {
        let input = l(vec![l(vec![s("a"), s("b")]), l(vec![s("c"), s("d")])]);
        let out = drive(&ConcatList, &input).unwrap();
        assert!(eq(&out, &l(vec![s("ab"), s("cd")])));
    }

    #[test]
    fn list_cmd_promotes_string_to_chars() {
        // ConcatList accepts only list; on a String, framework promotes to
        // List<char-as-String> then applies → re-concats back to original.
        let out = drive(&ConcatList, &s("hello")).unwrap();
        assert!(eq(&out, &s("hello")));
    }

    #[test]
    fn neither_method_errors() {
        struct Neither;
        impl Executor for Neither {}
        assert!(drive(&Neither, &s("x")).is_err());
    }
}
