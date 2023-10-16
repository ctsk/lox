use std::convert::identity;
use std::iter::Peekable;
use std::str::CharIndices;

use crate::bc::{Chunk, Op, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TokenType {
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

#[derive(Debug, PartialEq, Eq)]
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

    fn get_line(&self, pos: usize) -> usize {
        self.line_breaks
            .binary_search(&pos)
            .unwrap_or_else(identity)
    }

    fn get_lines(&self, start: usize, slice: &str) -> (usize, usize) {
        let end = start + slice.len();
        (self.get_line(start), self.get_line(end))
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
        self.iter.next_if(|&(_, c)| p(c)).map(|(p, _c)| p)
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
        for (p, c) in self.iter.by_ref() {
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

struct Parser<'src> {
    scanner: Peekable<Scanner<'src>>,
}

enum Associativity {
    Left,
    Right,
    NonAssoc,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl<'src> Parser<'src> {
    fn new(sc: Scanner<'src>) -> Self {
        Parser {
            scanner: sc.into_iter().peekable(),
        }
    }

    fn precedence(ttype: TokenType) -> Precedence {
        use TokenType::*;
        match ttype {
            Plus | Minus => Precedence::Term,
            Star | Slash => Precedence::Factor,
            EqualEqual | BangEqual => Precedence::Equality,
            Greater | GreaterEqual | Less | LessEqual => Precedence::Comparison,
            RightParen => Precedence::None,
            _ => panic!("{:?}", ttype),
        }
    }

    fn associativity(prec: Precedence) -> Associativity {
        use Precedence::*;
        match prec {
            Term | Factor | Equality | Comparison => Associativity::Left,
            None => Associativity::Left,
            Unary => Associativity::Right,
            _ => Associativity::NonAssoc,
        }
    }

    fn _expression(&mut self, chunk: &mut Chunk, min_prec: Precedence) {
        match self.scanner.next() {
            None => panic!("Expected further tokens"),
            Some(token) => match token.ttype {
                TokenType::Minus | TokenType::Bang => {
                    self._expression(chunk, Precedence::Unary);
                    let op = match token.ttype {
                        TokenType::Minus => Op::Negate,
                        TokenType::Bang => Op::Not,
                        _ => unreachable!(),
                    };
                    chunk.add_op(op, 0);
                },
                TokenType::Number => {
                    match token.span.parse::<f64>() {
                        Ok(c) => chunk.add_constant(c.into(), 0),
                        _ => panic!("Could not parse number"),
                    };
                },
                TokenType::String => {
                    let without_quotes = &token.span[1..(token.span.len() - 1)];
                    chunk.add_constant(without_quotes.into(), 0);
                },
                TokenType::Nil | TokenType::True | TokenType::False => {
                    let op = match token.ttype {
                        TokenType::Nil => Op::Nil,
                        TokenType::True => Op::True,
                        TokenType::False => Op::False,
                        _ => unreachable!()
                    };
                    chunk.add_op(op, 0);
                },
                TokenType::LeftParen => {
                    self._expression(chunk, Precedence::None);
                    assert_eq!(self.scanner.next().unwrap().ttype, TokenType::RightParen)
                },
                _ => panic!("Expected '-' or number"),
            },
        };

        while let Some(op) = self.scanner.next_if(|token| {
            let op_prec = Self::precedence(token.ttype);
            if op_prec == min_prec {
                match Self::associativity(min_prec) {
                    Associativity::Left => false,
                    Associativity::Right => true,
                    Associativity::NonAssoc => {
                        panic!("NonAssoc operation found in associative position")
                    }
                }
            } else {
                op_prec > min_prec
            }
        }) {
            // Generates code for rhs
            self._expression(chunk, Self::precedence(op.ttype));

            match op.ttype {
                TokenType::Plus => chunk.add_op(Op::Add, 0),
                TokenType::Minus => chunk.add_op(Op::Subtract, 0),
                TokenType::Star => chunk.add_op(Op::Multiply, 0),
                TokenType::Slash => chunk.add_op(Op::Divide, 0),
                TokenType::EqualEqual => chunk.add_op(Op::Equal, 0),
                TokenType::Greater => chunk.add_op(Op::Greater, 0),
                TokenType::Less => chunk.add_op(Op::Less, 0),
                TokenType::BangEqual => chunk.add_op(Op::Equal, 0).add_op(Op::Not, 0),
                TokenType::GreaterEqual => chunk.add_op(Op::Less, 0).add_op(Op::Not, 0),
                TokenType::LessEqual => chunk.add_op(Op::Greater, 0).add_op(Op::Not, 0),
                _ => todo!(),
            };
        }
    }

    pub fn expression(&mut self, chunk: &mut Chunk) {
        self._expression(chunk, Precedence::None)
    }
}

pub fn compile(source: &str, chunk: &mut Chunk) {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner);
    parser.expression(chunk);
}

#[cfg(test)]
mod tests {
    use crate::bc::Value;

    use super::*;

    #[test]
    fn test_scanner() {
        let source = "print(1+2*3);";
        let scanner = Scanner::new(source);

        let tokens: Vec<Token> = scanner.collect();

        assert_eq!(
            tokens,
            vec![
                Token {
                    ttype: TokenType::Print,
                    span: &source[0..=4]
                },
                Token {
                    ttype: TokenType::LeftParen,
                    span: &source[5..=5]
                },
                Token {
                    ttype: TokenType::Number,
                    span: &source[6..=6]
                },
                Token {
                    ttype: TokenType::Plus,
                    span: &source[7..=7]
                },
                Token {
                    ttype: TokenType::Number,
                    span: &source[8..=8]
                },
                Token {
                    ttype: TokenType::Star,
                    span: &source[9..=9]
                },
                Token {
                    ttype: TokenType::Number,
                    span: &source[10..=10]
                },
                Token {
                    ttype: TokenType::RightParen,
                    span: &source[11..=11]
                },
                Token {
                    ttype: TokenType::Semicolon,
                    span: &source[12..=12]
                }
            ]
        );
    }

    #[test]
    fn string_scan() {
        let source = "\"hello world\"";
        let scanner = Scanner::new(source);
        let tokens: Vec<Token> = scanner.collect();

        assert_eq!(
            tokens,
            vec![
             Token {
                 ttype: TokenType::String,
                 span: &source[0..=12]
             }
            ]
        );

        assert_eq!(tokens[0].span, source);
    }

    fn test_parse_expression(source: &str, expected: &Chunk) {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let mut chunk = Chunk::new();
        parser.expression(&mut chunk);
        assert!(chunk.instr_eq(expected));
    }

    #[test]
    fn test_parser() {
        let source = "1 + 1 * (2 + 1)";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![
                Constant { offset: 0 },
                Constant { offset: 1 },
                Constant { offset: 2 },
                Constant { offset: 3 },
                Add,
                Multiply,
                Add,
            ],
            vec![],
            vec![1., 1., 2., 1.].into_iter().map(Value::from).collect(),
        );

        test_parse_expression(source, &expected);
    }

    #[test]
    fn parse_nil() {
        let source = "nil + nil";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![Nil, Nil, Add],
            vec![],
            vec![],
        );

        test_parse_expression(source, &expected);
    }

    #[test]
    fn parse_bool_literals() {
        let source = "true * false";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![True, False, Multiply],
            vec![],
            vec![],
        );

        test_parse_expression(source, &expected);
    }

    #[test]
    fn parse_bool_expression() {
        let source = "!false == !true >= true <= false > true < false != true";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![
                False, Not, True, Not, True, Less, Not,
                False, Greater, Not,
                 True, Greater,
                False, Less,
                Equal,
                 True, Equal, Not],
            vec![],
            vec![],
        );

        test_parse_expression(source, &expected);
    }
}
