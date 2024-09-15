use miette::{miette, LabeledSpan, Result};
use serde_json::{Map, Number, Value};
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
    full: &'a [u8],
    cursor: &'a [u8],
}

impl<'a> Decoder<'a> {
    /// Returns an [`Decoder`]
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            full: input,
            cursor: input,
        }
    }

    /// BenDecode the provided str and advance by the amount of
    /// tokens found.
    pub fn decode(&mut self) -> Result<Value> {
        let peeked = self.cursor.first();
        match peeked {
            // Parsed the start of a map
            Some(b'd') => {
                self.advance_one();
                let mut map = Map::new();
                while let Ok(key) = self.decode() {
                    let value = self.decode()?;
                    let key = key
                        .as_str()
                        .ok_or(miette!("expected string as key"))?
                        .to_string();
                    map.insert(key, value);
                }
                self.assert_next_terminator()?;
                self.advance_one();
                Ok(Value::Object(map))
            }
            // Parsed the start of a list
            Some(b'l') => {
                self.advance_one();
                let mut values = vec![];
                while let Ok(decoded) = self.decode() {
                    values.push(decoded);
                }
                self.assert_next_terminator()?;
                self.advance_one();
                let arr = Value::Array(values);
                Ok(arr)
            }
            // Parse the start of a number
            Some(b'i') => {
                let delimiter_pos = self.cursor.iter().position(|x| x == &b'e').ok_or(
                    miette!(
                        labels = vec![LabeledSpan::at_offset(
                            self.full.len() - self.cursor.len(),
                            "here"
                        )],
                        "expected closing e",
                    )
                    .with_source_code(self.full.to_vec()),
                )?;
                let string = &self.cursor[1..delimiter_pos];
                self.advance_n(delimiter_pos + 1);
                Ok(Value::Number(
                    Number::from_str(&String::from_utf8_lossy(string))
                        .map_err(|_| miette!("cannot convert to number"))?,
                ))
            }
            // Parsed the start of a string
            Some(c) if c.is_ascii_digit() => {
                let delimiter_pos = self.cursor.iter().position(|x| x == &b':').ok_or(
                    miette!(
                        labels = vec![LabeledSpan::at_offset(
                            self.full.len() - self.cursor.len(),
                            "here"
                        )],
                        "expected : delimiter",
                    )
                    .with_source_code(self.full.to_vec()),
                )?;
                let str = std::str::from_utf8(&self.cursor[..delimiter_pos])
                    .map_err(|_| miette!("invalid str"))?;
                let string_len = str
                    .parse::<usize>()
                    .map_err(|_| miette!("cannot convert to string"))?;
                let bytes = &self.cursor[delimiter_pos + 1..delimiter_pos + 1 + string_len];
                self.advance_n(delimiter_pos + 1 + string_len);
                if let Ok(s) = std::str::from_utf8(bytes) {
                    Ok(Value::String(s.to_string()))
                } else {
                    // If the string is not a valid utf8 string, we read it as a hex string
                    let s = hex::encode(bytes);
                    Ok(Value::String(s))
                }
            }
            // Parsed a terminator
            Some(b'e') => {
                // This is a terminating char
                Err(miette!("terminator"))
            }
            _ => Err(miette!("unhandled char")),
        }
    }

    /// Returns an error if the char at the cursor isn't an 'e'.
    fn assert_next_terminator(&mut self) -> Result<()> {
        if !self.cursor.starts_with(b"e") {
            return Err(miette!(
                labels = vec![LabeledSpan::at_offset(
                    self.full.len() - self.cursor.len() - 1,
                    "here"
                )],
                "expected closing e",
            )
            .with_source_code(self.full.to_vec()));
        }
        Ok(())
    }

    /// Advance the cursor by one.
    fn advance_one(&mut self) {
        self.cursor = &self.cursor[1..];
    }

    /// Advance the cursor by the provided n value.
    fn advance_n(&mut self, n: usize) {
        self.cursor = &self.cursor[n..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        // Test str
        let input = b"5:hello";
        let mut decoder = Decoder::new(input);

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, value!("hello"));
        assert_eq!(decoder.cursor, b"");
    }

    #[test]
    fn test_parse_number() {
        // Test str
        let mut decoder = Decoder::new(b"i563e");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, value!(563));
        assert_eq!(decoder.cursor, b"");
    }

    #[test]
    fn test_parse_empty_list() {
        // Test str
        let mut decoder = Decoder::new(b"le");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, Value::Array(vec![]));
        assert_eq!(decoder.cursor, b"");
    }

    #[test]
    fn test_parse_list_simple() {
        // Test str
        let mut decoder = Decoder::new(b"l5:helloi52ee");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(value, Value::Array(vec![value!("hello"), value!(52)]));
        assert_eq!(decoder.cursor, b"");
    }

    #[test]
    fn test_parse_list_complex() {
        // Test str
        let mut decoder = Decoder::new(b"lli4eei5ee");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(
            value,
            Value::Array(vec![Value::Array(vec![value!(4)]), value!(5)])
        );
        assert_eq!(decoder.cursor, b"");
    }

    #[test]
    fn test_parse_map_simple() {
        // Test str
        let mut decoder = Decoder::new(b"d3:foo3:bar5:helloi52ee");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(
            value,
            Value::Object(Map::from_iter(
                [("foo".into(), value!("bar")), ("hello".into(), value!(52))].into_iter()
            ))
        );
        assert_eq!(decoder.cursor, b"");
    }

    #[test]
    fn test_parse_map_complex() {
        // Test str
        let mut decoder =
            Decoder::new(b"d6:lengthi92063e4:name10:sample.txt12:piece lengthi32768e6:pieces1:ae");

        // Start the decoder
        let value = decoder.decode().unwrap();

        // Check the result and the str left in the decoder
        assert_eq!(
            value,
            Value::Object(Map::from_iter(
                [
                    ("length".into(), value!(92063)),
                    ("name".into(), value!("sample.txt")),
                    ("piece length".into(), value!(32768)),
                    ("pieces".into(), value!("a")),
                ]
                .into_iter()
            ))
        );
        assert_eq!(decoder.cursor, b"");
    }
}
