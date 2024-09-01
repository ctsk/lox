use std::iter::Peekable;
use std::ops::Range;
use std::str::CharIndices;
use std::{collections::HashMap, convert::identity};

use crate::bc::{Chunk, Op};
use crate::gc::allocate_string;

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

#[derive(Debug, PartialEq, Eq, Clone)]
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
                .bytes()
                .enumerate()
                .filter_map(|(pos, c)| if c == b'\n' { Some(pos) } else { None })
                .collect(),
        }
    }

    fn pos_to_line(&self, pos: usize) -> usize {
        let (Ok(index) | Err(index)) = self.line_breaks.binary_search(&pos);
        index + 1
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

    pub fn get_range(&self, token: Token) -> Option<Range<usize>> {
        let [source_addr, span_addr]: [usize; 2] =
            [self.source, token.span].map(|s| s.as_ptr() as usize);
        if span_addr < source_addr || span_addr + token.span.len() > source_addr + self.source.len()
        {
            return None; // out of bounds
        }
        let start_index = span_addr - source_addr;
        Some(start_index..start_index + token.span.len())
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

        // todo: make robust to non-ascii chars

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
        self.consume_until_eq('\n');
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
                    Some(_) => {
                        self.scan_comment();
                        self.next()
                    }
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
    errors: Vec<ParseError<'src>>,
    intern_table: HashMap<&'src str, u8>,
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

enum ParseError<'src> {
    InvalidNumber(Token<'src>),
    UnexpectedToken(Vec<TokenType>, Token<'src>),
    UnexpectedEOF,
}

type Result<'src> = std::result::Result<(), ParseError<'src>>;

impl<'src> Parser<'src> {
    fn new(sc: Scanner<'src>) -> Self {
        Parser {
            scanner: sc.into_iter().peekable(),
            errors: Vec::new(),
            intern_table: HashMap::new(),
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
            _ => panic!("Undefined precedence: {:?}", ttype),
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

    fn _expression(&mut self, chunk: &mut Chunk, min_prec: Precedence) -> Result<'src> {
        match self.scanner.next() {
            None => return Err(ParseError::UnexpectedEOF),
            Some(token) => match token.ttype {
                TokenType::Minus | TokenType::Bang => {
                    self._expression(chunk, Precedence::Unary)?;
                    let op = match token.ttype {
                        TokenType::Minus => Op::Negate,
                        TokenType::Bang => Op::Not,
                        _ => unreachable!(),
                    };
                    chunk.add_op(op, 0);
                }
                TokenType::Number => {
                    match token.span.parse::<f64>() {
                        Ok(c) => Ok(chunk.add_constant(c.into(), 0)),
                        _ => Err(ParseError::InvalidNumber(token)),
                    }?;
                }
                TokenType::String => {
                    let without_quotes = &token.span[1..(token.span.len() - 1)];
                    match self.intern_table.get(without_quotes) {
                        Some(&index) => {
                            chunk.add_op(Op::Constant { offset: index }, 0);
                        }
                        None => {
                            let object = unsafe { allocate_string(without_quotes) }.unwrap();
                            chunk.add_constant(object.get_object().into(), 0);
                            self.intern_table
                                .insert(without_quotes, chunk.constants.len() as u8 - 1);
                            chunk.allocations.push_front(object);
                        }
                    };
                }
                TokenType::LeftParen => {
                    self._expression(chunk, Precedence::None)?;
                    assert_eq!(self.scanner.next().unwrap().ttype, TokenType::RightParen)
                }
                TokenType::Nil => {
                    chunk.add_op(Op::Nil, 0);
                }
                TokenType::True => {
                    chunk.add_op(Op::True, 0);
                }
                TokenType::False => {
                    chunk.add_op(Op::False, 0);
                }
                _ => {
                    use TokenType::*;
                    return Err(ParseError::UnexpectedToken(
                        vec![Minus, Bang, Number, String, Nil, True, False, LeftParen],
                        token,
                    ));
                }
            },
        };

        while let Some(op) = self.scanner.next_if(|token| {
            if token.ttype == TokenType::Semicolon {
                return false;
            }

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
            self._expression(chunk, Self::precedence(op.ttype))?;

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
                _ => {
                    use TokenType::*;
                    return Err(ParseError::UnexpectedToken(
                        vec![
                            Plus,
                            Minus,
                            Star,
                            Slash,
                            EqualEqual,
                            Greater,
                            Less,
                            BangEqual,
                            GreaterEqual,
                            LessEqual,
                        ],
                        op,
                    ));
                }
            };
        }

        Ok(())
    }

    pub fn expression(&mut self, chunk: &mut Chunk) -> Result<'src> {
        self._expression(chunk, Precedence::None)
    }

    pub fn must_consume(&mut self, ttype: TokenType) -> Result<'src> {
        match self.scanner.peek() {
            Some(token) => {
                if token.ttype == ttype {
                    self.scanner.next();
                    Ok(())
                } else {
                    Err(ParseError::UnexpectedToken(vec![ttype], token.clone()))
                }
            },
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn print_statement(&mut self, chunk: &mut Chunk) -> Result<'src> {
        self.must_consume(TokenType::Print)?;
        self.expression(chunk)?;
        chunk.add_op(Op::Print, 0);
        self.must_consume(TokenType::Semicolon)
    }

    pub fn expr_statement(&mut self, chunk: &mut Chunk) -> Result<'src> {
        self.expression(chunk)?;
        chunk.add_op(Op::Pop, 0);
        self.must_consume(TokenType::Semicolon)
    }

    pub fn statement(&mut self, chunk: &mut Chunk) -> Result<'src> {
        match self.scanner.peek().unwrap().ttype {
            TokenType::Print => self.print_statement(chunk),
            _ => self.expr_statement(chunk),
        }
    }

    pub fn synchronize(&mut self) {
        use TokenType::*;
        while let Some(token) = self.scanner.next_if(
            |tok| ![Semicolon, Class, Fun, Var, For, If, While, Print, Return].contains(&tok.ttype)
        ) {}


    }

    pub fn declaration(&mut self, chunk: &mut Chunk) {
        self.statement(chunk).unwrap_or_else(
            |err| {
                self.errors.push(err);
                self.synchronize();
            }
        )
    }

    pub fn compile(&mut self, chunk: &mut Chunk) {
        while let Some(_) = self.scanner.peek() {
            self.declaration(chunk)
        }
    }
}

#[cfg(test)]
pub fn compile_expr(source: &str, chunk: &mut Chunk) {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner);
    parser.expression(chunk);
}

pub fn compile(source: &str, chunk: &mut Chunk) {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner);
    parser.compile(chunk);
}

#[cfg(test)]
mod tests {
    use std::collections::LinkedList;

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
                    span: "print"
                },
                Token {
                    ttype: TokenType::LeftParen,
                    span: "("
                },
                Token {
                    ttype: TokenType::Number,
                    span: "1"
                },
                Token {
                    ttype: TokenType::Plus,
                    span: "+"
                },
                Token {
                    ttype: TokenType::Number,
                    span: "2"
                },
                Token {
                    ttype: TokenType::Star,
                    span: "*"
                },
                Token {
                    ttype: TokenType::Number,
                    span: "3"
                },
                Token {
                    ttype: TokenType::RightParen,
                    span: ")"
                },
                Token {
                    ttype: TokenType::Semicolon,
                    span: ";"
                }
            ]
        );
    }

    // #[test]
    // fn number_scan() {
    //     let source = "1a";
    //     let scanner = Scanner::new(source);
    //     let tokens: Vec<Token> = scanner.collect();
    //     assert_eq!(
    //         tokens,
    //         vec![Token{ttype: TokenType::Number, span: "1a"}]
    //     );
    // }

    #[test]
    fn comment_scan() {
        let source = "1\n2//comment\n3";
        let scanner = Scanner::new(source);
        let tokens: Vec<Token> = scanner.collect();

        assert_eq!(
            tokens,
            vec![
                Token {
                    ttype: TokenType::Number,
                    span: "1"
                },
                Token {
                    ttype: TokenType::Number,
                    span: "2"
                },
                Token {
                    ttype: TokenType::Number,
                    span: "3"
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
            vec![Token {
                ttype: TokenType::String,
                span: "\"hello world\""
            }]
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

    fn test_parse_program<'src>(source: &'src str, expected: &Chunk) {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let mut chunk = Chunk::new();
        parser.compile(&mut chunk);
        assert!(parser.errors.is_empty());
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
            LinkedList::new(),
        );

        test_parse_expression(source, &expected);
    }

    #[test]
    fn parse_nil() {
        let source = "nil + nil";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(vec![Nil, Nil, Add], vec![], vec![], LinkedList::new());

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
            LinkedList::new(),
        );

        test_parse_expression(source, &expected);
    }

    #[test]
    fn parse_bool_expression() {
        let source = "!false == !true >= true <= false > true < false != true";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![
                False, Not, True, Not, True, Less, Not, False, Greater, Not, True, Greater, False,
                Less, Equal, True, Equal, Not,
            ],
            vec![],
            vec![],
            LinkedList::new(),
        );

        test_parse_expression(source, &expected);
    }

    #[test]
    fn string_interning() {
        let source = "\"ho\" + \"ho\" + \"ho\"";
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let mut chunk = Chunk::new();
        parser.expression(&mut chunk);

        assert_eq!(chunk.allocations.len(), 1);
        assert_eq!(chunk.constants.len(), 1);
    }

    #[test]
    fn basic_print_statement() {
        let source = "print 1 + 1;";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![Constant { offset: 0 }, Constant { offset: 1 }, Add, Print],
            vec![],
            vec![Value::from(1.0), Value::from(1.0)],
            LinkedList::new(),
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn basic_print_string_statement() {
        let source = "print \"string\";";
        let allocation = unsafe { allocate_string("string").unwrap() };
        let object = allocation.get_object();
        let mut allocations = LinkedList::new();
        allocations.push_front(allocation);
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![Constant { offset: 0 }, Print],
            vec![],
            vec![Value::from(object)],
            allocations,
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn basic_expr_statement() {
        let source = "1 / 1;";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![Constant { offset: 0 }, Constant { offset: 1 }, Divide, Pop],
            vec![],
            vec![Value::from(1.0), Value::from(1.0)],
            LinkedList::new(),
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn no_line_breaks() {
        let line_map = LineMap::new("0123456789");

        for i in 0..=9 {
            assert_eq!(line_map.pos_to_line(i), 1);
        }
    }

    #[test]
    fn some_line_breaks() {
        let line_map = LineMap::new("012\n456\n89\n");

        for i in 0..=2 {
            assert_eq!(line_map.pos_to_line(i), 1);
        }

        assert_eq!(line_map.pos_to_line(3), 1);

        for i in 4..=6 {
            assert_eq!(line_map.pos_to_line(i), 2);
        }

        assert_eq!(line_map.pos_to_line(7), 2);

        for i in 8..=9 {
            assert_eq!(line_map.pos_to_line(i), 3);
        }

        assert_eq!(line_map.pos_to_line(10), 3);
    }
}
