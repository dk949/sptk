mod args;
mod interpreter;
use std::io;

fn main() -> io::Result<()> {
    interpreter::run(args::input_and_program()?)
}
