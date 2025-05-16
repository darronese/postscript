mod interpreter;

use interpreter::interpreter::Interpreter;
use interpreter::object::PSObject;
use interpreter::stack::Stack;
use wasm_bindgen::prelude::*;

use std::io::{self, Write};

#[wasm_bindgen]
pub fn run_interpreter(input: &str) -> String {
    let mut my_interpreter = Interpreter::new();
    // Run the PostScript command
    my_interpreter.run(input);

    "Execution finished".to_string()
}

// Main function is still needed to build the WASM, but won't be used as entry
fn main() {
    println!("Rust WASM entry point!");
}
