use super::*;

type TestInput<T> = (&'static str, &'static [u8], bool, T, Option<&'static u8>);
fn run<
    T: fmt::Debug + std::cmp::PartialEq,
    F: FnOnce(&mut iter::Peekable<slice::Iter<u8>>) -> StringResult<T>,
>(
    func: F,
    (name, input, is_ok, expected, remaining): TestInput<T>,
) {
    let mut iter = input.iter().peekable();
    let res = func(&mut iter);
    assert_eq!(
        res.is_ok(),
        is_ok,
        "is_ok, test: {}, err: {}",
        name,
        res.unwrap_err()
    );
    if !is_ok {
        return;
    }
    assert_eq!(res.unwrap(), expected);
    assert_eq!(iter.next(), remaining, "remaining, test: {}", name);
}
#[test]
fn get_number_test() {
    #[cfg_attr(rustfmt, rustfmt_skip)]
        for test in [
            ("1234"     , "1234".as_bytes()     , true  , 1234.0 , None)               ,
            ("12.34"    , "12.34".as_bytes()    , true  , 12.340 , None)               ,
            (""         , "".as_bytes()         , false , 0.0    , None)               ,
            ("1234."    , "1234.".as_bytes()    , true  , 1234.0 , None)               ,
            ("1234abc"  , "1234abc".as_bytes()  , true  , 1234.0 , Some(&('a' as u8))) ,
            ("abc1234"  , "abc1234".as_bytes()  , false , 0.0    , Some(&('a' as u8))) ,
            ("12.34abc" , "12.34abc".as_bytes() , true  , 12.340 , Some(&('a' as u8))) ,
            ("abc12.34" , "abc12.34".as_bytes() , false , 0.0    , Some(&('a' as u8))) ,

        ] {
            run(get_number, test);
        }
}

#[test]
fn get_string_test() {
    #[cfg_attr(rustfmt, rustfmt_skip)]
        for test in [
            ("\"hello\""  , "\"hello\""  .as_bytes() , true  , "hello".to_string() , None)               ,
            ("\"\""       , "\"\""       .as_bytes() , true  , "".to_string()      , None)               ,
            ("\"hello"    , "\"hello"    .as_bytes() , false , "".to_string()      , None)               ,
            (""           , ""           .as_bytes() , false , "".to_string()      , None)               ,
            ("\"hello\"a" , "\"hello\"a" .as_bytes() , true  , "hello".to_string() , Some(&('a' as u8))) ,
            ("\"\"a"      , "\"\"a"      .as_bytes() , true  , "".to_string()      , Some(&('a' as u8))) ,
        ] {
            run(get_string, test);
        }
}
