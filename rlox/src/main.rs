mod bc;
mod lc;
mod vm;

use std::env;

fn repl() {

}

fn run_file() {

}

fn main() {

    let num_args = env::args().len();
    let mut chunk = bc::Chunk::new();

    lc::compile("print(1+2*3)", &mut chunk);

    if num_args == 1 {
        repl();
    } else if num_args == 2 {
        run_file();
    } else {
        println!("Usage: rlox [path]");
    }
}

#[cfg(test)]
mod tests {
    use crate::{bc::Chunk, lc::compile, vm::VM};

    #[test]
    fn test_compile_and_run_pi_math() {
        let source = "-(3 * 7 * 11 * 17) / -(500 + 1000 - 250)";
        let mut chunk = Chunk::new();
        compile(source, &mut chunk);
        let mut vm = VM::new();
        vm.run(&chunk).unwrap();
    }
}
