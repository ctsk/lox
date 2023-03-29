mod vm;

fn main() {
    let mut chunk = vm::Chunk::new("TEST".to_string());
    chunk.add(vm::Op::Return);
    println!("{:?}", chunk);
}
