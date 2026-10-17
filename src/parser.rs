use crate::ast::Expr;
use crate::lexer::{LexError, Lexer, Token};

#[derive(Debug)]
pub enum ParseError {
    Lexical(LexError),
    UnexpectedToken,
    UnexpectedEof,
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError::Lexical(err)
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>,
}

// expr    = term, { ("+" | "-"), term } ;
// term    = primary, { ("*" | "/"), primary } ;
// primary = NUMBER | "(", expr, ")" ;

// expr    = term, { ("+" | "-"), term } ;
// term    = unary, { ("*" | "/"), unary } ;
// unary   = "-", unary | primary ;
// primary = NUMBER | "(", expr, ")" ;

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, LexError> {
        let mut lexer = Lexer::new(input);

        let current = lexer.next_token()?;

        Ok(Self { lexer, current })
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn advance(&mut self) -> Result<Option<Token>, LexError> {
        let next_token = self.lexer.next_token()?;

        let old_token = std::mem::replace(&mut self.current, next_token);

        Ok(old_token)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_term()?;

        while matches!(self.peek(), Some(Token::Plus | Token::Minus)) {
            let op = self.advance()?.unwrap();

            let rhs = self.parse_term()?;

            let expr = match op {
                Token::Plus => Expr::Add(Box::new(lhs), Box::new(rhs)),
                Token::Minus => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            };

            lhs = expr;
        }

        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;

        while matches!(self.peek(), Some(Token::Star | Token::Slash)) {
            let op = self.advance()?.unwrap();

            let rhs = self.parse_unary()?;

            let term = match op {
                Token::Star => Expr::Mul(Box::new(lhs), Box::new(rhs)),
                Token::Slash => Expr::Div(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            };

            lhs = term;
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek();

        match token {
            Some(Token::Minus) => {
                self.advance()?;
                let unary = self.parse_unary()?;
                Ok(Expr::Neg(Box::new(unary)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance()?;
        // let token = self.advance().map_err(ParseError::Lexical)?';

        match token {
            Some(Token::Number(num)) => Ok(Expr::Number(num)),

            Some(Token::LeftParen) => {
                let expr = self.parse_expr()?;

                let token = self.advance()?;

                match token {
                    Some(Token::RightParen) => Ok(expr),
                    Some(_) => Err(ParseError::UnexpectedToken),
                    None => Err(ParseError::UnexpectedEof),
                }
            }

            Some(_) => Err(ParseError::UnexpectedToken),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_expr()?;

        if self.peek().is_some() {
            return Err(ParseError::UnexpectedToken);
        }

        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_ast(input: &str, expected: Expr) {
        let mut parser = Parser::new(input).unwrap();
        let actual = parser.parse().unwrap();

        assert_eq!(actual, expected);
    }

    fn num(n: i32) -> Expr {
        Expr::Number(n)
    }

    fn add(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Add(Box::new(lhs), Box::new(rhs))
    }

    fn sub(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Sub(Box::new(lhs), Box::new(rhs))
    }

    fn mul(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Mul(Box::new(lhs), Box::new(rhs))
    }

    fn neg(hs: Expr) -> Expr {
        Expr::Neg(Box::new(hs))
    }

    #[test]
    fn test_parse_expr() {
        assert_ast(
            "8 - 3 - 2 * 4",
            sub(sub(num(8), num(3)), mul(num(2), num(4))),
        );
    }

    #[test]
    fn test_parse_parentheses() {
        assert_ast("(1 + 2) * 3", mul(add(num(1), num(2)), num(3)));
    }

    #[test]
    fn test_parse_nested_parentheses() {
        assert_ast(
            "2 * (3 + (4 - 1))",
            mul(num(2), add(num(3), sub(num(4), num(1)))),
        );
    }

    #[test]
    fn test_parse_negative() {
        assert_ast("----1", neg(neg(neg(neg(num(1))))));
    }

    #[test]
    fn test_negative_precedence() {
        assert_ast("-2 * 3", mul(neg(num(2)), num(3)));
    }

    #[test]
    fn test_negative_parentheses() {
        assert_ast("-(1 + 2)", neg(add(num(1), num(2))));
    }
}
