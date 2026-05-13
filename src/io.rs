use std::{
    fs,
    io::{self, Read, Write},
};

pub fn read_file(path: &str) -> io::Result<String> {
    if path == "-" {
        let mut stdin = io::stdin();
        let mut out = String::new();
        stdin.read_to_string(&mut out)?;
        Ok(out)
    } else {
        fs::read_to_string(path)
    }
}

pub fn write_file(path: &str, data: &str) -> io::Result<()> {
    if path == "-" {
        let mut stdout = io::stdout();
        stdout.write_all(data.as_bytes())
    } else {
        fs::write(path, data)
    }
}
