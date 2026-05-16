use regex::Regex;

use crate::commands::{Result, Value};

// Primitive coercions: shared between `apply_to_value` (auto-coerce on
// shape-dispatch fallthrough) and `Arg::resolve` (value-level coercion at
// runtime substitution).

pub fn int_to_float(n: i64) -> f64 {
    n as f64
}

pub fn int_to_str(n: i64) -> String {
    n.to_string()
}

pub fn float_to_str(n: f64) -> String {
    n.to_string()
}

/// `None` if `n` is non-integral or out of i64 range.
pub fn float_to_int(n: f64) -> Option<i64> {
    let as_int = n as i64;
    if (as_int as f64) == n {
        Some(as_int)
    } else {
        None
    }
}

/// Promote a String to a List of single-char Strings.
pub fn string_to_chars(s: &str) -> Vec<Value> {
    s.chars().map(|c| Value::String(c.to_string())).collect()
}

// Value-level helpers: used by `${N}` substitution at resolve time. Rules
// (per spec): int/float -> string; int -> float; integral float -> int.
// No list-to-string, no string-to-numeric.

pub fn to_string(v: &Value) -> Result<String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Int(n) => Ok(int_to_str(*n)),
        Value::Float(n) => Ok(float_to_str(*n)),
        Value::List(_) => Err("cannot coerce List to String".into()),
    }
}

pub fn to_int(v: &Value) -> Result<i64> {
    match v {
        Value::Int(n) => Ok(*n),
        Value::Float(n) => {
            float_to_int(*n).ok_or_else(|| format!("cannot coerce non-integral Float {n} to Int"))
        }
        Value::String(_) => Err("cannot coerce String to Int".into()),
        Value::List(_) => Err("cannot coerce List to Int".into()),
    }
}

pub fn to_float(v: &Value) -> Result<f64> {
    match v {
        Value::Float(n) => Ok(*n),
        Value::Int(n) => Ok(int_to_float(*n)),
        Value::String(_) => Err("cannot coerce String to Float".into()),
        Value::List(_) => Err("cannot coerce List to Float".into()),
    }
}

pub fn to_regex(v: &Value) -> Result<Regex> {
    match v {
        Value::String(s) => Regex::new(s).map_err(|e| e.to_string()),
        Value::Int(_) => Err("cannot coerce Int to Regex".into()),
        Value::Float(_) => Err("cannot coerce Float to Regex".into()),
        Value::List(_) => Err("cannot coerce List to Regex".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // primitives ----------------------------------------------------------

    #[test]
    fn float_to_int_integral() {
        assert_eq!(float_to_int(2.0), Some(2));
        assert_eq!(float_to_int(-7.0), Some(-7));
        assert_eq!(float_to_int(0.0), Some(0));
    }

    #[test]
    fn float_to_int_non_integral() {
        assert_eq!(float_to_int(2.5), None);
        assert_eq!(float_to_int(-0.1), None);
    }

    #[test]
    fn float_to_int_huge() {
        // 1e20 exceeds i64 range; cast saturates, roundtrip fails.
        assert_eq!(float_to_int(1e20), None);
    }

    #[test]
    fn string_to_chars_basic() {
        let out = string_to_chars("ab");
        assert_eq!(out.len(), 2);
        assert!(matches!(&out[0], Value::String(s) if s == "a"));
        assert!(matches!(&out[1], Value::String(s) if s == "b"));
    }

    // to_string -----------------------------------------------------------

    #[test]
    fn to_string_from_string() {
        assert_eq!(to_string(&Value::String("hi".into())).unwrap(), "hi");
    }

    #[test]
    fn to_string_from_int() {
        assert_eq!(to_string(&Value::Int(42)).unwrap(), "42");
    }

    #[test]
    fn to_string_from_float() {
        assert_eq!(to_string(&Value::Float(1.5)).unwrap(), "1.5");
    }

    #[test]
    fn to_string_rejects_list() {
        assert!(to_string(&Value::List(vec![])).is_err());
    }

    // to_int / to_float ---------------------------------------------------

    #[test]
    fn to_int_from_int() {
        assert_eq!(to_int(&Value::Int(7)).unwrap(), 7);
    }

    #[test]
    fn to_int_from_integral_float() {
        assert_eq!(to_int(&Value::Float(2.0)).unwrap(), 2);
    }

    #[test]
    fn to_int_rejects_non_integral_float() {
        assert!(to_int(&Value::Float(2.5)).is_err());
    }

    #[test]
    fn to_int_rejects_string() {
        assert!(to_int(&Value::String("3".into())).is_err());
    }

    #[test]
    fn to_float_from_int() {
        assert_eq!(to_float(&Value::Int(7)).unwrap(), 7.0);
    }

    #[test]
    fn to_float_rejects_string() {
        assert!(to_float(&Value::String("1.5".into())).is_err());
    }

    // to_regex ------------------------------------------------------------

    #[test]
    fn to_regex_from_string() {
        let r = to_regex(&Value::String("a+".into())).unwrap();
        assert!(r.is_match("aaa"));
    }

    #[test]
    fn to_regex_invalid_string() {
        assert!(to_regex(&Value::String("[".into())).is_err());
    }

    #[test]
    fn to_regex_rejects_non_string() {
        assert!(to_regex(&Value::Int(1)).is_err());
        assert!(to_regex(&Value::List(vec![])).is_err());
    }
}
