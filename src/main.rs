mod interpreter;

use interpreter::Interpreter;
use object::PSObject;
use stack::Stack;

// bring in rust's io
use std::io::{self, Write};

fn main() {
    // start our interpreter
    let mut my_interpreter = Interpreter::new();
    // basic loop to act as a repl
    loop {
        print!("ps> ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        if io::stdin().read_line(&mut buf).unwrap() == 0 {
            break;
        }
        let line = buf.trim();

        if line.is_empty() {
            continue;
        }

        if line == "quit" {
            break;
        }

        // run the code
        my_interpreter.run(line);
    }
    /* debug stack: create our stack
    let mut my_stack = Stack::new();
    */
    /* debug statement: stack push and pop
    my_stack.push(PSObject::Int(42));
    let mut s = my_stack.peek();
    // :? tells the compiler to print values with the debug trait
    println!("Curr stack before pop: {:?}", s);
    my_stack.pop();
    s = my_stack.peek();
    println!("Curr stack after pop: {:?}", s);
    */

    /* debug statement: stack exch
    my_stack.push(PSObject::Int(42));
    my_stack.push(PSObject::Int(52));
    let mut s = my_stack.peek();
    println!("Stack peek (before exch): {:?}", s);
    my_stack.exch();
    s = my_stack.peek();
    println!("Stack peek (after exch): {:?}", s);
     */

    /* debug statement: stack dup
    my_stack.push(PSObject::Int(42));
    my_stack.dup();
    while my_stack.peek().is_some() {
        println!("Stack: {:?}", my_stack.peek());
        my_stack.pop();
    }
     */

    /* debug statement: stack count and pop
    my_stack.push(PSObject::Int(42));
    my_stack.push(PSObject::Int(42));
    my_stack.push(PSObject::Int(42));
    my_stack.push(PSObject::Int(42));
    println!("Stack Count(before clear): {:?}", my_stack.count());
    my_stack.clear();
    println!("Stack Count(after clear): {:?}", my_stack.count());
     */

    /* debug statement: copy
    my_stack.push(PSObject::Int(1));
    my_stack.push(PSObject::Int(2));
    my_stack.push(PSObject::Int(3));
    my_stack.push(PSObject::Int(4));
    my_stack.copy(3);
    my_stack.print_all();
    */
}
