/*
* Copyright (C) 2019-2021, Miklos Maroti
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

//! A tokenizer that breaks down an input string to standard tokens.

/// Standard token types.
#[derive(PartialEq, Eq, Debug)]
pub enum Token<'a> {
    Literal(&'a str),
    Integer(usize),
    Operator(char),
    String(&'a str),
    Error(&'a str),
}

/// A tokenizer that breaks down an input string into tokens separated by
/// whitespace.
pub struct Tokenizer<'a> {
    /// byte position
    index: usize,

    /// input string
    input: &'a str,

    /// operator characters
    opers: &'a str,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer for the given input string and operator
    /// characters.
    pub fn new(input: &'a str, opers: &'a str) -> Self {
        Self {
            index: 0,
            input,
            opers,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iter = self.input[self.index..].char_indices();

        // eat whitespace
        let mut pos1 = self.index;
        let mut head = ' ';
        for (n, c) in &mut iter {
            if !c.is_whitespace() {
                pos1 = self.index + n;
                head = c;
                break;
            }
        }

        // end of string
        if head == ' ' {
            self.index = self.input.len();
            return None;
        }

        // handle cases
        let mut pos2 = self.input.len();
        let token = if head.is_alphabetic() {
            for (n, c) in iter {
                if !c.is_alphanumeric() {
                    pos2 = self.index + n;
                    break;
                }
            }
            Token::Literal(&self.input[pos1..pos2])
        } else if head.is_ascii_digit() {
            for (n, c) in iter {
                if !c.is_ascii_digit() {
                    pos2 = self.index + n;
                    break;
                }
            }
            match self.input[pos1..pos2].parse::<usize>() {
                Ok(num) => Token::Integer(num),
                Err(_) => Token::Error(&self.input[pos1..pos2]),
            }
        } else if head == '"' {
            for (n, c) in iter {
                if c == '"' {
                    pos2 = self.index + n;
                    break;
                }
            }
            if pos2 == self.input.len() {
                Token::Error(&self.input[pos1..])
            } else {
                pos2 += 1;
                Token::String(&self.input[(pos1 + 1)..(pos2 - 1)])
            }
        } else {
            pos2 = pos1 + head.len_utf8();
            if self.opers.contains(head) {
                Token::Operator(head)
            } else {
                Token::Error(&self.input[pos1..pos2])
            }
        };

        self.index = pos2;
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer() {
        let mut tokens = Tokenizer::new(
            " ab \"12x \"c2 34d 123456789123456789123 x(999+ \"y",
            "()+-*/",
        );
        assert_eq!(tokens.next(), Some(Token::Literal("ab")));
        assert_eq!(tokens.next(), Some(Token::String("12x ")));
        assert_eq!(tokens.next(), Some(Token::Literal("c2")));
        assert_eq!(tokens.next(), Some(Token::Integer(34)));
        assert_eq!(tokens.next(), Some(Token::Literal("d")));
        assert_eq!(tokens.next(), Some(Token::Error("123456789123456789123")));
        assert_eq!(tokens.next(), Some(Token::Literal("x")));
        assert_eq!(tokens.next(), Some(Token::Operator('(')));
        assert_eq!(tokens.next(), Some(Token::Integer(999)));
        assert_eq!(tokens.next(), Some(Token::Operator('+')));
        assert_eq!(tokens.next(), Some(Token::Error("\"y")));
        assert_eq!(tokens.next(), None);
    }
}
