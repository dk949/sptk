mod args;
mod common;
mod error;
mod file_io;
mod interpreter;

fn main() -> error::ExitResult {
    args::get_input_and_program()
        .and_then(interpreter::run)
        .map_err(String::into)
}
