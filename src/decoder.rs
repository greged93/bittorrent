pub struct BenDecoder<'a> {
    input: &'a str,
}

impl<'a> BenDecoder<'a> {
    /// Returns a new [`BenDecoder`] which hold a reference
    /// to the input str.
    pub fn new(decodable: &'a str) -> Self {
        Self { input: decodable }
    }
}

impl<'a> Iterator for BenDecoder<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.input.chars().peekable();
        let peeked = chars.peek()?;
        match peeked {
            'i' => {
                let delimiter_pos = self.input.find('e')?;
                let string = &self.input[1..delimiter_pos];
                self.input = &self.input[delimiter_pos + 1..];
                Some(string)
            }
            c if c.is_ascii_digit() => {
                let delimiter_pos = self.input.find(':')?;
                let string_len = self.input[..delimiter_pos].parse::<usize>().ok()?;
                let string = &self.input[delimiter_pos + 1..delimiter_pos + 1 + string_len];

                self.input = &self.input[delimiter_pos + 1 + string_len..];

                Some(string)
            }
            _ => unimplemented!("not implemented yet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::decoder::BenDecoder;

    #[test]
    fn test_parse_string() {
        // Test str
        let input = "5:hello";

        // Start the decoder
        let mut decoder = BenDecoder::new(input);

        // Advance the decoder
        let result = decoder.next();

        // Check the result and the str left in the decoder
        assert_eq!(result, Some("hello"));
        assert_eq!(decoder.input, "");
    }

    #[test]
    fn test_parse_number() {
        // Test str
        let input = "i563e";

        // Start the decoder
        let mut decoder = BenDecoder::new(input);

        // Advance the decoder
        let result = decoder.next();

        // Check the result and the str left in the decoder
        assert_eq!(result, Some("563"));
        assert_eq!(decoder.input, "");
    }
}
