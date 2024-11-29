mod bc;
mod lc;
mod vm;
mod gc;

use std::env;
use std::fs;
use std::io;
use std::process::ExitCode;

use bc::Chunk;
use vm::VM;


fn compile_and_run(source: &str, do_trace: bool) -> ExitCode {
    let mut chunk = Chunk::new();
    let errors = lc::compile(source, &mut chunk);

    if errors.is_empty() {
        let mut vm = VM::new();
        vm.set_trace(do_trace);
        if let Err(err) = vm.stdrun(&chunk) {
            eprintln!("{}", err);
            ExitCode::from(70)
        } else {
            ExitCode::from(0)
        }

    } else {
        for error in errors {
            eprintln!("{}", error);
        }
        ExitCode::from(65)
    }
}


fn repl() {
    let mut buffer = String::new();

    let do_trace = env::var("LOX_TRACE").is_ok();

    loop {
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                compile_and_run(buffer.as_str(), do_trace);
                buffer.clear();
            }
            Err(error) => println!("{:?}", error),
        }
    }
}

fn run_file(path: String) -> ExitCode {
    let do_trace = env::var("LOX_TRACE").is_ok();
    let source = fs::read_to_string(path).unwrap();
    compile_and_run(source.as_str(), do_trace)
}

fn main() -> ExitCode {
    let num_args = env::args().len();

    if num_args == 1 {
        repl();
        ExitCode::SUCCESS
    } else if num_args == 2 {
        let source = env::args().nth(1).unwrap();
        run_file(source)
    } else {
        println!("Usage: rlox [path]");
        ExitCode::from(64)
    }
}
