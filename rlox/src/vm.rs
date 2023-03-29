use std::convert::From;
use std::fmt;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Op {
    Return,
    Constant { offset: usize },
}

#[derive(Debug)]
pub struct Value {
    val: f64,
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value { val: value }
    }
}

pub struct Chunk {
    code: Vec<Op>,
    name: String,
    debug_info: Vec<usize>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Chunk {
            code: Vec::new(),
            name: name,
            debug_info: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_op(&mut self, op: Op, line: usize) {
        self.code.push(op);
        self.debug_info.push(line);

    }

    pub fn add_constant(&mut self, value: Value) {
        self.constants.push(value);
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "-*-*- {} -*-*-", self.name)?;
        for (idx, op) in self.code.iter().copied().enumerate() {
            write!(f, "{:04}  ", idx)?;

            let line = self.debug_info[idx];

            if idx > 0 && self.debug_info[idx-1] == line {
                write!(f, "   |  ")?;
            } else {
                write!(f, "{:4}  ", line)?;
            }

            match op {
                Op::Return => writeln!(f, "{:?}", op),
                Op::Constant { offset } =>
                    f.debug_struct("Constant")
                     .field("val", &self.constants[offset].val)
                     .finish(),
            }?;
        }

        return Ok(());
    }
}
