use std::convert::From;
use std::fmt;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Op {
    Return,
    Constant { offset: usize },
}

#[derive(Copy, Clone)]
pub struct Value {
    val: f64,
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
                write!(f, "   |  ")
            } else {
                write!(f, "{:4}  ", line)
            }?;

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

const VM_STACK_SIZE: usize = 256;

struct VM {
    trace: bool,
    stack: Vec<Value>,
    code: Chunk
}

enum VMError {
    Compile,
    Runtime
}

impl VM {
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), VMError> {
        for instr in chunk.code.iter().copied() {
            if self.trace {
                print!("        [");
                for value in self.stack.iter() {
                    println!("{:?} | ", value);
                }
                println!("_ ]");

                println!("{:?}", instr);
            }

            match instr {
                Op::Return => {
                    print!("{:?}", self.pop());
                    return Ok(())
                },
                Op::Constant { offset } => {
                    self.push(self.code.constants[offset])
                }
            }
        }

        return Ok(())
    }
}
