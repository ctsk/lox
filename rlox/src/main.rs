mod vm;

fn main() {
    let mut chunk = vm::Chunk::new("TEST".to_string());
    chunk.add_op(vm::Op::Return, 1);
    chunk.add_op(vm::Op::Constant { offset: 0 }, 1);
    chunk.add_constant(vm::Value::from(3.14));
    println!("{:?}", chunk);
}
