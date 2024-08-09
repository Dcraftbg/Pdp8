use std::num::ParseIntError;

use crate::{Token, TokenKind};

#[derive(Debug)]
#[allow(dead_code)]
pub enum LexerError {
    Unparsable(char),
    ParseIntError(ParseIntError),
}
impl From<ParseIntError> for LexerError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}
pub struct Lexer<'d> {
    src: &'d str,
}
impl <'d> Lexer <'d> {
    pub const fn new(src: &'d str) -> Self {
        Self { src }
    }
    // NOTE: Needed later on when we introduce Location
    fn trim(&mut self) {
        self.src = self.src.trim_start();
    }
    fn parse_int_literal(&mut self, lit: &str) -> Result<u16, LexerError> {
        if let Some(lit) = lit.strip_prefix("0b") {
            Ok(u16::from_str_radix(lit, 2)?)
        } else if let Some(lit) = lit.strip_prefix("0x") {
            Ok(u16::from_str_radix(lit, 16)?)
        } else if let Some(lit) = lit.strip_prefix("0o") {
            Ok(u16::from_str_radix(lit, 12)?)
        } else if let Ok(lit_int) = u16::from_str_radix(lit, 10) {
            Ok(lit_int)
        } else {
            todo!("Unparsable integer literal {}",lit);
        }
    }
    fn report(&mut self, e: LexerError) -> ! {
        eprintln!("ERROR: Lexer: {:?}",e);
        panic!();
    }
    fn parse_word(&mut self) -> &'d str {
        let mut end = self.src.len();
        for (i,c) in self.src.char_indices() {
            if !c.is_alphanumeric() && c != '_' {
               end = i;
               break; 
            }
        }
        let (word, rest) = self.src.split_at(end);
        self.src = rest;
        word
    }
    pub fn next(&mut self) -> Option<Token<'d>> {
        self.trim();
        if self.src.len() == 0 { return None; }
        match self.src.chars().next().unwrap() {
            ';' => {
                let mut end = self.src.len();
                for (i,c) in self.src.char_indices() {
                    if c == '\n' {
                       end = i;
                       break; 
                    }
                }
                let (_comment, rest) = self.src.split_at(end);
                self.src = rest;
                self.next()
            }
            '$' => {
                self.src = self.src.strip_prefix("$").unwrap();
                let mut end = self.src.len();
                for (i,c) in self.src.char_indices() {
                    if !c.is_alphanumeric() {
                       end = i;
                       break; 
                    }
                }
                let (lit, rest) = self.src.split_at(end);
                if lit.len() == 0 {
                    return Some(Token { kind: TokenKind::CurrentInst }); 
                }
                self.src = rest;
                Some(
                  Token {
                      kind: TokenKind::Integer(self.parse_int_literal(lit)
                                .unwrap_or_else(|e| self.report(e)))
                  }
                )
            }

            '=' => {
                self.src = &self.src[1..];
                Some(Token { kind: TokenKind::Equal })
            } 
            '.' => {
                self.src = &self.src[1..];
                Some(Token { kind: TokenKind::DotWord(self.parse_word()) })
            } 
            c if c.is_alphabetic() => {
                Some(Token { kind: TokenKind::Word(self.parse_word()) })
            }
            '[' => {
                self.src = &self.src[1..];
                Some(Token { kind: TokenKind::OpenSquare })
            }
            ':' => {
                self.src = &self.src[1..];
                Some(Token { kind: TokenKind::DoubleDot })
            }
            ']' => {
                self.src = &self.src[1..];
                Some(Token { kind: TokenKind::CloseSquare })
            }
            c => self.report(LexerError::Unparsable(c))
        }
    }
    pub fn peak(&mut self) -> Option<Token<'d>> {
        let src = self.src;
        let res = self.next();
        self.src = src;
        res
    }
    pub fn eat(&mut self) {
        self.next().expect("Token to eat");
    }
}
