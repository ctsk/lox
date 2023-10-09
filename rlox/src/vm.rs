use crate::bc::{Chunk, Op, TraceInfo, Value};
use std::ops::Not;
use std::rc::Rc;

pub struct VM {
    pub trace: bool,
    stack: Vec<Value>,
    pc: usize,
}

#[derive(Debug, PartialEq)]
pub enum VMError {
    Compile,
    Runtime(Rc<str>, usize),
}

impl VM {
    pub fn new() -> VM {
        VM {
            trace: false,
            stack: Vec::new(),
            pc: 0,
        }
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    fn runtime_err(&self, msg: &'static str) -> VMError {
        VMError::Runtime(msg.into(), self.pc)
    }

    fn type_err(&self, expected: &'static str, found: Value) -> VMError {
        VMError::Runtime(
            format!("Expected: {:}, Found: {:?}", expected, found).into(),
            self.pc,
        )
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, VMError> {
        self.stack
            .pop()
            .ok_or(self.runtime_err("Attempt to pop of empty stack."))
    }

    fn pop_num(&mut self) -> Result<f64, VMError> {
        let top_of_stack = self.pop()?;
        top_of_stack
            .as_num()
            .ok_or(self.type_err("Number", top_of_stack))
    }

    fn pop_bool(&mut self) -> Result<bool, VMError> {
        let top_of_stack = self.pop()?;
        top_of_stack
            .as_bool()
            .ok_or(self.type_err("Boolean", top_of_stack))
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<Option<Value>, VMError> {
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
                        chunk
                    }
                );
            }

            match instr {
                Op::Return => print!("{:?}", self.pop()?),
                Op::Constant { offset } => self.push(chunk.constants[offset]),
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Negate => {
                    let new_val = -self.pop_num()?;
                    self.push(new_val.into());
                }
                Op::Not => {
                    let top_of_stack = self.pop()?;
                    let new_val = match top_of_stack {
                        Value::Nil => Ok(true),
                        Value::Bool(val) => Ok(!val),
                        _ => Err(self.type_err("Boolean or Nil", top_of_stack)),
                    }?;
                    self.push(new_val.into());
                }
                Op::Add | Op::Subtract | Op::Multiply | Op::Divide => {
                    let b = self.pop_num()?;
                    let a = self.pop_num()?;
                    let r = match instr {
                        Op::Add => a + b,
                        Op::Subtract => a - b,
                        Op::Multiply => a * b,
                        Op::Divide => a / b,
                        _ => unreachable!(),
                    };
                    self.push(r.into())
                }
                Op::Equal | Op::Greater | Op::Less => {
                    let b = self.pop()?;
                    let a  = self.pop()?;
                    let r = match instr {
                        Op::Equal => a == b,
                        Op::Greater => a > b,
                        Op::Less => a < b,
                        _ => unreachable!(),
                    };
                    self.push(r.into())
                }
            }
        }

        Ok(self
            .stack
            .is_empty()
            .not()
            .then_some(self.stack[self.stack.len() - 1]))
    }
}

#[cfg(test)]
mod tests {
    use crate::bc::Op::Equal;
    use super::{Chunk, Op, Value, VM};
    use crate::vm::VMError;

    #[test]
    fn simple_arithmetic() {
        let chunk = Chunk::new_with(
            vec![
                Op::Constant { offset: 0 },
                Op::Constant { offset: 1 },
                Op::Multiply,
                Op::Constant { offset: 2 },
                Op::Constant { offset: 3 },
                Op::Multiply,
                Op::Multiply,
                Op::Negate,
                Op::Constant { offset: 4 },
                Op::Constant { offset: 5 },
                Op::Add,
                Op::Constant { offset: 6 },
                Op::Subtract,
                Op::Negate,
                Op::Divide,
            ],
            vec![],
            vec![3., 7., 11., 17., 500., 1000., 250.]
                .into_iter()
                .map(Value::from)
                .collect(),
        );

        let mut vm = VM::new();
        vm.run(&chunk).unwrap();

        assert_eq!(vm.stack[0], Value::from(3.1416));
    }

    #[test]
    fn nil_error() {
        let chunk = Chunk::new_with(
            vec![Op::Nil, Op::Negate],
            vec![],
            vec![],
        );

        let mut vm = VM::new();
        assert_eq!(
            vm.run(&chunk).unwrap_err(),
            vm.type_err("Number", Value::Nil)
        );
    }

    #[test]
    fn simple_booleans() {
        let chunk = Chunk::new_with(
            vec![Op::False, Op::Not, Op::False, Op::Not, Op::Equal],
            vec![],
            vec![],
        );
        let mut vm = VM::new();
        vm.run(&chunk).unwrap();

        assert_eq!(vm.stack[0], true.into());
    }

    #[test]
    fn not_nil_is_true() {
        let chunk = Chunk::new_with(
            vec![Op::Nil, Op::Not],
            vec![],
            vec![],
        );
        let mut vm = VM::new();
        vm.run(&chunk).unwrap();

        assert_eq!(vm.stack[0], true.into());
    }
}
