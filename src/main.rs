use aheui_core::{engines, Env, OwnedCode, VM};
use std::io::Write;

fn main() {
    let input_file = std::env::args().nth(1).unwrap();
    let input = std::fs::read_to_string(input_file).unwrap();
    let code = OwnedCode::parse(&input);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    let env = Env::new(&code, &mut input, &mut output);
    let result = VM::new(env, engines::Interpreter).execute();
    output.flush().unwrap();
    std::process::exit(result);
}
