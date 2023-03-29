mod chunk;
mod bytecode;

use chunk::Chunk;
use bytecode::Op;

fn main() {
    let mut chunk = Chunk::new("TEST".to_string());
    chunk.add(Op::Return);
    println!("{:?}", chunk);
}
