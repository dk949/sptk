use crate::commands::{
    Executor, Parser, Result, Value,
    parser::{make_err, skip},
};

#[derive(Clone, Copy)]
enum Base {
    Float,
    Dec,
    Hex,
    Oct,
    Bin,
}

impl Base {
    fn radix(self) -> u32 {
        match self {
            Base::Float => 10,
            Base::Dec => 10,
            Base::Hex => 16,
            Base::Oct => 8,
            Base::Bin => 2,
        }
    }

    fn allow_sign(self) -> bool {
        matches!(self, Base::Float | Base::Dec)
    }
}

pub struct Number {
    base: Base,
    strict: bool,
}

impl Parser for Number {
    const CHAR: char = 'n';

    fn parse(inp: &str) -> Result<(Self, &str)> {
        let inp = skip(inp);
        let Some(c) = inp.chars().next() else {
            return make_err::<_, Number>(inp, "expected one of f/i/h/o/b (or F/I/H/O/B)");
        };
        let (base, strict) = match c {
            'f' => (Base::Float, true),
            'i' => (Base::Dec, true),
            'h' => (Base::Hex, true),
            'o' => (Base::Oct, true),
            'b' => (Base::Bin, true),
            'F' => (Base::Float, false),
            'I' => (Base::Dec, false),
            'H' => (Base::Hex, false),
            'O' => (Base::Oct, false),
            'B' => (Base::Bin, false),
            _ => return make_err::<_, Number>(inp, "expected one of f/i/h/o/b (or F/I/H/O/B)"),
        };
        Ok((Number { base, strict }, &inp[c.len_utf8()..]))
    }
}

impl Executor for Number {
    fn apply_str(&self, s: &str) -> Option<Result<Value>> {
        let res = match (self.base, self.strict) {
            (Base::Float, true) => parse_float_strict(s).map(Value::Float),
            (Base::Float, false) => Ok(Value::Float(parse_float_prefix(s))),
            (b, true) => parse_int_strict(s, b).map(Value::Int),
            (b, false) => Ok(Value::Int(parse_int_prefix(s, b))),
        };
        Some(res)
    }
}

fn strip_radix_prefix(s: &str, base: Base) -> &str {
    let prefixes: &[&str] = match base {
        Base::Hex => &["0x", "0X"],
        Base::Oct => &["0o", "0O"],
        Base::Bin => &["0b", "0B"],
        _ => return s,
    };
    for p in prefixes {
        if let Some(rest) = s.strip_prefix(p) {
            return rest;
        }
    }
    s
}

fn split_sign(s: &str, base: Base) -> (bool, &str) {
    if !base.allow_sign() {
        return (false, s);
    }
    match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    }
}

fn parse_int_strict(s: &str, base: Base) -> Result<i64> {
    let (neg, after_sign) = split_sign(s, base);
    let body = strip_radix_prefix(after_sign, base);
    if body.is_empty() {
        return Err(format!("n: empty number for base {}", base.radix()));
    }
    if body.starts_with('-') || body.starts_with('+') {
        return Err(format!("n: sign not allowed for base {}", base.radix()));
    }
    let n = i64::from_str_radix(body, base.radix()).map_err(|e| format!("n: {e}"))?;
    if neg {
        n.checked_neg()
            .ok_or_else(|| "n: integer overflow on negation".into())
    } else {
        Ok(n)
    }
}

fn parse_int_prefix(s: &str, base: Base) -> i64 {
    let (neg, after_sign) = split_sign(s, base);
    let body = strip_radix_prefix(after_sign, base);
    let radix = base.radix();
    let mut end = 0;
    for (i, c) in body.char_indices() {
        if c.is_digit(radix) {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return 0;
    }
    let n = i64::from_str_radix(&body[..end], radix).unwrap_or(0);
    if neg { n.checked_neg().unwrap_or(0) } else { n }
}

fn parse_float_strict(s: &str) -> Result<f64> {
    s.parse::<f64>().map_err(|e| format!("n f: {e}"))
}

fn parse_float_prefix(s: &str) -> f64 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut saw_digit = false;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        saw_digit = true;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            saw_digit = true;
        }
    }
    if saw_digit && i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut j = i + 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        let exp_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_start {
            i = j;
        }
    }
    if !saw_digit {
        return 0.0;
    }
    s[..i].parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(prog: &str, s: &str) -> Result<Value> {
        let (n, rest) = Number::parse(prog)?;
        assert_eq!(rest, "", "unexpected trailing parser input");
        n.apply_str(s).unwrap()
    }

    fn as_int(v: Value) -> i64 {
        match v {
            Value::Int(n) => n,
            _ => panic!("expected Int"),
        }
    }
    fn as_float(v: Value) -> f64 {
        match v {
            Value::Float(n) => n,
            _ => panic!("expected Float"),
        }
    }

    // parser --------------------------------------------------------------

    #[test]
    fn parse_eof() {
        assert!(Number::parse("").is_err());
    }

    #[test]
    fn parse_bad_letter() {
        assert!(Number::parse("z").is_err());
    }

    #[test]
    fn parse_consumes_one_char() {
        let (_, rest) = Number::parse("ftrailing").unwrap();
        assert_eq!(rest, "trailing");
    }

    // strict int ----------------------------------------------------------

    #[test]
    fn i_dec() {
        assert_eq!(as_int(run("i", "123").unwrap()), 123);
        assert_eq!(as_int(run("i", "-7").unwrap()), -7);
        assert_eq!(as_int(run("i", "+7").unwrap()), 7);
        assert!(run("i", "12x").is_err());
        assert!(run("i", "").is_err());
    }

    #[test]
    fn h_hex() {
        assert_eq!(as_int(run("h", "ff").unwrap()), 0xff);
        assert_eq!(as_int(run("h", "0xFF").unwrap()), 0xff);
        assert_eq!(as_int(run("h", "0XFF").unwrap()), 0xff);
        assert!(run("h", "-ff").is_err(), "sign not allowed for h");
        assert!(run("h", "gg").is_err());
    }

    #[test]
    fn o_oct() {
        assert_eq!(as_int(run("o", "17").unwrap()), 0o17);
        assert_eq!(as_int(run("o", "0o17").unwrap()), 0o17);
        assert!(run("o", "8").is_err());
    }

    #[test]
    fn b_bin() {
        assert_eq!(as_int(run("b", "101").unwrap()), 0b101);
        assert_eq!(as_int(run("b", "0b101").unwrap()), 0b101);
        assert!(run("b", "102").is_err());
    }

    // partial int ---------------------------------------------------------

    #[test]
    fn cap_i_dec_partial() {
        assert_eq!(as_int(run("I", "12abc").unwrap()), 12);
        assert_eq!(as_int(run("I", "-7xyz").unwrap()), -7);
        assert_eq!(as_int(run("I", "abc").unwrap()), 0);
    }

    #[test]
    fn cap_h_hex_partial() {
        assert_eq!(as_int(run("H", "ffzz").unwrap()), 0xff);
        assert_eq!(as_int(run("H", "0xdeadXXX").unwrap()), 0xdead);
        assert_eq!(as_int(run("H", "zzz").unwrap()), 0);
    }

    #[test]
    fn cap_o_oct_partial() {
        assert_eq!(as_int(run("O", "178").unwrap()), 0o17);
        assert_eq!(as_int(run("O", "xx").unwrap()), 0);
    }

    #[test]
    fn cap_b_bin_partial() {
        assert_eq!(as_int(run("B", "1012").unwrap()), 0b101);
        assert_eq!(as_int(run("B", "abc").unwrap()), 0);
    }

    // float ---------------------------------------------------------------

    #[test]
    fn f_float_strict() {
        assert_eq!(as_float(run("f", "1.5").unwrap()), 1.5);
        assert_eq!(as_float(run("f", "-2e3").unwrap()), -2000.0);
        assert!(run("f", "1.5x").is_err());
    }

    #[test]
    fn cap_f_float_partial() {
        assert_eq!(as_float(run("F", "1.5x").unwrap()), 1.5);
        assert_eq!(as_float(run("F", "-2e3suffix").unwrap()), -2000.0);
        assert_eq!(as_float(run("F", "abc").unwrap()), 0.0);
        // exponent w/o digits: must not consume the `e`.
        assert_eq!(as_float(run("F", "1.5e").unwrap()), 1.5);
    }
}
