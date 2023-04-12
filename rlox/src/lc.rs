use std::collections::BTreeSet;
use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug)]
enum TokenType {
    Eof,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Number,

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
}

#[derive(Debug)]
pub struct Token<'src> {
    ttype: TokenType,
    span: &'src str,
}

pub struct Scanner<'src> {
    source: &'src str,
    iter: Peekable<CharIndices<'src>>,
    line_map: LineMap,
}

pub struct LineMap {
    line_breaks: Vec<usize>,
}

impl LineMap {
    fn new(source: &str) -> LineMap {
        LineMap {
            line_breaks: source
                .char_indices()
                .filter_map(|(pos, c)| if c == '\n' { Some(pos) } else { None })
                .collect(),
        }
    }

    fn get_lines(&self, slice: &str) -> (usize, usize) {
        return (0, 0)
    }
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Scanner {
            source,
            iter: source.char_indices().peekable(),
            line_map: LineMap::new(source),
        }
    }

    fn make_token(&self, ttype: TokenType, start: usize, end: usize) -> Token<'src> {
        Token {
            ttype,
            span: &self.source[start..=end],
        }
    }

    fn consume_if<P>(&mut self, p: P) -> Option<usize>
    where
        P: Fn(char) -> bool,
    {
        self.iter.next_if(|&(_, c)| p(c)).map(|(p, c)| p)
    }

    fn consume_if_eq(&mut self, expected: char) -> Option<usize> {
        self.consume_if(|c| c == expected)
    }

    fn consume_while<P>(&mut self, p: P) -> Option<usize>
    where
        P: Fn(char) -> bool + Copy,
    {
        self.consume_if(p).map(|pos| {
            let mut last = pos;
            while let Some(pos) = self.consume_if(p) {
                last = pos
            }
            last
        })
    }

    fn consume_until_eq(&mut self, limit: char) -> Option<usize> {
        while let Some((p, c)) = self.iter.next() {
            if c == limit {
                return Some(p);
            }
        }
        None
    }

    fn scan_string(&mut self, start: usize) -> Token<'src> {
        let end = self.consume_until_eq('"').unwrap_or_else(|| {
            panic!("Undelimited String");
        });

        self.make_token(TokenType::String, start, end)
    }

    fn scan_number(&mut self, start: usize) -> Token<'src> {
        let mut end = start;
        end = self
            .consume_while(|c| c.is_ascii_alphanumeric())
            .unwrap_or(end);

        if let Some(pos) = self.consume_if_eq('.') {
            end = pos;

            end = self
                .consume_while(|c| c.is_ascii_alphanumeric())
                .unwrap_or(end);
        }

        self.make_token(TokenType::Number, start, end)
    }

    fn scan_identifier(&mut self, start: usize) -> Token<'src> {
        let mut end = start;

        end = self
            .consume_while(|c| c.is_ascii_alphanumeric())
            .unwrap_or(end);

        let slice = &self.source[start..=end];

        let ttype = match slice {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        };

        Token { ttype, span: slice }
    }

    fn scan_comment(&mut self) {
        self.consume_until_eq('"');
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip Whitespace
        while self
            .iter
            .next_if(|(_, b)| b.is_ascii_whitespace())
            .is_some()
        {}

        if let Some((start_pos, start_ch)) = self.iter.next() {
            let make_simple_token =
                |s: &Self, ttype: TokenType| Some(s.make_token(ttype, start_pos, start_pos));

            let handle_eq_suffix = |s: &mut Self, if_present: TokenType, if_absent: TokenType| {
                Some(match s.consume_if_eq('=') {
                    Some(end) => s.make_token(if_present, start_pos, end),
                    None => s.make_token(if_absent, start_pos, start_pos),
                })
            };

            match start_ch {
                '(' => make_simple_token(self, TokenType::LeftParen),
                ')' => make_simple_token(self, TokenType::RightParen),
                '{' => make_simple_token(self, TokenType::LeftBrace),
                '}' => make_simple_token(self, TokenType::RightBrace),
                ',' => make_simple_token(self, TokenType::Comma),
                '.' => make_simple_token(self, TokenType::Dot),
                '-' => make_simple_token(self, TokenType::Minus),
                '+' => make_simple_token(self, TokenType::Plus),
                ';' => make_simple_token(self, TokenType::Semicolon),
                '/' => match self.consume_if_eq('/') {
                    Some(_) => self.next(),
                    None => make_simple_token(self, TokenType::Slash),
                },
                '*' => make_simple_token(self, TokenType::Star),
                '!' => handle_eq_suffix(self, TokenType::BangEqual, TokenType::Bang),
                '=' => handle_eq_suffix(self, TokenType::EqualEqual, TokenType::Equal),
                '>' => handle_eq_suffix(self, TokenType::GreaterEqual, TokenType::Greater),
                '<' => handle_eq_suffix(self, TokenType::LessEqual, TokenType::Less),
                _ => {
                    let token = if start_ch.is_ascii_digit() {
                        self.scan_number(start_pos)
                    } else if start_ch.is_ascii_alphabetic() {
                        self.scan_identifier(start_pos)
                    } else if start_ch == '"' {
                        self.scan_string(start_pos)
                    } else {
                        panic!("Invalid character");
                    };

                    Some(token)
                }
            }
        } else {
            None
        }
    }
}

pub fn compile(source: &str) {
    let scanner = Scanner::new(source);

    for token in scanner {
        println!("{:?}", token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        compile("print(1+2*3)");
    }
}
