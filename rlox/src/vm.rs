use std::convert::From;
use std::fmt;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Op {
    Return,
    Constant { offset: usize },
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
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
            writeln!(
                f,
                "{:?}",
                TraceInfo {
                    offset: idx,
                    op: op,
                    chunk: &self
                }
            )?;
        }

        return Ok(());
    }
}

struct TraceInfo<'a> {
    offset: usize,
    op: Op,
    chunk: &'a Chunk,
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

pub struct VM {
    pub trace: bool,
    stack: Vec<Value>,
    pc: usize,
}

#[derive(Debug)]
pub enum VMError {
    Compile,
    Runtime,
}

impl VM {
    pub fn new() -> VM {
        VM {
            trace: false,
            stack: Vec::new(),
            pc: 0,
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, VMError> {
        self.stack.pop().ok_or(VMError::Runtime)
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), VMError> {
        while self.pc < chunk.code.len() {
            let instr = chunk.code[self.pc];
            self.pc += 1;

            if self.trace {
                print!("            [ ");
                for value in self.stack.iter() {
                    print!("{:?} | ", value);
                }
                println!("_ ]\n");

                println!(
                    "{:?}\n",
                    TraceInfo {
                        offset: self.pc - 1,
                        op: instr,
                        chunk: chunk
                    }
                );
            }

            match instr {
                Op::Return => {
                    print!("{:?}", self.pop()?);
                    return Ok(());
                }
                Op::Constant { offset } => self.push(chunk.constants[offset]),
                Op::Negate => {
                    let new_val = -self.pop()?.val;
                    self.push(Value::from(new_val));
                }
                Op::Add | Op::Subtract | Op::Multiply | Op::Divide => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let r = match instr {
                        Op::Add => a.val + b.val,
                        Op::Subtract => a.val - b.val,
                        Op::Multiply => a.val * b.val,
                        Op::Divide => a.val / b.val,
                        _ => unreachable!()
                    };
                    self.push(Value::from(r))
                }
            }
        }

        return Ok(());
    }
}
