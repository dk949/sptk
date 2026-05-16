use regex::Regex;

use crate::commands::{Result, Value, coerce, store::StepStore};

/// A cmd argument slot that's either a literal value (parsed inline) or a
/// reference to the output of a prior pipeline step (`${N}`).
#[derive(Clone)]
pub enum Arg<T> {
    Lit(T),
    Sub(usize),
}

impl<T> Arg<T> {
    pub fn add_refs(&self, out: &mut Vec<usize>) {
        if let Arg::Sub(n) = self {
            out.push(*n);
        }
    }
}

impl<T: Clone + FromValue> Arg<T> {
    /// Produce the concrete value: clone the literal, or look up the
    /// referenced step's output and coerce.
    pub fn resolve(&self, store: &StepStore) -> Result<T> {
        match self {
            Arg::Lit(v) => Ok(v.clone()),
            Arg::Sub(n) => T::from_value(store.get(*n)?),
        }
    }
}

/// Coercion from a stored Value to a concrete arg type.
pub trait FromValue: Sized {
    fn from_value(v: &Value) -> Result<Self>;
}

impl FromValue for String {
    fn from_value(v: &Value) -> Result<Self> {
        coerce::to_string(v)
    }
}

impl FromValue for i64 {
    fn from_value(v: &Value) -> Result<Self> {
        coerce::to_int(v)
    }
}

impl FromValue for f64 {
    fn from_value(v: &Value) -> Result<Self> {
        coerce::to_float(v)
    }
}

impl FromValue for Regex {
    fn from_value(v: &Value) -> Result<Self> {
        coerce::to_regex(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_lit() {
        let a: Arg<String> = Arg::Lit("hi".into());
        let store = StepStore::new(0);
        assert_eq!(a.resolve(&store).unwrap(), "hi");
    }

    #[test]
    fn resolve_sub_string() {
        let mut s = StepStore::new(2);
        s.set(1, Value::String("hi".into()));
        let a: Arg<String> = Arg::Sub(1);
        assert_eq!(a.resolve(&s).unwrap(), "hi");
    }

    #[test]
    fn resolve_sub_int_to_string() {
        let mut s = StepStore::new(2);
        s.set(1, Value::Int(42));
        let a: Arg<String> = Arg::Sub(1);
        assert_eq!(a.resolve(&s).unwrap(), "42");
    }

    #[test]
    fn resolve_sub_list_to_string_errors() {
        let mut s = StepStore::new(2);
        s.set(1, Value::List(vec![]));
        let a: Arg<String> = Arg::Sub(1);
        assert!(a.resolve(&s).is_err());
    }

    #[test]
    fn add_refs_picks_subs() {
        let mut v = Vec::new();
        let a: Arg<String> = Arg::Lit("x".into());
        a.add_refs(&mut v);
        assert!(v.is_empty());
        let b: Arg<String> = Arg::Sub(7);
        b.add_refs(&mut v);
        assert_eq!(v, vec![7]);
    }
}
