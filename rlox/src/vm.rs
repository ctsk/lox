use crate::bc::{Chunk, Op, TraceInfo, Value};

pub struct VM {
    pub trace: bool,
    stack: Vec<Value>,
    pc: usize,
}

#[derive(Debug)]
pub enum VMError {
    Compile,
    Runtime(&'static str, usize),
}

impl VM {
    pub fn new() -> VM {
        VM {
            trace: false,
            stack: Vec::new(),
            pc: 0,
        }
    }

    fn runtime_err(&self, msg: &'static str) -> VMError {
        return VMError::Runtime(msg, self.pc);
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, VMError> {
        self.stack.pop().ok_or_else(|| self.runtime_err("Attempt to pop of empty stack."))
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<(), VMError> {
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
                Op::Return => print!("{:?}", self.pop()?),
                Op::Constant { offset } => self.push(chunk.constants[offset]),
                Op::Negate => {
                    let new_val = -self.pop()?.val;
                    self.push(Value::from(new_val));
                }
                Op::Add | Op::Subtract | Op::Multiply | Op::Divide => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let r = match instr {
                        Op::Add => Ok(a.val + b.val),
                        Op::Subtract => Ok(a.val - b.val),
                        Op::Multiply => Ok(a.val * b.val),
                        Op::Divide => Ok(a.val / b.val),
                        _ => Err(self.runtime_err("Op not implemented"))
                    }?;
                    self.push(r.into())
                }
            }
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::{Chunk, Op, Value, VM};

    #[test]
    fn simple_arithmetic() {
        let mut chunk = Chunk::new();
        chunk.add_constant(Value::from(3.));
        chunk.add_constant(Value::from(7.));
        chunk.add_constant(Value::from(11.));
        chunk.add_constant(Value::from(17.));
        chunk.add_constant(Value::from(500.));
        chunk.add_constant(Value::from(1000.));
        chunk.add_constant(Value::from(250.));

        chunk.add_op(Op::Constant { offset: 0 }, 1);
        chunk.add_op(Op::Constant { offset: 1 }, 1);
        chunk.add_op(Op::Multiply, 1);
        chunk.add_op(Op::Constant { offset: 2 }, 1);
        chunk.add_op(Op::Constant { offset: 3 }, 1);
        chunk.add_op(Op::Multiply, 1);
        chunk.add_op(Op::Multiply, 1);
        chunk.add_op(Op::Negate, 1);
        chunk.add_op(Op::Constant { offset: 4 }, 2);
        chunk.add_op(Op::Constant { offset: 5 }, 2);
        chunk.add_op(Op::Add, 2);
        chunk.add_op(Op::Constant { offset: 6 }, 2);
        chunk.add_op(Op::Subtract, 2);
        chunk.add_op(Op::Negate, 2);
        chunk.add_op(Op::Divide, 2);

        let mut vm = VM::new();
        vm.run(&chunk).unwrap();

        assert_eq!(vm.stack[0], Value::from(3.1416));
    }
}
