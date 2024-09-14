use miette::{miette, LabeledSpan, Result};
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

pub struct Decoder<'a> {
    full: &'a str,
    cursor: &'a str,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            full: input,
            cursor: input,
        }
    }

    // TODO: add miette here instead of Option
    /// BenDecode the provided str and advance by the amount of
    /// tokens found.
    pub fn decode(&mut self) -> Result<Value> {
        let mut chars = self.cursor.chars().peekable();
        let peeked = chars.peek().ok_or(miette!("EOF"))?;
        match peeked {
            'l' => {
                // Start recursion
                self.cursor = &self.cursor[1..];
                let mut values = vec![];
                while let Ok(decoded) = self.decode() {
                    values.push(decoded);
                }
                if !self.cursor.starts_with('e') {
                    return Err(miette!(
                        labels = vec![LabeledSpan::at_offset(
                            self.full.len() - self.cursor.len() - 1,
                            "here"
                        )],
                        "expected closing e",
                    )
                    .with_source_code(self.full.to_string()));
                }
                self.cursor = &self.cursor[1..];
                let arr = Value::Array(values);
                Ok(arr)
            }
            'i' => {
                let delimiter_pos = self.cursor.find('e').ok_or(
                    miette!(
                        labels = vec![LabeledSpan::at_offset(
                            self.full.len() - self.cursor.len(),
                            "here"
                        )],
                        "expected closing e",
                    )
                    .with_source_code(self.full.to_string()),
                )?;
                let string = &self.cursor[1..delimiter_pos];
                self.cursor = &self.cursor[delimiter_pos + 1..];
                Ok(Value::Number(
                    Number::from_str(string).map_err(|_| miette!("cannot convert to number"))?,
                ))
            }
            c if c.is_ascii_digit() => {
                let delimiter_pos = self.cursor.find(':').ok_or(
                    miette!(
                        labels = vec![LabeledSpan::at_offset(
                            self.full.len() - self.cursor.len(),
                            "here"
                        )],
                        "expected closing :",
                    )
                    .with_source_code(self.full.to_string()),
                )?;
                let string_len = self.cursor[..delimiter_pos]
                    .parse::<usize>()
                    .map_err(|_| miette!("cannot convert to string"))?;
                let string = &self.cursor[delimiter_pos + 1..delimiter_pos + 1 + string_len];

                self.cursor = &self.cursor[delimiter_pos + 1 + string_len..];

                Ok(Value::String(string.to_string()))
            }
            'e' => {
                // This is a terminating char
                Err(miette!("terminator"))
            }
            _ => unimplemented!("not implemented yet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        // Test str
        let mut decoder = Decoder::new("5:hello");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, value!("hello"));
        assert_eq!(decoder.cursor, "");
    }

    #[test]
    fn test_parse_number() {
        // Test str
        let mut decoder = Decoder::new("i563e");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, value!(563));
        assert_eq!(decoder.cursor, "");
    }

    #[test]
    fn test_parse_empty_list() {
        // Test str
        let mut decoder = Decoder::new("le");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, Value::Array(vec![]));
        assert_eq!(decoder.cursor, "");
    }

    #[test]
    fn test_parse_list_simple() {
        // Test str
        let mut decoder = Decoder::new("l5:helloi52ee");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, Value::Array(vec![value!("hello"), value!(52)]));
        assert_eq!(decoder.cursor, "");
    }

    #[test]
    fn test_parse_list_complex() {
        // Test str
        let mut decoder = Decoder::new("lli4eei5ee");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(
            value,
            Value::Array(vec![Value::Array(vec![value!(4)]), value!(5)])
        );
        assert_eq!(decoder.cursor, "");
    }
}
