use crate::bc::{Chunk, Op, TraceInfo, Value};
use crate::gc::{concat_string, GcHandle, ObjectType};
use std::collections::LinkedList;
use std::io;
use std::rc::Rc;

pub struct VM {
    pub trace: bool,
    stack: Vec<Value>,
    pc: usize,
}

#[derive(Debug, PartialEq)]
pub enum VMError {
    // Compile,
    Runtime(Rc<str>, usize),
}

type Result<T> = std::result::Result<T, VMError>;

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

    fn pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or(self.runtime_err("Attempt to pop of empty stack."))
    }

    fn pop_num(&mut self) -> Result<f64> {
        let top_of_stack = self.pop()?;
        top_of_stack
            .as_num()
            .ok_or(self.type_err("Number", top_of_stack))
    }

    pub fn stdrun(
        &mut self,
        chunk: &Chunk,
    ) -> Result<()> {
        self.run(chunk, &mut io::stdout())
    }

    pub fn run<Output: io::Write>(
        &mut self,
        chunk: &Chunk,
        output: &mut Output,
    ) -> Result<()> {
        let mut allocations: LinkedList<GcHandle> = LinkedList::new();

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
                Op::Constant { offset } => self.push(chunk.constants[offset as usize].clone()),
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
                Op::Add => {
                    let b = self.pop()?;
                    match b {
                        Value::Number(num) => {
                            let a = self.pop_num()?;
                            self.push(Value::from(num + a));
                        }
                        Value::Obj(b) => match b.get_otype() {
                            ObjectType::String => {
                                let a = self.pop()?;
                                match a {
                                    Value::Obj(a) => match a.get_otype() {
                                        ObjectType::String => {
                                            let new_obj = unsafe { concat_string(a, b).unwrap() };
                                            self.push(Value::from(new_obj.get_object()));
                                            allocations.push_front(new_obj);
                                            Ok(())
                                        }
                                    },
                                    _ => Err(self.type_err("String", a)),
                                }?
                            }
                        },
                        _ => {
                            return Err(VMError::Runtime(
                                "Operands of + need to be numbers or strings".into(),
                                self.pc,
                            ))
                        }
                    };
                }
                Op::Subtract | Op::Multiply | Op::Divide => {
                    let b = self.pop_num()?;
                    let a = self.pop_num()?;
                    let r = match instr {
                        Op::Subtract => a - b,
                        Op::Multiply => a * b,
                        Op::Divide => a / b,
                        _ => unreachable!(),
                    };
                    self.push(r.into())
                }
                Op::Greater | Op::Less => {
                    let b = self.pop_num()?;
                    let a = self.pop_num()?;
                    let r = match instr {
                        Op::Greater => a > b,
                        Op::Less => a < b,
                        _ => unreachable!(),
                    };
                    self.push(r.into())
                }
                Op::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let r = a == b;
                    self.push(r.into())
                }
                Op::Print => {
                    let value = self.pop()?;
                    writeln!(output, "{}", value)
                        .map_err(|_| VMError::Runtime("Failed to print".into(), self.pc))?
                },
                Op::Pop => {
                    self.pop()?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::LinkedList;

    use super::{Chunk, Op, VMError, Value, VM};

    #[test]
    fn simple_arithmetic() -> Result<(), VMError>{
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
            LinkedList::new(),
        );

        let mut vm = VM::new();
        vm.stdrun(&chunk)?;

        let tos = vm.stack.last().unwrap();

        assert_eq!(*tos, 3.1416.into());

        Ok(())
    }

    #[test]
    fn nil_error() {
        let chunk = Chunk::new_with(vec![Op::Nil, Op::Negate], vec![], vec![], LinkedList::new());

        let mut vm = VM::new();
        assert_eq!(
            vm.stdrun(&chunk).unwrap_err(),
            vm.type_err("Number", Value::Nil)
        );
    }

    #[test]
    fn simple_booleans() -> Result<(), VMError> {
        let chunk = Chunk::new_with(
            vec![Op::False, Op::Not, Op::False, Op::Not, Op::Equal],
            vec![],
            vec![],
            LinkedList::new(),
        );

        let mut vm = VM::new();
        vm.stdrun(&chunk)?;

        assert_eq!(vm.stack, vec![Value::Bool(true)]);

        Ok(())
    }

    #[test]
    fn not_nil_is_true() -> Result<(), VMError>{
        let chunk = Chunk::new_with(vec![Op::Nil, Op::Not], vec![], vec![], LinkedList::new());

        let mut vm = VM::new();
        vm.stdrun(&chunk)?;

        assert_eq!(vm.stack, vec![Value::Bool(true)]);

        Ok(())
    }
}
