use std::convert;
use std::fmt;

pub type StringResult<T> = Result<T, String>;

pub trait IntoStringResult<T> {
    fn into_string_result(self, msg: String) -> StringResult<T>;
}

pub trait IntoStringResultErr<T, E> {
    fn into_string_result_err<F>(self, f: F) -> StringResult<T>
    where
        F: FnOnce(E) -> String;
}

pub trait IntoStringResultMsg<T> {
    fn into_string_result_msg(self) -> StringResult<T>;
}

impl<T, E> IntoStringResult<T> for Result<T, E> {
    fn into_string_result(self, msg: String) -> StringResult<T> {
        self.or_else(|_| Err(msg))
    }
}

impl<T> IntoStringResult<T> for Option<T> {
    fn into_string_result(self, msg: String) -> StringResult<T> {
        self.ok_or_else(|| msg)
    }
}

impl<T, E> IntoStringResultErr<T, E> for Result<T, E> {
    fn into_string_result_err<F>(self, f: F) -> StringResult<T>
    where
        F: FnOnce(E) -> String,
    {
        self.map_err(|e| f(e))
    }
}

impl<T, E: std::fmt::Display> IntoStringResultMsg<T> for Result<T, E> {
    fn into_string_result_msg(self) -> StringResult<T> {
        self.map_err(|e| e.to_string())
    }
}

pub struct ExitError {
    msg: String,
}

pub type ExitResult = Result<(), ExitError>;

impl fmt::Debug for ExitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<String> for ExitError {
    fn from(s: String) -> Self {
        ExitError { msg: s }
    }
}
