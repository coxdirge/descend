#[derive(Debug, PartialEq)]
pub enum Token {
    Number(i32),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
}

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar(u8),
    IntegerOverflow,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.peek()?;

        self.pos += 1;
        Some(byte)
    }

    fn read_number(&mut self) -> Result<i32, LexError> {
        let mut res = 0_i32;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                let digit = (ch - b'0') as i32;

                res = res.checked_mul(10).ok_or(LexError::IntegerOverflow)?;
                res = res.checked_add(digit).ok_or(LexError::IntegerOverflow)?;

                self.advance();
            } else {
                break;
            }
        }

        Ok(res)
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        // 第一步：跳过空白字符
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n')) {
            self.advance();
        }

        // 第二步：识别 Token
        let token = match self.peek() {
            None => return Ok(None),

            Some(b'0'..=b'9') => {
                let num = self.read_number()?;
                return Ok(Some(Token::Number(num)));
            }

            Some(b'+') => Token::Plus,
            Some(b'-') => Token::Minus,
            Some(b'*') => Token::Star,
            Some(b'/') => Token::Slash,
            Some(b'(') => Token::LeftParen,
            Some(b')') => Token::RightParen,

            Some(ch) => return Err(LexError::UnexpectedChar(ch)),
        };

        // 第三步：统一消费单字符 Token
        self.advance();

        Ok(Some(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokens(input: &str, expected: &[Token]) {
        let mut lexer = Lexer { input, pos: 0 };

        for token in expected {
            assert_eq!(lexer.next_token().unwrap().as_ref(), Some(token));
        }

        assert!(matches!(lexer.next_token(), Ok(None)));
    }

    #[test]
    fn test_lexer_expr() {
        assert_tokens(
            "12 + 34 * (5 - 6)",
            &[
                Token::Number(12),
                Token::Plus,
                Token::Number(34),
                Token::Star,
                Token::LeftParen,
                Token::Number(5),
                Token::Minus,
                Token::Number(6),
                Token::RightParen,
            ],
        );
    }

    #[test]
    fn test_lexer_empty_input() {
        assert_tokens("", &[]);
        assert_tokens("   \t\n", &[]);
    }

    #[test]
    fn test_lexer_number() {
        assert_tokens("123", &[Token::Number(123)]);
        assert_tokens("00042", &[Token::Number(42)]);
    }

    #[test]
    fn test_lexer_invalid_char() {
        let mut lexer = Lexer {
            input: "1 @ 2",
            pos: 0,
        };

        assert_eq!(lexer.next_token().unwrap(), Some(Token::Number(1)));

        assert!(matches!(
            lexer.next_token(),
            Err(LexError::UnexpectedChar(b'@'))
        ));
    }

    #[test]
    fn test_lexer_integer_overflow() {
        assert_tokens("2147483647", &[Token::Number(i32::MAX)]);

        for input in ["2147483648", "999999999999999999999999"] {
            let mut lexer = Lexer { input, pos: 0 };

            assert!(matches!(lexer.next_token(), Err(LexError::IntegerOverflow)));
        }
    }
}
