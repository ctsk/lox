use crate::gc::{GcHandle, Object};
use std::collections::LinkedList;
use std::convert::From;
use std::fmt;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
    Return,
    Constant { offset: u8 },
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

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(Object),
}

impl Value {
    pub fn as_num(&self) -> Option<f64> {
        match self {
            &Value::Number(val) => Some(val),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            &Value::Bool(val) => Some(val),
            _ => None,
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Value::Obj(value)
    }
}

pub struct Chunk {
    pub code: Vec<Op>,
    pub debug_info: Vec<usize>,
    pub constants: Vec<Value>,
    pub allocations: LinkedList<GcHandle>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            debug_info: Vec::new(),
            constants: Vec::new(),
            allocations: LinkedList::new()
        }
    }

    pub fn new_with(code: Vec<Op>, debug_info: Vec<usize>, constants: Vec<Value>, allocations: LinkedList<GcHandle>) -> Self {
        Chunk {
            code,
            debug_info,
            constants,
            allocations
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
                offset: self.constants.len() as u8 - 1,
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
                    .field("val", &chunk.constants[offset as usize])
                    .finish()?;
                write!(f, "")
            }
            _ => write!(f, "{:?}", op)
        }
    }
}

mod tests {

    #[test]
    fn string_value_equality() {
        use crate::gc::allocate_string;
        use crate::bc::Value;

        let s1 = "bla5";
        let s2 = "bla6";

        unsafe {
            let o1 = allocate_string(s1).unwrap();
            let o2 = allocate_string(s2).unwrap();
            let o3 = allocate_string(s2).unwrap();
            let v1 = Value::from(o1.get_object());
            let v2 = Value::from(o2.get_object());
            let v3 = Value::from(o3.get_object());
            let v4 = v2.clone();

            assert_ne!(v1, v2);
            assert_eq!(v2, v3);
            assert_eq!(v2, v4);
        }
    }
}
