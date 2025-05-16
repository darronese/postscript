// import from PSObject so our functions can modify it
use crate::interpreter::object::PSObject;
use std::slice::Iter;

// our stack will contain it's own stack of vector objects
// allow for clones of this stack to happen (necessary for some types)
#[derive(Clone)]
pub struct Stack {
    // holds ur actual stack
    // this post says linked list is always worse, so i use vec: https://www.reddit.com/r/rust/comments/qpmue5/question_should_i_use_a_vec_or_a_linkedlist/
    stack: Vec<PSObject>,
}

impl Stack {
    // constructor
    pub fn new() -> Self {
        Stack { stack: Vec::new() }
    }

    // allows iteration for the stack
    pub fn iter(&self) -> Iter<'_, PSObject> {
        return self.stack.iter();
    }
    // helper function push to add objects into stack
    pub fn push(&mut self, val: PSObject) {
        self.stack.push(val);
    }
    // helper function to see the top of stack
    // we have to borrow the object and not take ownership, so we reference it with &
    pub fn peek(&self) -> Option<&PSObject> {
        // since last returns an option, we don't have to worry about stack being empty it will
        // return none or some
        return self.stack.last();
    }
    // helper function to print all of stack
    pub fn print_all(&self) {
        for n in &self.stack {
            println!("{:?}", n);
        }
    }
    // pop will be a function type that returns a bool (success indicator)
    // should i use &mut or just self?
    pub fn pop(&mut self) -> Option<PSObject> {
        // since pop will return an option (either Some<t> or none), we don't need the logic to
        // handle empty stacks
        return self.stack.pop();
    }
    // exch will be a function type that swaps the first two elements within the stack and returns
    // a bool (success indicator)
    pub fn exch(&mut self) -> bool {
        // check the length is valid first
        if self.stack.len() < 2 {
            return false;
        }
        let i = self.stack.len();
        // swap the first two elements in stack
        self.stack.swap(i - 1, i - 2);
        return true;
    }
    // dup will duplicate the top of the stack and returns a bool (success indicator)
    pub fn dup(&mut self) -> bool {
        // duplicate top value of stack
        let v = self.peek().cloned();
        // check if v is none
        if v.is_none() {
            return false;
        }
        // push in duplicated value
        self.push(v.unwrap());
        return true;
    }
    // copy will take n integer operands as a parameter and create and set n elements in the
    // operand stack
    // they are pushed in the same order they originall appear,
    // i.e [1,2,3,4], 3 copy => [1,2,3,4,2,3,4]
    pub fn copy(&mut self, mut n: i32) -> bool {
        // make a copy of the stack
        let mut cp_stack = self.clone();
        // check if there are even n values in stack
        if (self.count() < n) {
            return false;
        }
        // get n copies
        // doesnt work since we would be borrowing the same object twice: let n_copies = &self.stack[self.count() as usize - n as usize..];
        // so first, get n count
        let n_index = self.count() as usize - n as usize;
        let n_copies = &self.stack[n_index..].to_vec();
        // apend the copies to our stack
        self.stack.extend_from_slice(&n_copies);
        return true;
    }
    // clear will discard all elements of the stack
    pub fn clear(&mut self) {
        while self.peek().is_some() {
            self.pop();
        }
    }
    // count will count the elements of the stack and pushed as a new element (returning)
    pub fn count(&mut self) -> i32 {
        // clone a copy and iterate through that stack
        // clone the entire stack, not just the vector object i.e not self.stack.clone, but
        // self.clone
        let mut cp_stack = self.clone();
        let mut count = 0;
        while cp_stack.peek().is_some() {
            count += 1;
            cp_stack.pop();
        }
        return count;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::object::PSObject;
    use std::collections::HashMap;

    #[test]
    fn test_push() {
        // create a stack
        let mut s = Stack::new();
        s.push(PSObject::Int(32));
        s.push(PSObject::Int(42));
        // throw error if empty
        let val = s.peek().expect("Stack is empty, push failed!");

        /* check int */
        match val {
            PSObject::Int(n) => assert_eq!(*n, 42),
            _ => panic!("Top of stack was not integer value!"),
        }

        /* check bool */
        let temp: bool = false;
        s.push(PSObject::Bool(temp));
        let val2 = s.peek().expect("Stack is empty, push failed!");
        match val2 {
            PSObject::Bool(n) => {
                assert_eq!(*n, false);
            }
            _ => {
                panic!("Top of stack was not of boolean value!");
            }
        }
        println!("Actual value {:?}", s.peek());

        let temp = String::from("cool");
        s.push(PSObject::String(temp));

        let val3 = s.peek().expect("Stack is empty, push failed!");
        /* check string */
        match val3 {
            PSObject::String(n) => assert_eq!(*n, String::from("cool")),
            _ => panic!("Top of stack was not of string value value!"),
        }
        /* check dict */
        /* check array */
    }

    #[test]
    fn test_pop() {
        // create a stack
        let mut s = Stack::new();
        s.push(PSObject::Int(42));
        s.push(PSObject::Int(32));
        s.pop();
        let val = s.peek().expect("Stack is empty, push error");
        match val {
            PSObject::Int(n) => assert_eq!(*n, 42),
            _ => panic!("Top of stack is not correct value, pop failed!"),
        }
    }
    #[test]
    fn test_copy() {
        // create a stack
        let mut s = Stack::new();
        s.push(PSObject::Int(1));
        s.push(PSObject::Int(2));
        s.push(PSObject::Int(3));
        s.copy(2);
        s.pop();
        s.print_all();
        let val = s.peek().expect("Stack is empty, push error");
        match val {
            PSObject::Int(n) => assert_eq!(*n, 2),
            _ => panic!("Top of stack is not correct value after copy and pop, copy failed!"),
        }
    }
    #[test]
    fn test_dup() {
        // create a stack
        let mut s = Stack::new();
        s.push(PSObject::Int(42));
        s.push(PSObject::Int(32));
        s.dup();
        s.pop();
        let val = s.peek().expect("Stack is empty, push error");
        match val {
            PSObject::Int(n) => assert_eq!(*n, 32),
            _ => panic!("Top of stack is not correct value, pop after dup failed!"),
        }
    }
    #[test]
    fn test_clear() {
        // create a stack
        let mut s = Stack::new();
        s.push(PSObject::Int(42));
        s.push(PSObject::Int(32));
        s.push(PSObject::Int(42));
        s.push(PSObject::Int(32));
        s.clear();
        s.push(PSObject::Bool(true));
        let val = s.peek().expect("Stack is empty, push error!");
        match val {
            PSObject::Bool(n) => assert_eq!(*n, true),
            _ => panic!("Top of stack is not correct object type! pop failed!"),
        }
    }
    #[test]
    fn test_count() {
        // create a stack
        let mut s = Stack::new();
        s.push(PSObject::Int(42));
        s.push(PSObject::Int(32));
        s.push(PSObject::Int(42));
        s.push(PSObject::Int(32));
        let val = s.count();
        assert_eq!(val, 4);
    }

    #[test]
    fn exch_too_few() {
        let mut s = Stack::new();
        assert_eq!(s.exch(), false);
        s.push(PSObject::Int(1));
        assert_eq!(s.exch(), false);
    }

    #[test]
    fn dup_empty() {
        let mut s = Stack::new();
        assert_eq!(s.dup(), false);
        assert!(s.peek().is_none());
    }

    #[test]
    fn copy_too_big_and_zero() {
        let mut s = Stack::new();
        s.push(PSObject::Int(1));
        // too big
        assert_eq!(s.copy(2), false);
        // zero is a no-op but valid
        assert_eq!(s.copy(0), true);
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn pop_empty_does_not_panic() {
        let mut s = Stack::new();
        s.pop();
        assert!(s.peek().is_none());
    }

    #[test]
    fn double_exch_idempotent() {
        let mut s = Stack::new();
        s.push(PSObject::Int(10));
        s.push(PSObject::Int(20));
        assert!(s.exch());
        // [20,10] now
        assert!(s.exch());
        // should be back to [10,20]
        if let PSObject::Int(top) = s.peek().unwrap() {
            assert_eq!(*top, 20);
        } else {
            panic!("Expected Int");
        }
        s.pop();
        if let PSObject::Int(next) = s.peek().unwrap() {
            assert_eq!(*next, 10);
        } else {
            panic!("Expected Int");
        }
    }

    #[test]
    fn mixed_type_count_clear() {
        let mut s = Stack::new();
        s.push(PSObject::Bool(true));
        s.push(PSObject::String("hi".into()));
        s.push(PSObject::Array(vec![
            PSObject::Int(1),
            PSObject::Bool(false),
        ]));
        assert_eq!(s.count(), 3);
        s.clear();
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn copy_slice_order() {
        let mut s = Stack::new();
        for i in 1..5 {
            s.push(PSObject::Int(i));
        }
        assert!(s.copy(3));
        // after copying [2,3,4], stack should be [1,2,3,4,2,3,4]
        let mut values = Vec::new();
        while let Some(obj) = s.peek() {
            if let PSObject::Int(n) = obj {
                values.push(*n);
            }
            s.pop();
        }
        assert_eq!(
            values,
            vec![4, 3, 2, 4, 3, 2, 1].into_iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn dict_and_array_peek() {
        let mut s = Stack::new();
        let mut d = HashMap::new();
        d.insert("x".into(), PSObject::Int(99));
        s.push(PSObject::Dict(d.clone()));
        s.push(PSObject::Array(vec![PSObject::Bool(true)]));
        // peek on array
        match s.peek().unwrap() {
            PSObject::Array(a) => assert_eq!(a.len(), 1),
            _ => panic!("Expected Array"),
        }
        s.pop();
        // peek on dict
        match s.peek().unwrap() {
            PSObject::Dict(m) => assert_eq!(m.get("x"), Some(&PSObject::Int(99))),
            _ => panic!("Expected Dict"),
        }
    }
}
