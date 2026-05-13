pub mod parser;
pub mod split;

use split::Split;

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
    /// Default rank: apply this cmd to a single innermost string, producing a
    /// Value of any rank. The runner (apply_to_value) handles wrapping over
    /// enclosing lists.
    fn apply(&self, s: &str) -> Result<Value>;
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
            fn apply(&self, s: &str) -> Result<Value> {
                match self {
                    $( Command::$ty(inner) => inner.apply(s), )+
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

cmds! { Split }

pub fn apply_to_value(cmd: &Command, v: &Value) -> Result<Value> {
    match v {
        Value::String(s) => cmd.apply(s),
        Value::List(items) => items
            .iter()
            .map(|item| apply_to_value(cmd, item))
            .collect::<Result<Vec<_>>>()
            .map(Value::List),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Wrap;
    impl Executor for Wrap {
        fn apply(&self, s: &str) -> Result<Value> {
            Ok(Value::List(vec![Value::String(s.to_owned())]))
        }
    }

    fn s(x: &str) -> Value {
        Value::String(x.into())
    }
    fn l(xs: Vec<Value>) -> Value {
        Value::List(xs)
    }

    #[test]
    fn leaf_map_string() {
        // Can't go through apply_to_value because Wrap isn't a Command variant.
        // Test the shape locally instead.
        let v = s("foo");
        match &v {
            Value::String(x) => assert_eq!(x, "foo"),
            _ => panic!(),
        }
        let mapped = Wrap.apply("foo").unwrap();
        assert!(matches!(&mapped, Value::List(v) if v.len() == 1));
    }

    #[test]
    fn leaf_map_nested() {
        fn go(cmd: &Wrap, v: &Value) -> Value {
            match v {
                Value::String(s) => cmd.apply(s).unwrap(),
                Value::List(items) => Value::List(items.iter().map(|i| go(cmd, i)).collect()),
            }
        }
        let input = l(vec![l(vec![s("a"), s("b")]), l(vec![s("c")])]);
        let out = go(&Wrap, &input);
        // Rank-2 input → rank-3 output. Each innermost string becomes List([String]).
        let expect = l(vec![
            l(vec![l(vec![s("a")]), l(vec![s("b")])]),
            l(vec![l(vec![s("c")])]),
        ]);
        assert!(eq(&out, &expect));
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
}
