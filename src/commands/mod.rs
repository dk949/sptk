pub mod arg;
pub mod coerce;
pub mod debug;
pub mod join;
pub mod number;
pub mod parser;
pub mod pipe_ref;
pub mod split;
pub mod store;

use split::Split;

use crate::commands::{
    debug::Dbg, join::Join, number::Number, pipe_ref::PipeRef, store::StepStore,
};

pub type Error = String;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    List(Vec<Value>),
    Int(i64),
    Float(f64),
}

#[derive(Debug)]
pub struct State {
    last: Value,
}

impl State {
    pub fn new(v: Value) -> Self {
        Self { last: v }
    }

    pub fn into_last(self) -> Value {
        self.last
    }
}

/// Parse-time context. Carries the step index the next step-producing cmd
/// will receive (1-based). Lets `$N` validate that `N < next_step`.
pub struct ParseCtx {
    pub next_step: usize,
}

impl ParseCtx {
    pub fn new() -> Self {
        Self { next_step: 1 }
    }
}

impl Default for ParseCtx {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Parser: Sized {
    const CHAR: char;
    fn parse<'a>(inp: &'a str, ctx: &ParseCtx) -> Result<(Self, &'a str)>;
}

pub trait Executor {
    /// Pipeline-level hook. If `Some`, the runner uses this result and skips
    /// the value-shape dispatch entirely. The cmd reads the prior step
    /// outputs from `StepStore`. Used by `$` (pipeline subst).
    fn apply_pipeline(&self, _store: &StepStore) -> Option<Result<Value>> {
        None
    }
    /// Resolve any `${N}` arg substitutions against the step store.
    /// Returns `Some(self_with_subs_replaced)` if the cmd had `${N}` args,
    /// `None` otherwise. Called before any `apply_*` dispatch.
    fn resolve(&self, _store: &StepStore) -> Result<Option<Self>>
    where
        Self: Sized,
    {
        Ok(None)
    }
    /// Whether this cmd advances the step counter. Defaults to `true`.
    /// `$` returns `false`: it replaces the current value but is not itself
    /// addressable.
    fn produces_step(&self) -> bool {
        true
    }
    /// Push every step index this cmd references into `out` (for the static
    /// pre-scan of which slots need storage). Default no-op.
    fn refs(&self, _out: &mut Vec<usize>) {}
    /// If `Some`, the framework uses this result and skips all shape dispatch +
    /// leaf-mapping. For cmds that want to see the whole Value verbatim
    /// (e.g. structural debug).
    fn apply(&self, _v: &Value) -> Option<Result<Value>> {
        None
    }
    /// Default `None` = cmd does not accept this shape. The framework then tries
    /// the other variants and/or promotes (String → List<char>) before giving up.
    /// `Some(Ok|Err)` = cmd accepts the shape and ran (successfully or not).
    fn apply_str(&self, _s: &str) -> Option<Result<Value>> {
        None
    }
    fn apply_list(&self, _items: &[Value]) -> Option<Result<Value>> {
        None
    }
    fn apply_int(&self, _n: i64) -> Option<Result<Value>> {
        None
    }
    fn apply_float(&self, _n: f64) -> Option<Result<Value>> {
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
            fn apply_pipeline(&self, store: &StepStore) -> Option<Result<Value>> {
                match self {
                    $( Command::$ty(inner) => inner.apply_pipeline(store), )+
                }
            }
            fn resolve(&self, store: &StepStore) -> Result<Option<Self>> {
                match self {
                    $( Command::$ty(inner) =>
                        Ok(inner.resolve(store)?.map(Command::$ty)), )+
                }
            }
            fn produces_step(&self) -> bool {
                match self {
                    $( Command::$ty(inner) => inner.produces_step(), )+
                }
            }
            fn refs(&self, out: &mut Vec<usize>) {
                match self {
                    $( Command::$ty(inner) => inner.refs(out), )+
                }
            }
            fn apply(&self, v: &Value) -> Option<Result<Value>> {
                match self {
                    $( Command::$ty(inner) => inner.apply(v), )+
                }
            }
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
            fn apply_int(&self, n: i64) -> Option<Result<Value>> {
                match self {
                    $( Command::$ty(inner) => inner.apply_int(n), )+
                }
            }
            fn apply_float(&self, n: f64) -> Option<Result<Value>> {
                match self {
                    $( Command::$ty(inner) => inner.apply_float(n), )+
                }
            }
        }

        pub fn parse_dispatch<'a>(ch: char, rest: &'a str, ctx: &ParseCtx)
            -> Option<Result<(Command, &'a str)>>
        {
            match ch {
                $( <$ty as Parser>::CHAR =>
                    Some(<$ty>::parse(rest, ctx).map(|(v, n)| (v.into(), n))), )+
                _ => None,
            }
        }
    };
}

cmds! { Split, Join, Number, Dbg, PipeRef }

pub fn apply_to_value(cmd: &Command, v: &Value) -> Result<Value> {
    if let Some(res) = cmd.apply(v) {
        return res;
    }
    match v {
        Value::String(s) => {
            if let Some(res) = cmd.apply_str(s) {
                return res;
            }
            // Promote String → List of single-char Strings, then retry as list.
            cmd.apply_list(&coerce::string_to_chars(s))
                .unwrap_or_else(|| Err("Cmd accepts neither String nor List input".into()))
        }
        Value::List(items) => {
            let all_leaves = items
                .iter()
                .all(|i| matches!(i, Value::String(_) | Value::Int(_) | Value::Float(_)));
            if all_leaves && let Some(res) = cmd.apply_list(items) {
                return res;
            }
            items
                .iter()
                .map(|item| apply_to_value(cmd, item))
                .collect::<Result<Vec<_>>>()
                .map(Value::List)
        }
        Value::Int(n) => {
            if let Some(res) = cmd.apply_int(*n) {
                return res;
            }
            if let Some(res) = cmd.apply_float(coerce::int_to_float(*n)) {
                return res;
            }
            if let Some(res) = cmd.apply_str(&coerce::int_to_str(*n)) {
                return res;
            }
            Err("Cmd does not accept Int input".into())
        }
        Value::Float(n) => {
            if let Some(res) = cmd.apply_float(*n) {
                return res;
            }
            if let Some(as_int) = coerce::float_to_int(*n)
                && let Some(res) = cmd.apply_int(as_int)
            {
                return res;
            }
            if let Some(res) = cmd.apply_str(&coerce::float_to_str(*n)) {
                return res;
            }
            Err("Cmd does not accept Float input".into())
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
                cmd.apply_list(&coerce::string_to_chars(s))
                    .unwrap_or_else(|| Err("neither".into()))
            }
            Value::List(items) => {
                let all_leaves = items
                    .iter()
                    .all(|i| matches!(i, Value::String(_) | Value::Int(_) | Value::Float(_)));
                if all_leaves && let Some(res) = cmd.apply_list(items) {
                    return res;
                }
                items
                    .iter()
                    .map(|item| drive(cmd, item))
                    .collect::<Result<Vec<_>>>()
                    .map(Value::List)
            }
            Value::Int(n) => {
                if let Some(res) = cmd.apply_int(*n) {
                    return res;
                }
                if let Some(res) = cmd.apply_float(coerce::int_to_float(*n)) {
                    return res;
                }
                if let Some(res) = cmd.apply_str(&coerce::int_to_str(*n)) {
                    return res;
                }
                Err("neither".into())
            }
            Value::Float(n) => {
                if let Some(res) = cmd.apply_float(*n) {
                    return res;
                }
                if let Some(as_int) = coerce::float_to_int(*n)
                    && let Some(res) = cmd.apply_int(as_int)
                {
                    return res;
                }
                if let Some(res) = cmd.apply_str(&coerce::float_to_str(*n)) {
                    return res;
                }
                Err("neither".into())
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

    // coercion ------------------------------------------------------------

    // Cmd that only accepts strings — used to test Int/Float → str coercion.
    struct StrOnly;
    impl Executor for StrOnly {
        fn apply_str(&self, s: &str) -> Option<Result<Value>> {
            Some(Ok(Value::String(s.to_owned())))
        }
    }

    // Cmd that only accepts floats — used to test Int → float widening.
    struct FloatOnly;
    impl Executor for FloatOnly {
        fn apply_float(&self, n: f64) -> Option<Result<Value>> {
            Some(Ok(Value::Float(n)))
        }
    }

    // Cmd that only accepts ints — used to test Float → int narrowing.
    struct IntOnly;
    impl Executor for IntOnly {
        fn apply_int(&self, n: i64) -> Option<Result<Value>> {
            Some(Ok(Value::Int(n)))
        }
    }

    // Cmd accepting both int and str — to test that non-integral Float
    // skips apply_int and falls through to apply_str.
    struct IntAndStr;
    impl Executor for IntAndStr {
        fn apply_int(&self, n: i64) -> Option<Result<Value>> {
            Some(Ok(Value::Int(n)))
        }
        fn apply_str(&self, s: &str) -> Option<Result<Value>> {
            Some(Ok(Value::String(s.to_owned())))
        }
    }

    #[test]
    fn coerce_int_to_str() {
        let out = drive(&StrOnly, &Value::Int(42)).unwrap();
        assert!(eq(&out, &s("42")));
    }

    #[test]
    fn coerce_float_to_str() {
        let out = drive(&StrOnly, &Value::Float(1.5)).unwrap();
        assert!(eq(&out, &s("1.5")));
    }

    #[test]
    fn coerce_int_to_float() {
        let out = drive(&FloatOnly, &Value::Int(7)).unwrap();
        assert!(matches!(out, Value::Float(x) if x == 7.0));
    }

    #[test]
    fn coerce_integral_float_to_int() {
        let out = drive(&IntOnly, &Value::Float(2.0)).unwrap();
        assert!(matches!(out, Value::Int(2)));
    }

    #[test]
    fn non_integral_float_to_int_only_errors() {
        // IntOnly has no apply_str fallback; 2.5 is non-integral → error.
        assert!(drive(&IntOnly, &Value::Float(2.5)).is_err());
    }

    #[test]
    fn non_integral_float_falls_through_to_str() {
        // IntAndStr accepts both; 2.5 is non-integral so apply_int is skipped,
        // apply_str gets "2.5".
        let out = drive(&IntAndStr, &Value::Float(2.5)).unwrap();
        assert!(eq(&out, &s("2.5")));
    }

    #[test]
    fn integral_float_prefers_int_over_str() {
        // IntAndStr: 2.0 is integral → apply_int wins.
        let out = drive(&IntAndStr, &Value::Float(2.0)).unwrap();
        assert!(matches!(out, Value::Int(2)));
    }

    #[test]
    fn huge_float_not_narrowed_to_int() {
        // 1e20 exceeds i64 range; roundtrip check rejects narrowing, falls
        // through to apply_str.
        let out = drive(&IntAndStr, &Value::Float(1e20)).unwrap();
        match out {
            Value::String(got) => assert_eq!(got, (1e20_f64).to_string()),
            _ => panic!("expected stringified float"),
        }
    }
}
