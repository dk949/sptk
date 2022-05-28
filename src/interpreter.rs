use std::io;




pub fn run((input, program): (String, String)) -> io::Result<()> {
    println!("input = {}", input);
    println!("program = {}", program);
    Ok(())
}
