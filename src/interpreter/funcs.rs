use crate::error::*;
use crate::interpreter::types::*;

pub fn plus(input: &mut Input, stack: &mut Stack) -> StringResult<()> {
    if let Token::Num(val) = stack
        .pop_front()
        .into_string_result("Expected 1 value on the stack to call plus".to_string())?
    {
        *input = plus_impl(input, val)?;
    } else {
        return Err("Cannot call plus with non-number argument".to_string());
    }
    Ok(())
}

fn plus_impl(input: &Input, val: f64) -> StringResult<Input> {
    match input {
        Input::Num(n) => Ok(Input::Num(n + val)),
        Input::Str(s) => Ok(Input::Num(s.parse::<f64>().into_string_result_msg()? + val)),
        Input::List(l) => Ok(Input::List(
            l.iter()
                .map(|n| plus_impl(n, val))
                .collect::<StringResult<Vec<Input>>>()?,
        )),
    }
}
