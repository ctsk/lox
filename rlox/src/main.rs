mod vm;

fn main() {
    let mut chunk = vm::Chunk::new("TEST".to_string());
    chunk.add_constant(vm::Value::from(3.));
    chunk.add_constant(vm::Value::from(7.));
    chunk.add_constant(vm::Value::from(11.));
    chunk.add_constant(vm::Value::from(17.));
    chunk.add_constant(vm::Value::from(500.));
    chunk.add_constant(vm::Value::from(1000.));
    chunk.add_constant(vm::Value::from(250.));


    chunk.add_op(vm::Op::Constant { offset: 0 }, 1);
    chunk.add_op(vm::Op::Constant { offset: 1 }, 1);
    chunk.add_op(vm::Op::Multiply, 1);
    chunk.add_op(vm::Op::Constant { offset: 2 }, 1);
    chunk.add_op(vm::Op::Constant { offset: 3 }, 1);
    chunk.add_op(vm::Op::Multiply, 1);
    chunk.add_op(vm::Op::Multiply, 1);
    chunk.add_op(vm::Op::Negate, 1);
    chunk.add_op(vm::Op::Constant { offset: 4 }, 2);
    chunk.add_op(vm::Op::Constant { offset: 5 }, 2);
    chunk.add_op(vm::Op::Add, 2);
    chunk.add_op(vm::Op::Constant { offset: 6 }, 2);
    chunk.add_op(vm::Op::Subtract, 2);
    chunk.add_op(vm::Op::Negate, 2);
    chunk.add_op(vm::Op::Divide, 2);
    chunk.add_op(vm::Op::Return, 3);

    println!("{:?}", chunk);

    let mut interpreter = vm::VM::new();
    interpreter.trace = true;
    interpreter.interpret(&chunk).unwrap()
}
