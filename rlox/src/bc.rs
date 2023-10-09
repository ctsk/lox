use crate::bc::Value::{Bool, Number};
use std::convert::From;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Return,
    Constant { offset: usize },
    Nil,
    True,
    False,
    Not,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Greater,
    Less,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

impl Value {
    pub fn as_num(self) -> Option<f64> {
        match self {
            Number(val) => Some(val),
            _ => None,
        }
    }

    pub fn as_bool(self) -> Option<bool> {
        match self {
            Bool(val) => Some(val),
            _ => None,
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Number(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Bool(value)
    }
}

pub struct Chunk {
    pub code: Vec<Op>,
    pub debug_info: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            debug_info: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn new_with(code: Vec<Op>, debug_info: Vec<usize>, constants: Vec<Value>) -> Self {
        Chunk {
            code,
            debug_info,
            constants,
        }
    }

    pub fn instr_eq(&self, other: &Chunk) -> bool {
        self.code == other.code && self.constants == other.constants
    }

    pub fn add_op(&mut self, op: Op, line: usize) -> &mut Self {
        self.code.push(op);
        self.debug_info.push(line);

        self
    }

    pub fn add_constant(&mut self, value: Value, line: usize) -> &mut Self {
        self.constants.push(value);
        self.add_op(
            Op::Constant {
                offset: self.constants.len() - 1,
            },
            line,
        )
    }
}

pub struct NamedChunk {
    name: String,
    chunk: Chunk,
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "-*-*- Chunk @ {:p} -*-*-", self)?;
        for (idx, op) in self.code.iter().copied().enumerate() {
            writeln!(
                f,
                "{:?}",
                TraceInfo {
                    offset: idx,
                    op,
                    chunk: self,
                }
            )?;
        }

        Ok(())
    }
}

impl fmt::Debug for NamedChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "-*-*- {} -*-*-", self.name)?;
        write!(f, "{:?}", self.chunk)
    }
}

pub struct TraceInfo<'a> {
    pub offset: usize,
    pub op: Op,
    pub chunk: &'a Chunk,
}

impl fmt::Debug for TraceInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let chunk = self.chunk;
        let op = self.op;
        let offset = self.offset;

        write!(f, "{:04}  ", offset)?;

        let line = chunk.debug_info[offset];

        if offset > 0 && chunk.debug_info[offset - 1] == line {
            write!(f, "   |  ")
        } else {
            write!(f, "{:4}  ", line)
        }?;

        match op {
            Op::Constant { offset } => {
                f.debug_struct("Constant")
                    .field("val", &chunk.constants[offset])
                    .finish()?;
                write!(f, "")
            }
            _ => write!(f, "{:?}", op)
        }
    }
}
