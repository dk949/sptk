use crate::error::*;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::*;

pub fn read_to_string<P: AsRef<Path>>(path: P) -> StringResult<String> {
    fs::read_to_string(path).into_string_result_msg()
}

pub fn get_input() -> StringResult<String> {
    let mut buf = vec![];
    io::stdin().read_to_end(&mut buf).into_string_result_msg()?;
    String::from_utf8(buf).into_string_result_msg()
}
