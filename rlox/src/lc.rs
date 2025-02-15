use std::{collections::hash_map, fmt};
use std::iter::Peekable;
use std::str::CharIndices;
use std::collections::HashMap;

use crate::bc::Value;
use crate::{bc::{Chunk, Op}, gc::GC};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanErrorKind {
    UndelimitedString,
}

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

    Error(ScanErrorKind),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<'src> {
    ttype: TokenType,
    span: &'src str,
    line: usize,
}

struct Source<'src> {
    line: usize,
    iter: CharIndices<'src>,
}

impl<'src> Source<'src> {
    fn new(str: &'src str) -> Self {
        Source {
            line: 1,
            iter: str.char_indices(),
        }
    }
}

impl Iterator for Source<'_> {
    type Item = (usize, usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((p, ch)) => {
                let old_line = self.line;
                if ch == '\n' {
                    self.line += 1;
                }

                Some((old_line, p, ch))
            },
            None => None,
        }
    }
}

pub struct Scanner<'src> {
    source: &'src str,
    iter: Peekable<Source<'src>>,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Scanner {
            source,
            iter: Source::new(source).peekable(),
        }
    }

    fn make_token(&self, ttype: TokenType, line: usize, start: usize, end: usize) -> Token<'src> {
        Token {
            ttype,
            span: &self.source[start..=end],
            line,
        }
    }

    fn consume_if<P>(&mut self, p: P) -> Option<usize>
    where
        P: Fn(char) -> bool,
    {
        self.iter.next_if(|&(_, _, c)| p(c)).map(|(_l, p, _c)| p)
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

    fn consume_until_eq(&mut self, limit: char) -> std::result::Result<usize, usize> {
        for (_line, p, c) in self.iter.by_ref() {
            if c == limit {
                return Ok(p);
            }
        }

        Err(self.source.len())
    }

    fn scan_string(&mut self) -> std::result::Result<usize, usize> {
        self.consume_until_eq('"')
    }

    fn scan_number(&mut self, start: usize) -> usize {
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

        end
    }

    fn scan_identifier(&mut self, line: usize, start: usize) -> Token<'src> {
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

        Token { ttype, span: slice, line }
    }

    fn scan_comment(&mut self) {
        let _ = self.consume_until_eq('\n');
    }
}

impl<'src> Iterator for Scanner<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_while(|ch| ch.is_ascii_whitespace());

        if let Some((start_line, start_pos, start_ch)) = self.iter.next() {
            let make_simple_token =
                |s: &Self, ttype: TokenType| Some(s.make_token(ttype, start_line, start_pos, start_pos));

            let handle_eq_suffix = |s: &mut Self, if_present: TokenType, if_absent: TokenType| {
                Some(match s.consume_if_eq('=') {
                    Some(end) => s.make_token(if_present, start_line, start_pos, end),
                    None => s.make_token(if_absent, start_line, start_pos, start_pos),
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
                        let end = self.scan_number(start_pos);
                        self.make_token(TokenType::Number, start_line, start_pos, end)
                    } else if start_ch.is_ascii_alphabetic() {
                        self.scan_identifier(start_line, start_pos)
                    } else if start_ch == '"' {
                        match self.scan_string() {
                            Ok(end) => self.make_token(TokenType::String, start_line, start_pos, end),
                            Err(end) => self.make_token(TokenType::Error(ScanErrorKind::UndelimitedString), start_line, start_pos, end - 1),
                        }
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

struct Compiler {
    locals: Vec<(String, usize, bool)>,
    scope_depth: usize,
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler {
            locals: Vec::new(),
            scope_depth: 0,
        }
    }
}

enum LocalsError {
    TooMany,
    DuplicateInScope,
}

impl Compiler {
    fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn exit_scope(&mut self) -> usize {
        let mut pop_count = 0;
        while let Some(local) = self.locals.last() {
            if local.1 < self.scope_depth {
                break;
            }
            self.locals.pop();
            pop_count += 1;
        }
        self.scope_depth -= 1;
        return pop_count;
    }

    fn in_global_scope(&self) -> bool {
        self.scope_depth <= 0
    }

    fn declare_local(&mut self, name: &str) -> std::result::Result<(), LocalsError> {
        if self.locals.len() > u8::MAX as usize {
            Err(LocalsError::TooMany)
        } else {
            for idx in (0..self.locals.len()).rev() {
                if self.locals[idx].1 < self.scope_depth {
                    break
                }

                if self.locals[idx].0 == name {
                    return Err(LocalsError::DuplicateInScope)
                }
            }

            self.locals.push((name.to_string(), self.scope_depth, false));
            Ok(())
        }
    }

    fn mark_last_initialized(&mut self) {
       self.locals.last_mut().unwrap().2 = true;
    }

    // Err ( false ) -> not declared
    // Err ( true ) -> declared but not initialized
    fn resolve_local(&self, target: &str) -> std::result::Result<u8, bool> {
        for idx in (0..self.locals.len()).rev() {
            if self.locals[idx].0 == target {
                return if !self.locals[idx].2 {
                    Err(true)
                } else {
                    Ok(idx as u8)
                }
            }
        }

        Err(false)
    }
}

struct Parser<'src> {
    scanner: Peekable<Scanner<'src>>,
    errors: Vec<ParseError<'src>>,
    intern_table: HashMap<&'src str, u8>,
    end_line: usize,
    compiler: Compiler,
}

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    InvalidNumber,
    IncompleteExpression,
    NoSemicolonAfterValue,
    NoSemicolonAfterExpression,
    NoVariableName,
    NoSemicolonAfterVarDecl,
    InvalidAssignmentTarget,
    InvalidVariableName,
    ScanError(ScanErrorKind),
    RightBraceAfterBlock,
    TooManyLocals,
    DuplicateLocalInScope,
    LocalInOwnInitializer
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::InvalidNumber => todo!(),
            ParseErrorKind::IncompleteExpression => write!(f, "Expect expression."),
            ParseErrorKind::NoSemicolonAfterValue => write!(f, "Expect ';' after value."),
            ParseErrorKind::NoSemicolonAfterExpression => write!(f, "Expect ';' after expression."),
            ParseErrorKind::NoVariableName => write!(f, "Expect variable name."),
            ParseErrorKind::NoSemicolonAfterVarDecl => write!(f, "Expect ';' after variable declaration."),
            ParseErrorKind::InvalidAssignmentTarget => write!(f, "Invalid assignment target."),
            ParseErrorKind::InvalidVariableName => write!(f, "Expect variable name."),
            ParseErrorKind::ScanError(ScanErrorKind::UndelimitedString) =>
                write!(f, "Unterminated string."),
            ParseErrorKind::RightBraceAfterBlock => write!(f, "Expect '}}' after block."),
            ParseErrorKind::TooManyLocals => write!(f, "Too many local variables in function."),
            ParseErrorKind::DuplicateLocalInScope => write!(f, "Already a variable with this name in this scope."),
            ParseErrorKind::LocalInOwnInitializer => write!(f, "Can't read local variable in its own initializer."),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError<'src> {
    location: Option<Token<'src>>,
    line: usize,
    kind: ParseErrorKind,
}

impl<'src> fmt::Display for ParseError<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(location) => {
                match location.ttype {
                    TokenType::Error(_) => write!(f, "[line {}] Error: {}", self.line, self.kind),
                    _ => write!(f, "[line {}] Error at '{}': {}", self.line, location.span, self.kind),
                }
            }
            None => {
                write!(f, "[line {}] Error at end: {}", self.line, self.kind)
            },
        }
    }
}

enum Associativity {
    Left,
    Right,
    NonAssoc,
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
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

type Result<'src, T> = std::result::Result<T, ParseError<'src>>;

impl<'src> Parser<'src> {
    fn new(sc: Scanner<'src>) -> Self {
        let line_count = sc.source.chars().filter(|c| *c == '\n').count() + 1;
        Parser {
            scanner: sc.into_iter().peekable(),
            errors: Vec::new(),
            intern_table: HashMap::new(),
            end_line: line_count,
            compiler: Default::default(),
        }
    }

    fn precedence(ttype: TokenType) -> Option<Precedence> {
        use TokenType::*;
        match ttype {
            Plus | Minus => Some(Precedence::Term),
            Star | Slash => Some(Precedence::Factor),
            EqualEqual | BangEqual => Some(Precedence::Equality),
            Greater | GreaterEqual | Less | LessEqual => Some(Precedence::Comparison),
            Equal => Some(Precedence::Assignment),
            _ => None,
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

    fn error_end(&self, kind: ParseErrorKind) -> ParseError<'src> {
        ParseError {
            location: None,
            line: self.end_line,
            kind
        }
    }

    fn error_at(&self, location: Token<'src>, kind: ParseErrorKind) -> ParseError<'src> {
        let line = location.line;
        ParseError {
            location: Some(location),
            line,
            kind
        }
    }

    fn error_at_or_end(&self, location: Option<Token<'src>>, kind: ParseErrorKind) -> ParseError<'src> {
        match location {
            Some(location) => self.error_at(location, kind),
            None => self.error_end(kind),
        }
    }

    fn add_string(&mut self, chunk: &mut Chunk, string: &'src str) -> u8 {
        match self.intern_table.entry(string) {
            hash_map::Entry::Occupied(entry) => {
                entry.get().clone()
            },
            hash_map::Entry::Vacant(entry) => {
                let handle = GC::new_string(string);
                chunk.add_constant_value(Value::from(handle.get_object()));
                chunk.allocations.push_front(handle);
                let offset = chunk.constants.len() as u8 - 1;
                entry.insert(offset).clone()
            },
        }
    }

    fn _expression(&mut self, chunk: &mut Chunk, min_prec: Precedence) -> Result<'src, ()> {
        match self.scanner.next() {
            None => return Err(self.error_end(ParseErrorKind::IncompleteExpression)),
            Some(token) => match token.ttype {
                TokenType::Minus | TokenType::Bang => {
                    self._expression(chunk, Precedence::Unary)?;
                    let op = match token.ttype {
                        TokenType::Minus => Op::Negate,
                        TokenType::Bang => Op::Not,
                        _ => unreachable!(),
                    };
                    chunk.add_op(op, token.line);
                }
                TokenType::Number => {
                    match token.span.parse::<f64>() {
                        Ok(c) => Ok(chunk.add_constant(c.into(), token.line)),
                        _ => Err(self.error_at(token, ParseErrorKind::InvalidNumber)),
                    }?;
                }
                TokenType::String => {
                    let without_quotes = &token.span[1..(token.span.len() - 1)];
                    let offset = self.add_string(chunk, without_quotes);
                    chunk.add_op(
                        Op::Constant {
                            offset,
                        },
                        token.line
                    );
                }
                TokenType::LeftParen => {
                    self._expression(chunk, Precedence::None)?;
                    assert_eq!(self.scanner.next().unwrap().ttype, TokenType::RightParen)
                }
                TokenType::Nil => {
                    chunk.add_op(Op::Nil, token.line);
                }
                TokenType::True => {
                    chunk.add_op(Op::True, token.line);
                }
                TokenType::False => {
                    chunk.add_op(Op::False, token.line);
                }
                TokenType::Identifier => {
                    let (get_op, set_op) = match self.compiler.resolve_local(&token.span) {
                        Ok(offset) => (Op::GetLocal { offset }, Op::SetLocal { offset }),
                        Err(true) => {
                            return Err(self.error_at(token, ParseErrorKind::LocalInOwnInitializer));
                        },
                        Err(false) => {
                            let offset = self.add_string(chunk, token.span);
                            (Op::GetGlobal { offset }, Op::SetGlobal { offset })

                        }
                    };

                    if let Some(eq_token) = self.scanner.next_if(|token| token.ttype == TokenType::Equal) {
                        if min_prec <= Precedence::Assignment {
                            self._expression(chunk, Precedence::Assignment)?;
                            chunk.add_op(set_op, token.line);
                        } else {
                            return Err(self.error_at(eq_token, ParseErrorKind::InvalidAssignmentTarget));
                        }
                    } else {
                        chunk.add_op(get_op, token.line);
                    };
                }
                TokenType::Error(err) => {
                    return Err(self.error_at(token, ParseErrorKind::ScanError(err)));
                }
                _ => {
                    return Err(self.error_at(token, ParseErrorKind::IncompleteExpression));
                }
            },
        };

        while let Some(op) = self.scanner.next_if(|token| {
            if let Some(op_prec) = Self::precedence(token.ttype) {
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
            } else {
                false
            }
        }) {
            // Generates code for rhs
            self._expression(chunk, Self::precedence(op.ttype).unwrap())?;

            match op.ttype {
                TokenType::Plus => chunk.add_op(Op::Add, op.line),
                TokenType::Minus => chunk.add_op(Op::Subtract, op.line),
                TokenType::Star => chunk.add_op(Op::Multiply, op.line),
                TokenType::Slash => chunk.add_op(Op::Divide, op.line),
                TokenType::EqualEqual => chunk.add_op(Op::Equal, op.line),
                TokenType::Greater => chunk.add_op(Op::Greater, op.line),
                TokenType::Less => chunk.add_op(Op::Less, op.line),
                TokenType::BangEqual => chunk.add_op(Op::Equal, op.line).add_op(Op::Not, op.line),
                TokenType::GreaterEqual => chunk.add_op(Op::Less, op.line).add_op(Op::Not, op.line),
                TokenType::LessEqual => chunk.add_op(Op::Greater, op.line).add_op(Op::Not, op.line),
                TokenType::Equal => {return Err(self.error_at(op, ParseErrorKind::InvalidAssignmentTarget))},
                _ => unreachable!(),
            };
        }

        Ok(())
    }

    pub fn expression(&mut self, chunk: &mut Chunk) -> Result<'src, ()> {
        self._expression(chunk, Precedence::None)
    }

    fn must_consume(&mut self, expected: TokenType, error_kind: ParseErrorKind) -> Result<'src, Token<'src>> {
        match self.scanner.peek().cloned() {
            Some(token) if token.ttype == expected => Ok(self.scanner.next().unwrap()),
            Some(token) => Err(self.error_at(token.clone(), error_kind)),
            _ => Err(self.error_end(error_kind)),
        }
    }

    fn print_statement(&mut self, print_token: Token<'src>, chunk: &mut Chunk) -> Result<'src, ()> {
        self.expression(chunk)?;
        chunk.add_op(Op::Print, print_token.line);
        self.must_consume(TokenType::Semicolon, ParseErrorKind::NoSemicolonAfterValue).map(|_| ())
    }

    fn block(&mut self, chunk: &mut Chunk) -> Result<'src, ()> {
        self.compiler.enter_scope();
        loop {
            match self.scanner.peek() {
                Some(token) if token.ttype == TokenType::RightBrace => {
                    let token = self.scanner.next().unwrap();
                    let pop_count = self.compiler.exit_scope();
                    for _ in 0..pop_count {
                        chunk.add_op(Op::Pop, token.line);
                    }
                    break Ok(());
                },
                Some(_) => self.declaration(chunk),
                None => {
                    break Err(self.error_end(ParseErrorKind::RightBraceAfterBlock));
                }
            }
        }
    }

    fn expr_statement(&mut self, chunk: &mut Chunk) -> Result<'src, ()> {
        self.expression(chunk)?;
        let pop_line =
            self.must_consume(TokenType::Semicolon, ParseErrorKind::NoSemicolonAfterExpression)
                .map(|tok| tok.line)?;
        chunk.add_op(Op::Pop, pop_line);

        Ok(())
    }

    fn statement(&mut self, chunk: &mut Chunk) -> Result<'src, ()> {
        match self.scanner.peek().unwrap().ttype {
            TokenType::Print => {
                let print_token = self.scanner.next().unwrap();
                self.print_statement(print_token, chunk)
            },
            TokenType::LeftBrace => {
                self.scanner.next();
                self.block(chunk)
            },
            _ => self.expr_statement(chunk),
        }
    }

    fn synchronize(&mut self) {
        use TokenType::*;

        while let Some(peek) = self.scanner.peek() {
            if peek.ttype == TokenType::Semicolon {
                self.scanner.next();
                return;
            }

            if [Class, Fun, Var, For, If, While, Print, Return].contains(&peek.ttype) {
                return;
            }

            self.scanner.next();
        }
    }

    fn variable(&mut self) -> Result<'src, Token<'src>> {
        let ident = self.must_consume(TokenType::Identifier, ParseErrorKind::NoVariableName)?;

        if ident.span == "nil" {
            Err(self.error_at(ident, ParseErrorKind::InvalidVariableName))
        } else {
            Ok(ident)
        }
    }

    fn var_declaration(&mut self, var_token: Token<'src>, chunk: &mut Chunk) ->  Result<'src, ()> {
        let ident = self.variable()?;

        if !self.compiler.in_global_scope() {
            self.compiler.declare_local(ident.span).map_err(
                |err| match err {
                    LocalsError::TooMany => self.error_at(ident.clone(), ParseErrorKind::TooManyLocals),
                    LocalsError::DuplicateInScope => self.error_at(ident.clone(), ParseErrorKind::DuplicateLocalInScope)
                }
            )?
        }

        match self.scanner.peek() {
            Some(token) if token.ttype == TokenType::Equal => {
                self.scanner.next();
                self.expression(chunk)?;
            },
            _ => {
                chunk.add_op(Op::Nil, ident.line);
            }
        }

        if self.compiler.in_global_scope() {
            let offset = self.add_string(chunk, ident.span);
            chunk.add_op(Op::DefineGlobal { offset }, var_token.line);
        } else {
            self.compiler.mark_last_initialized();
        }

        self.must_consume(TokenType::Semicolon, ParseErrorKind::NoSemicolonAfterVarDecl)?;

        Ok(())
    }

    pub fn declaration(&mut self, chunk: &mut Chunk) {
        let peeked = self.scanner.peek().unwrap().clone();
        let result = match peeked.ttype {
            TokenType::Var => {
                self.scanner.next();
                self.var_declaration(peeked, chunk)
            },
            _ => self.statement(chunk),
        };

        if let Err(err) = result {
            self.errors.push(err);
            self.synchronize();
        }
    }

    pub fn compile(&mut self, chunk: &mut Chunk) {
        while let Some(_) = self.scanner.peek() {
            self.declaration(chunk)
        }
    }
}

#[cfg(test)]
pub fn compile_expr<'src>(source: &'src str, chunk: &mut Chunk) -> Result<'src, ()>{
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner);
    parser.expression(chunk)
}

pub fn compile<'src>(source: &'src str, chunk: &mut Chunk) -> Vec<ParseError<'src>> {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner);
    parser.compile(chunk);
    return parser.errors;
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
                    span: "print",
                    line: 1,
                },
                Token {
                    ttype: TokenType::LeftParen,
                    span: "(",
                    line: 1,
                },
                Token {
                    ttype: TokenType::Number,
                    span: "1",
                    line: 1,
                },
                Token {
                    ttype: TokenType::Plus,
                    span: "+",
                    line: 1,
                },
                Token {
                    ttype: TokenType::Number,
                    span: "2",
                    line: 1,
                },
                Token {
                    ttype: TokenType::Star,
                    span: "*",
                    line: 1,
                },
                Token {
                    ttype: TokenType::Number,
                    span: "3",
                    line: 1,
                },
                Token {
                    ttype: TokenType::RightParen,
                    span: ")",
                    line: 1,
                },
                Token {
                    ttype: TokenType::Semicolon,
                    span: ";",
                    line: 1,
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
                    span: "1",
                    line: 1,
                },
                Token {
                    ttype: TokenType::Number,
                    span: "2",
                    line: 2,
                },
                Token {
                    ttype: TokenType::Number,
                    span: "3",
                    line: 3,
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
                span: "\"hello world\"",
                line: 1,
            }]
        );

        assert_eq!(tokens[0].span, source);
    }

    fn test_parse_expression(source: &str, expected: &Chunk) {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let mut chunk = Chunk::new();
        let result = parser.expression(&mut chunk);
        assert_eq!(result, Ok(()));
        assert!(chunk.instr_eq(expected));
    }

    fn test_parse_program<'src>(source: &'src str, expected: &Chunk) {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let mut chunk = Chunk::new();
        parser.compile(&mut chunk);

        assert_eq!(parser.errors, vec![]);
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
        let result = parser.expression(&mut chunk);

        assert_eq!(result, Ok(()));
        assert_eq!(chunk.allocations.len(), 1);
        assert_eq!(chunk.constants.len(), 1);
    }

    #[test]
    fn basic_print_statement() {
        let source = "print 1 + 1;";
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![Constant { offset: 0 }, Constant { offset: 1 }, Add, Print],
            vec![1, 1, 1, 1],
            vec![Value::from(1.0), Value::from(1.0)],
            LinkedList::new(),
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn basic_print_string_statement() {
        let source = "print \"string\";";
        let allocation = GC::new_string("string");
        let object = allocation.get_object();
        let mut allocations = LinkedList::new();
        allocations.push_front(allocation);
        use crate::bc::Op::*;
        let expected = Chunk::new_with(
            vec![Constant { offset: 0 }, Print],
            vec![1, 1],
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
            vec![1, 1, 1, 1],
            vec![Value::from(1.0), Value::from(1.0)],
            LinkedList::new(),
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn basic_var_decl() {
        let source = "var x;";
        use crate::bc::Op::*;
        let x = GC::new_string("x");
        let expected = Chunk::new_with(
            vec![Nil, DefineGlobal { offset: 0 }],
            vec![1, 1],
            vec![x.get_object().into()],
            LinkedList::new(),
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn basic_var_decl_with_initializer() {
        let source = "var x = 1 + 1;";
        use crate::bc::Op::*;
        let x = GC::new_string("x");
        let expected = Chunk::new_with(
            vec![Constant {offset: 1}, Constant {offset: 2}, Add, DefineGlobal { offset: 0 }],
            vec![1, 1, 1, 1],
            vec![x.get_object().into(), Value::from(1.0), Value::from(1.0)],
            LinkedList::new(),
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn assign() {
        let source = "var x = y = z;";
        use crate::bc::Op::*;
        let x = GC::new_string("x");
        let y = GC::new_string("y");
        let z = GC::new_string("z");
        let expected = Chunk::new_with(
            vec![GetGlobal { offset: 2 }, SetGlobal { offset: 1 }, DefineGlobal { offset: 0 }],
            vec![1, 1, 1],
            vec![x.get_object().into(), y.get_object().into(), z.get_object().into()],
            LinkedList::new(),
        );

        test_parse_program(source, &expected);
    }

    #[test]
    fn block_missing_brace() {
        let source = "{ var a; ";
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let mut chunk = Chunk::new();
        parser.compile(&mut chunk);

        assert_eq!(parser.errors[0].kind, ParseErrorKind::RightBraceAfterBlock)
    }
}
