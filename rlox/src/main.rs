mod bc;
mod lc;
mod vm;
mod gc;

use std::env;
use std::io;

use bc::Chunk;
use vm::VM;

fn repl() {
    let mut buffer = String::new();

    let do_trace = env::var("LOX_TRACE").is_ok();

    loop {
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let mut chunk = Chunk::new();
                lc::compile(buffer.as_str(), &mut chunk);
                let mut vm = VM::new();
                vm.set_trace(do_trace);
                let result = vm.run(&chunk);
                println!("{:?}", result);
                buffer.clear();
            }
            Err(error) =>
                println!("{:?}", error),
        }
    }
}

fn run_file() {
    todo!()
}

fn main() {
    let num_args = env::args().len();

    if num_args == 1 {
        repl()
    } else if num_args == 2 {
        run_file()
    } else {
        println!("Usage: rlox [path]")
    }
}

#[cfg(test)]
mod tests {
    use crate::{bc::{Chunk, Value}, gc::allocate_string, lc::compile, vm::VM};

    #[test]
    fn test_compile_and_run_pi_math() {
        let source = "-(3 * 7 * 11 * 17) / -(500 + 1000 - 250)";
        let mut chunk = Chunk::new();
        compile(source, &mut chunk);
        let mut vm = VM::new();
        vm.run(&chunk).unwrap();
    }

    #[test]
    fn string_concatenation() {
        let source = "\"hello\" + \" \" + \"world\"";
        let mut chunk = Chunk::new();
        compile(source, &mut chunk);
        let mut vm = VM::new();
        let (result, _allocs) = vm.run(&chunk).unwrap().unwrap();
        let target_alloc = unsafe { allocate_string("hello world").unwrap() };
        let target = Value::from(target_alloc.get_object());
        assert_eq!(result, target);
    }
}
