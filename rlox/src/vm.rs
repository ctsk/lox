use crate::bc::{Chunk, Op, TraceInfo, Value};
use crate::gc::{GcHandle, ObjString, ObjectType, GC};
use std::collections::{hash_map, HashMap, LinkedList};
use std::{fmt, io};

pub struct VM {
    pub trace: bool,
    stack: Vec<Value>,
    pc: usize,
    line: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VMError {
    line: usize,
    kind: VMErrorKind,
    msg: Option<String>,
}

impl VMError {
    fn new(kind: VMErrorKind, line: usize) -> Self {
        VMError { line, kind, msg: None }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VMErrorKind {
    InvalidAddOperands,
    InvalidMathOperands,
    InvalidMathOperand,
    UndefinedVariable,
    PopFromEmptyStack,
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            VMErrorKind::InvalidAddOperands =>
                {write!(f, "Operands must be two numbers or two strings.")?; }
            VMErrorKind::InvalidMathOperands =>
                {write!(f, "Operands must be numbers.")?; }
            VMErrorKind::InvalidMathOperand =>
                {write!(f, "Operand must be a number.")?; }
            _ => {}
        };

        if let Some(msg) = &self.msg {
            write!(f, "{}", msg)?
        }

        write!(f, "\n[line {}]", self.line)
    }
}

type Result<T> = std::result::Result<T, VMError>;

impl VM {
    pub fn new() -> VM {
        VM {
            trace: false,
            stack: Vec::new(),
            pc: 0,
            line: 0,
        }
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn err(&mut self, kind: VMErrorKind) -> VMError {
        VMError::new(
            kind,
            self.line
        )
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or(self.err(VMErrorKind::PopFromEmptyStack))
    }

    fn pop_num(&mut self) -> Result<f64> {
        let top_of_stack = self.pop()?;
        top_of_stack
            .as_num()
            .ok_or(self.err(VMErrorKind::InvalidMathOperand))
    }

    fn pop_nums(&mut self) -> Result<(f64, f64)> {
        let a = self.pop()?;
        let b = self.pop()?;
        match (a.as_num(), b.as_num()) {
            (Some(a), Some(b)) => Ok((a, b)),
            _ => Err(self.err(VMErrorKind::InvalidMathOperands))
        }
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
        let mut globals: HashMap<ObjString, Value> = HashMap::new();

        let err = |kind: VMErrorKind, msg: Option<String>| -> VMError {
            VMError {
                line: chunk.debug_info[self.pc - 1],
                kind,
                msg
            }
        };

        while self.pc < chunk.code.len() {
            let instr = chunk.code[self.pc];
            self.line = chunk.debug_info[self.pc];
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
                        _ => Err(self.err(VMErrorKind::InvalidMathOperand)),
                        //_ => Err(self.type_err("Boolean or Nil", top_of_stack)),
                    }?;
                    self.push(new_val.into());
                }
                Op::Add => {
                    let b = self.pop()?;
                    match b {
                        Value::Number(num) => {
                            let a = self.pop_num().or(Err(self.err(VMErrorKind::InvalidAddOperands)))?;
                            self.push(Value::from(num + a));
                            Ok(())
                        }
                        Value::Obj(b) => match b.get_otype() {
                            ObjectType::String => {
                                let a = self.pop()?;
                                match a {
                                    Value::Obj(a) => match a.get_otype() {
                                        ObjectType::String => {
                                            let (a, b) = (a.downcast().unwrap(), b.downcast().unwrap());
                                            let new_obj = GC::new_concat_string(a, b);
                                            self.push(Value::from(new_obj.get_object()));
                                            allocations.push_front(new_obj);
                                            Ok(())
                                        }
                                    },
                                    _ => Err(self.err(VMErrorKind::InvalidAddOperands)),
                                }
                            }
                        },
                        _ => Err(self.err(VMErrorKind::InvalidAddOperands)),
                    }?
                }
                Op::Subtract | Op::Multiply | Op::Divide => {
                    let (b, a) = self.pop_nums()?;
                    let r = match instr {
                        Op::Subtract => a - b,
                        Op::Multiply => a * b,
                        Op::Divide => a / b,
                        _ => unreachable!(),
                    };
                    self.push(r.into())
                }
                Op::Greater | Op::Less => {
                    let (b, a) = self.pop_nums()?;
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
                    writeln!(output, "{}", value).unwrap()
                },
                Op::Pop => {
                    self.pop()?;
                },
                Op::DefineGlobal { offset } => {
                    let ident = chunk.constants[offset as usize].clone();
                    if let Value::Obj(name) = ident {
                        let name = name.downcast::<ObjString>().unwrap();
                        globals.entry(name).insert_entry(self.pop()?);
                    } else {
                        unreachable!()
                    };
                },
                Op::GetGlobal { offset } => {
                    let ident = match chunk.constants[offset as usize] {
                        Value::Obj(object) => object.downcast::<ObjString>().unwrap(),
                        _ => todo!(),
                    };

                    if let Some(value) = globals.get(&ident) {
                        self.push(value.clone());

                        Ok(())
                    } else {
                        Err(VMError {
                            line: self.line,
                            kind: VMErrorKind::UndefinedVariable,
                            msg: Some(format!("Undefined variable '{}'.", ident)),
                        })
                    }?
                },
                Op::SetGlobal { offset } => {
                    let ident = match chunk.constants[offset as usize] {
                        Value::Obj(object) => object.downcast::<ObjString>().unwrap(),
                        _ => todo!(),
                    };

                    match globals.entry(ident) {
                        hash_map::Entry::Occupied(mut entry) => {
                            let v = self.stack.last().unwrap();
                            entry.insert(v.clone());
                            Ok(())
                        },
                        hash_map::Entry::Vacant(_) => {
                            Err(VMError {
                                line: self.line,
                                kind: VMErrorKind::UndefinedVariable,
                                msg: Some(format!("Undefined variable '{}'.", ident)),
                            })
                        },
                    }?
                },
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::LinkedList;

    use crate::{gc::GC, vm::VMErrorKind};

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
            vec![1; 15],
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
        let chunk = Chunk::new_with(vec![Op::Nil, Op::Negate], vec![1; 2], vec![], LinkedList::new());

        let mut vm = VM::new();
        assert_eq!(
            vm.stdrun(&chunk).unwrap_err().kind,
            VMErrorKind::InvalidMathOperand
        );
    }

    #[test]
    fn simple_booleans() -> Result<(), VMError> {
        let chunk = Chunk::new_with(
            vec![Op::False, Op::Not, Op::False, Op::Not, Op::Equal],
            vec![1; 5],
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
        let chunk = Chunk::new_with(vec![Op::Nil, Op::Not], vec![1; 2], vec![], LinkedList::new());

        let mut vm = VM::new();
        vm.stdrun(&chunk)?;

        assert_eq!(vm.stack, vec![Value::Bool(true)]);

        Ok(())
    }

    #[test]
    fn define_read_globals() -> Result<(), VMError> {
        let var = GC::new_string("global");
        use Op::*;
        let chunk = Chunk::new_with(
            vec![
                Constant { offset: 0 },
                DefineGlobal { offset: 1 },
                Constant { offset: 2 },
                GetGlobal { offset: 1 },
                Multiply,
            ],
            vec![1; 5],
            vec![Value::from(5.0), Value::from(var.get_object()), Value::from(6.0)],
            LinkedList::new()
        );

        let mut vm = VM::new();
        vm.stdrun(&chunk)?;

        assert_eq!(vm.stack, vec![Value::Number(30.0)]);

        Ok(())
    }

    #[test]
    fn define_write_read_globals() -> Result<(), VMError> {
        let var = GC::new_string("global");
        use Op::*;
        let chunk = Chunk::new_with(
            vec![
                Constant { offset: 0 },
                DefineGlobal { offset: 1 },
                GetGlobal { offset: 1 },
                Constant { offset: 2 },
                Add,
                SetGlobal { offset: 1 },
                Pop,
                GetGlobal { offset: 1 },
            ],
            vec![1; 8],
            vec![Value::from(5.0), Value::from(var.get_object()), Value::from(6.0)],
            LinkedList::new()
        );

        let mut vm = VM::new();
        vm.stdrun(&chunk)?;

        assert_eq!(vm.stack, vec![Value::Number(11.0)]);

        Ok(())
    }
}
