// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

//! Lexer for MiniObli language.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Bool(bool),
    Ident(String),

    // Keywords
    Let,
    If,
    Then,
    Else,
    Secret,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,

    // Delimiters
    LParen,
    RParen,
    Assign,

    // End
    Eof,
}

#[derive(Error, Debug)]
pub enum LexError {
    #[error("unexpected character: '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    #[error("invalid number at position {0}")]
    InvalidNumber(usize),
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            pos: 0,
        }
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, _)) = result {
            self.pos = pos;
        }
        result
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '#' {
                // Skip comments
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self, start: usize) -> Result<Token, LexError> {
        let mut end = start;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                end = self.advance().unwrap().0;
            } else {
                break;
            }
        }
        let num_str = &self.input[start..=end];
        num_str
            .parse::<i64>()
            .map(Token::Int)
            .map_err(|_| LexError::InvalidNumber(start))
    }

    fn read_ident(&mut self, start: usize) -> Token {
        let mut end = start;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                end = self.advance().unwrap().0;
            } else {
                break;
            }
        }
        let ident = &self.input[start..=end];
        match ident {
            "let" => Token::Let,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "secret" => Token::Secret,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Ident(ident.to_string()),
        }
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        let (pos, c) = match self.advance() {
            Some(pair) => pair,
            None => return Ok(Token::Eof),
        };

        match c {
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => Ok(Token::Slash),
            '%' => Ok(Token::Percent),
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Eq)
                } else {
                    Ok(Token::Assign)
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Ne)
                } else {
                    Ok(Token::Not)
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Le)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Ge)
                } else {
                    Ok(Token::Gt)
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Err(LexError::UnexpectedChar(c, pos))
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Err(LexError::UnexpectedChar(c, pos))
                }
            }
            _ if c.is_ascii_digit() => self.read_number(pos),
            _ if c.is_alphabetic() || c == '_' => Ok(self.read_ident(pos)),
            _ => Err(LexError::UnexpectedChar(c, pos)),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Token::Eof) => None,
            other => Some(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "let x = 42";
        let lexer = Lexer::new(input);
        let tokens: Result<Vec<_>, _> = lexer.collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::Let,
                Token::Ident("x".to_string()),
                Token::Assign,
                Token::Int(42),
            ]
        );
    }

    #[test]
    fn test_operators() {
        let input = "1 + 2 * 3 == 7";
        let lexer = Lexer::new(input);
        let tokens: Result<Vec<_>, _> = lexer.collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::Int(1),
                Token::Plus,
                Token::Int(2),
                Token::Star,
                Token::Int(3),
                Token::Eq,
                Token::Int(7),
            ]
        );
    }

    #[test]
    fn test_if_then_else() {
        let input = "if x > 0 then x else 0";
        let lexer = Lexer::new(input);
        let tokens: Result<Vec<_>, _> = lexer.collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::If,
                Token::Ident("x".to_string()),
                Token::Gt,
                Token::Int(0),
                Token::Then,
                Token::Ident("x".to_string()),
                Token::Else,
                Token::Int(0),
            ]
        );
    }
}
