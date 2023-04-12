mod vm;
mod lc;

use std::env;

fn repl() {
}

fn run_file() {

}

fn main() {

    let num_args = env::args().len();

    lc::compile("print(1+2*3)");

    if num_args == 1 {
        repl();
    } else if num_args == 2 {
        run_file();
    } else {
        println!("Usage: rlox [path]");
    }
}
