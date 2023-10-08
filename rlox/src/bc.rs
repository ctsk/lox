use std::convert::From;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Return,
    Constant { offset: usize },
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Value {
    pub val: f64,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.val)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value { val: value }
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
            constants
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
        self.add_op(Op::Constant {offset: self.constants.len() - 1}, line)
    }
}

pub struct NamedChunk {
    name: String,
    chunk: Chunk
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
                    chunk: self
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
            Op::Return | Op::Negate | Op::Add | Op::Subtract | Op::Multiply | Op::Divide => {
                write!(f, "{:?}", op)
            }
            Op::Constant { offset } => {
                f.debug_struct("Constant")
                    .field("val", &chunk.constants[offset].val)
                    .finish()?;
                write!(f, "")
            }
        }
    }
}
