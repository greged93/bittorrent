use serde_json::{Number, Value};
use std::str::FromStr;

#[macro_export]
macro_rules! value {
    ($input: expr) => {{
        let val = stringify!($input);
        if val.contains('"') {
            let input = val.replace('"', "");
            serde_json::Value::String(input.into())
        } else {
            use std::str::FromStr;
            serde_json::Value::Number(serde_json::Number::from_str(val).unwrap())
        }
    }};
}

// TODO: add miette here instead of Option
/// BenDecode the provided str and advance by the amount of
/// tokens found.
pub fn decode(input: &mut &str) -> Option<Value> {
    let mut chars = input.chars().peekable();
    let peeked = chars.peek()?;
    match peeked {
        'l' => {
            // Start recursion
            *input = &input[1..];
            let mut values = vec![];
            while let Some(decoded) = decode(input) {
                values.push(decoded);
            }
            let arr = Value::Array(values);
            Some(arr)
        }
        'i' => {
            let delimiter_pos = input.find('e')?;
            let string = &input[1..delimiter_pos];
            *input = &input[delimiter_pos + 1..];
            Some(Value::Number(Number::from_str(string).ok()?))
        }
        c if c.is_ascii_digit() => {
            let delimiter_pos = input.find(':')?;
            let string_len = input[..delimiter_pos].parse::<usize>().ok()?;
            let string = &input[delimiter_pos + 1..delimiter_pos + 1 + string_len];

            *input = &input[delimiter_pos + 1 + string_len..];

            Some(Value::String(string.to_string()))
        }
        'e' => {
            // Skip the 'e'
            *input = &input[1..];
            eprintln!("{}", input);
            None
        }
        _ => unimplemented!("not implemented yet"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        // Test str
        let mut input = "5:hello";

        // Start the decoder
        let value = decode(&mut input).unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, value!("hello"));
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_number() {
        // Test str
        let mut input = "i563e";

        // Start the decoder
        let value = decode(&mut input).unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, value!(563));
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_empty_list() {
        // Test str
        let mut input = "le";

        // Start the decoder
        let value = decode(&mut input).unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, Value::Array(vec![]));
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_list_simple() {
        // Test str
        let mut input = "l5:helloi52ee";

        // Start the decoder
        let value = decode(&mut input).unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, Value::Array(vec![value!("hello"), value!(52)]));
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_list_complex() {
        // Test str
        let mut input = "lli4eei5ee";

        // Start the decoder
        let value = decode(&mut input).unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(
            value,
            Value::Array(vec![Value::Array(vec![value!(4)]), value!(5)])
        );
        assert_eq!(input, "");
    }
}
