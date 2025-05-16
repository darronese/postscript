use crate::interpreter::object::PSObject;
use crate::interpreter::stack::Stack;
use std::collections::HashMap;

// can change to lexical scoping, dynamic is on by default
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Scoping {
    Dynamic,
    Lexical,
}

struct Frame {
    map: HashMap<String, PSObject>,
    // index in dict_stack
    parent: usize,
}

pub struct Interpreter {
    op_stack: Stack,
    // where our dictionary operations will lay
    dict_stack: Vec<Frame>,
    scoping: Scoping,
    // runs until quits
    quit: bool,
}

macro_rules! cmp_int {
    ($self:ident, $op:tt) => {{
        let b = $self.op_stack.pop().ok_or("stackunderflow")?;
        let a = $self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Int(a), PSObject::Int(b)) = (a, b) {
            $self.op_stack.push(PSObject::Bool(a $op b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }};
}

macro_rules! unary_int {
    ($self:ident, $body:expr) => {{
        let v = $self.op_stack.pop().ok_or("stackunderflow")?;
        if let PSObject::Int(n) = v {
            $self.op_stack.push(PSObject::Int($body(n)));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }};
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            // initialize the properties
            op_stack: Stack::new(),
            scoping: Scoping::Dynamic,
            quit: false,
            dict_stack: vec![Frame {
                map: HashMap::new(),
                parent: 0,
            }],
        }
    }

    // tokenize stack, return vector of objects
    fn raw_tokens(program: &str) -> Vec<String> {
        let mut toks = Vec::<String>::new();
        let mut buf = String::new();

        let mut in_str = false; // inside ( ... )
        let mut brace_depth = 0; // inside { ... }
        let mut bracket_depth = 0; // inside [ ... ]

        for c in program.chars() {
            if in_str {
                buf.push(c);
                if c == ')' {
                    in_str = false;
                    toks.push(buf.clone());
                    buf.clear();
                }
            } else if brace_depth > 0 || bracket_depth > 0 {
                match c {
                    '{' => {
                        brace_depth += 1;
                        buf.push(c);
                    }
                    '}' => {
                        brace_depth -= 1;
                        buf.push(c);
                        if brace_depth == 0 && bracket_depth == 0 {
                            toks.push(buf.clone());
                            buf.clear();
                        }
                    }
                    '[' => {
                        bracket_depth += 1;
                        buf.push(c);
                    }
                    ']' => {
                        bracket_depth -= 1;
                        buf.push(c);
                        if brace_depth == 0 && bracket_depth == 0 {
                            toks.push(buf.clone());
                            buf.clear();
                        }
                    }
                    _ => buf.push(c),
                }
            } else {
                match c {
                    '(' => {
                        in_str = true;
                        buf.push(c);
                    }
                    '{' => {
                        brace_depth = 1;
                        buf.push(c);
                    }
                    '[' => {
                        bracket_depth = 1;
                        buf.push(c);
                    }
                    ch if ch.is_whitespace() => {
                        if !buf.is_empty() {
                            toks.push(buf.clone());
                            buf.clear();
                        }
                    }
                    _ => buf.push(c),
                }
            }
        }
        if !buf.is_empty() {
            toks.push(buf);
        }
        toks
    }

    // turn into ps object after tokenizing
    fn parse_token(tok: &str) -> PSObject {
        // string literal
        if tok.starts_with('(') && tok.ends_with(')') {
            PSObject::String(tok[1..tok.len() - 1].into())

        // procedure literal
        } else if tok.starts_with('{') {
            let inner = &tok[1..tok.len() - 1];
            let code = Self::tokenize(inner);
            PSObject::Procedure {
                code,
                env: HashMap::new(),
            }

        // array
        } else if tok.starts_with('[') && tok.ends_with(']') {
            let inner = &tok[1..tok.len() - 1];
            let vec = Self::tokenize(inner);
            PSObject::Array(vec)

        // integer
        } else if let Ok(n) = tok.parse::<i32>() {
            PSObject::Int(n)
        // boolean
        } else if tok == "true" {
            PSObject::Bool(true)
        } else if tok == "false" {
            PSObject::Bool(false)

        // literal names
        } else if tok.starts_with('/') {
            PSObject::Name(tok.into())

        // executable name
        } else {
            PSObject::Name(tok.into())
        }
    }

    // run raw_toekns and parse tokens for convienence
    fn tokenize(program: &str) -> Vec<PSObject> {
        Self::raw_tokens(program)
            .into_iter()
            .map(|raw| Self::parse_token(&raw))
            .collect()
    }

    // main loop: for each token, look it up, dispatch it (operator), or push in as data
    pub fn run(&mut self, program: &str) -> Result<(), String> {
        for mut obj in Self::tokenize(program) {
            if self.quit {
                break;
            }

            // lexical attachment
            if let PSObject::Procedure { ref mut env, .. } = obj {
                if env.is_empty() {
                    *env = self.dict_stack.last().unwrap().map.clone();
                }
            }

            // ── dispatch ──
            let cur_top = self.dict_stack.len() - 1;
            self.execute_object(obj, cur_top)?;
        }
        Ok(())
    }

    // Matches if its dynamic or lexical
    fn lookup_name(&self, name: &str) -> Option<PSObject> {
        match self.scoping {
            Scoping::Dynamic => {
                for frame in self.dict_stack.iter().rev() {
                    if let Some(v) = frame.map.get(name) {
                        return Some(v.clone());
                    }
                }
                None
            }
            Scoping::Lexical => {
                let mut idx = self.dict_stack.len() - 1;
                loop {
                    let frame = &self.dict_stack[idx];
                    if let Some(v) = frame.map.get(name) {
                        return Some(v.clone());
                    }
                    if frame.parent == idx {
                        // reached the bottom
                        return None;
                    }
                    idx = frame.parent;
                }
            }
        }
    }

    // check if its a built in operator
    // utilizing pattern matching to efficient gathering
    fn is_operator(&self, name: &str) -> bool {
        matches!(
            name,
            "add"
                | "sub"
                | "eq"
                | "ne"
                | "gt"
                | "lt"
                | "ge"
                | "le"
                | "and"
                | "or"
                | "not"
                | "mul"
                | "div"
                | "mod"
                | "exch"
                | "pop"
                | "dup"
                | "copy"
                | "clear"
                | "count"
                | "dict"
                | "begin"
                | "end"
                | "def"
                | "length"
                | "maxlength"
                | "get"
                | "getinterval"
                | "putinterval"
                | "eq"
                | "ne"
                | "gt"
                | "lt"
                | "ge"
                | "le"
                | "and"
                | "or"
                | "not"
                | "true"
                | "false"
                | "if"
                | "ifelse"
                | "for"
                | "repeat"
                | "quit"
                | "print"
                | "="
                | "=="
                | "dict"
                | "begin"
                | "end"
                | "def"
                | "length"
                | "maxlength"
                | "get"
                | "getinterval"
                | "putinterval"
                | "idiv"
                | "abs"
                | "neg"
                | "ceiling"
                | "floor"
                | "round"
                | "sqrt"
                | "lexical"
                | "dynamic"
                | "exec"
        )
    }

    // Dispatch to the appropriate operator method
    fn execute_operator(&mut self, op: &str) -> Result<(), String> {
        match op {
            "add" => self.op_add(),
            "sub" => self.op_sub(),
            "mul" => self.op_mul(),
            "div" => self.op_div(),
            "mod" => self.op_mod(),
            "exch" => self.op_exch(),
            "pop" => {
                self.op_pop()?;
                Ok(())
            }
            "dup" => self.op_dup(),
            "clear" => self.op_clear(),
            "count" => self.op_count(),
            "quit" => {
                self.quit = true;
                Ok(())
            }
            "copy" => self.op_copy(),
            "dict" => self.op_dict(),
            "begin" => self.op_begin(),
            "end" => self.op_end(),
            "def" => self.op_def(),
            "length" => self.op_length(),
            "maxlength" => self.op_maxlength(),

            "=" => self.op_equals(),
            "==" => self.op_eqeq(),
            "print" => self.op_print(),

            "get" => self.op_get(),
            "getinterval" => self.op_getinterval(),
            "putinterval" => self.op_putinterval(),
            "eq" => self.op_eq(),
            "ne" => self.op_ne(),
            "gt" => self.op_gt(),
            "lt" => self.op_lt(),
            "ge" => self.op_ge(),
            "le" => self.op_le(),

            "and" => self.op_and(),
            "or" => self.op_or(),
            "not" => self.op_not(),
            "if" => self.op_if(),
            "ifelse" => self.op_ifelse(),
            "for" => self.op_for(),
            "repeat" => self.op_repeat(),
            "idiv" => self.op_idiv(),
            "abs" => self.op_abs(),
            "neg" => self.op_neg(),
            "ceiling" => self.op_ceiling(),
            "floor" => self.op_floor(),
            "round" => self.op_round(),
            "sqrt" => self.op_sqrt(),
            "lexical" => {
                self.scoping = Scoping::Lexical;
                Ok(())
            }
            "dynamic" => {
                self.scoping = Scoping::Dynamic;
                Ok(())
            }
            "exec" => self.op_exec(),
            _ => Err(format!("Unknown operator {}", op)),
        }
    }

    // PS ARITHMETIC
    fn op_add(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Int(a), PSObject::Int(b)) = (a, b) {
            self.op_stack.push(PSObject::Int(a + b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    fn op_sub(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Int(a), PSObject::Int(b)) = (a, b) {
            self.op_stack.push(PSObject::Int(a - b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }
    fn op_mul(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Int(a), PSObject::Int(b)) = (a, b) {
            self.op_stack.push(PSObject::Int(a * b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }
    fn op_div(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Int(a), PSObject::Int(b)) = (a, b) {
            if b == 0 {
                // PostScript error for /0
                return Err("undefinedresult".into());
            }
            self.op_stack.push(PSObject::Int(a / b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }
    fn op_mod(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Int(a), PSObject::Int(b)) = (a, b) {
            if b == 0 {
                return Err("undefinedresult".into());
            }
            self.op_stack.push(PSObject::Int(a % b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    // PS STACK OPERATION
    fn op_exch(&mut self) -> Result<(), String> {
        if self.op_stack.exch() {
            Ok(())
        } else {
            Err("stackunderflow".into())
        }
    }

    fn op_pop(&mut self) -> Result<(), String> {
        if self.op_stack.pop().is_some() {
            Ok(())
        } else {
            Err("stackunderflow".into())
        }
    }

    fn op_dup(&mut self) -> Result<(), String> {
        if self.op_stack.dup() {
            Ok(())
        } else {
            Err("stackunderflow".into())
        }
    }

    fn op_clear(&mut self) -> Result<(), String> {
        self.op_stack.clear();
        Ok(())
    }

    fn op_count(&mut self) -> Result<(), String> {
        let n = self.op_stack.count();
        self.op_stack.push(PSObject::Int(n));
        Ok(())
    }

    fn op_copy(&mut self) -> Result<(), String> {
        let n = match self.op_stack.pop() {
            Some(PSObject::Int(i)) => i,
            _ => return Err("typecheck".into()),
        };
        if self.op_stack.copy(n) {
            Ok(())
        } else {
            Err("rangecheck".into())
        }
    }

    // DICTIONARY OPERATIONS

    // create a new dictionary with specified size
    fn op_dict(&mut self) -> Result<(), String> {
        let n = match self.op_stack.pop() {
            Some(PSObject::Int(i)) => i as usize,
            _ => return Err("typecheck".into()),
        };
        self.op_stack
            .push(PSObject::Dict(HashMap::with_capacity(n)));
        Ok(())
    }

    // pop a dict and push it in as a new frame
    fn op_begin(&mut self) -> Result<(), String> {
        match self.op_stack.pop() {
            Some(PSObject::Dict(d)) => {
                let parent = self.dict_stack.len() - 1;
                self.dict_stack.push(Frame { map: d, parent });
                Ok(())
            }
            _ => Err("typecheck".into()),
        }
    }

    // pop from the top frame
    fn op_end(&mut self) -> Result<(), String> {
        if self.dict_stack.len() <= 1 {
            Err("dictstackunderflow".into())
        } else {
            self.dict_stack.pop();
            Ok(())
        }
    }

    // pop a value and literal name into the current Frame
    fn op_def(&mut self) -> Result<(), String> {
        let value = self.op_stack.pop().ok_or("stackunderflow")?;
        let key = self.op_stack.pop().ok_or("stackunderflow")?;
        if let PSObject::Name(name) = key {
            let frame = self.dict_stack.last_mut().unwrap();
            frame.map.insert(name, value);
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    // length of dict, string, arr
    fn op_length(&mut self) -> Result<(), String> {
        let obj = self.op_stack.pop().ok_or("stackunderflow")?;
        let len = match obj {
            PSObject::String(s) => s.chars().count() as i32,
            PSObject::Array(a) => a.len() as i32,
            PSObject::Dict(d) => d.len() as i32,
            _ => return Err("typecheck".into()),
        };
        self.op_stack.push(PSObject::Int(len));
        Ok(())
    }

    // find the max length
    fn op_maxlength(&mut self) -> Result<(), String> {
        self.op_length()
    }

    // string index get OR array index get
    fn op_get(&mut self) -> Result<(), String> {
        let idx = match self.op_stack.pop().ok_or("stackunderflow")? {
            PSObject::Int(i) if i >= 0 => i as usize,
            _ => return Err("typecheck".into()),
        };
        let obj = self.op_stack.pop().ok_or("stackunderflow")?;
        match obj {
            PSObject::String(s) => {
                if idx < s.len() {
                    let byte = s.as_bytes()[idx];
                    self.op_stack.push(PSObject::Int(byte as i32));
                    Ok(())
                } else {
                    Err("rangecheck".into())
                }
            }
            PSObject::Array(a) => {
                if idx < a.len() {
                    self.op_stack.push(a[idx].clone());
                    Ok(())
                } else {
                    Err("rangecheck".into())
                }
            }
            _ => Err("typecheck".into()),
        }
    }

    // gets the current interval
    fn op_getinterval(&mut self) -> Result<(), String> {
        let count = match self.op_stack.pop().ok_or("stackunderflow")? {
            PSObject::Int(i) if i >= 0 => i as usize,
            _ => return Err("typecheck".into()),
        };
        let idx = match self.op_stack.pop().ok_or("stackunderflow")? {
            PSObject::Int(i) if i >= 0 => i as usize,
            _ => return Err("typecheck".into()),
        };
        let obj = self.op_stack.pop().ok_or("stackunderflow")?;
        match obj {
            PSObject::String(s) => {
                if idx + count <= s.len() {
                    let substr = s[idx..idx + count].to_string();
                    self.op_stack.push(PSObject::String(substr));
                    Ok(())
                } else {
                    Err("rangecheck".into())
                }
            }
            PSObject::Array(a) => {
                if idx + count <= a.len() {
                    let slice = a[idx..idx + count].to_vec();
                    self.op_stack.push(PSObject::Array(slice));
                    Ok(())
                } else {
                    Err("rangecheck".into())
                }
            }
            _ => Err("typecheck".into()),
        }
    }

    // puts the interval
    fn op_putinterval(&mut self) -> Result<(), String> {
        // src must be string or array
        let src = self.op_stack.pop().ok_or("stackunderflow")?;
        let idx = match self.op_stack.pop().ok_or("stackunderflow")? {
            PSObject::Int(i) if i >= 0 => i as usize,
            _ => return Err("typecheck".into()),
        };
        let dest = self.op_stack.pop().ok_or("stackunderflow")?;
        match (dest, src) {
            (PSObject::String(mut d), PSObject::String(s)) => {
                if idx + s.len() <= d.len() {
                    d.replace_range(idx..idx + s.len(), &s);
                    self.op_stack.push(PSObject::String(d));
                    Ok(())
                } else {
                    Err("rangecheck".into())
                }
            }
            (PSObject::Array(mut d), PSObject::Array(s)) => {
                if idx + s.len() <= d.len() {
                    for i in 0..s.len() {
                        d[idx + i] = s[i].clone();
                    }
                    self.op_stack.push(PSObject::Array(d));
                    Ok(())
                } else {
                    Err("rangecheck".into())
                }
            }
            _ => Err("typecheck".into()),
        }
    }

    // PRINTING LOGIC
    // prints top of stack without new line
    fn op_equals(&mut self) -> Result<(), String> {
        let obj = self.op_stack.pop().ok_or("stackunderflow")?;
        println!("{:?}", obj); // keeps newline
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        Ok(())
    }

    // prints top of stack with new line
    fn op_eqeq(&mut self) -> Result<(), String> {
        let obj = self.op_stack.pop().ok_or("stackunderflow")?;
        print!("{:?}", obj); // no newline
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        Ok(())
    }

    // consumes string and prints it without any new line
    fn op_print(&mut self) -> Result<(), String> {
        match self.op_stack.pop() {
            Some(PSObject::String(s)) => {
                print!("{s}"); // NO println!()
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                Ok(())
            }
            Some(_) => Err("typecheck".into()),
            None => Err("stackunderflow".into()),
        }
    }

    // COMPARISONS LOGIC
    // checks for equal
    fn op_eq(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        self.op_stack.push(PSObject::Bool(a == b));
        Ok(())
    }

    fn op_ne(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        self.op_stack.push(PSObject::Bool(a != b));
        Ok(())
    }
    fn op_gt(&mut self) -> Result<(), String> {
        cmp_int!(self, >)
    }
    fn op_lt(&mut self) -> Result<(), String> {
        cmp_int!(self, <)
    }
    fn op_ge(&mut self) -> Result<(), String> {
        cmp_int!(self, >=)
    }
    fn op_le(&mut self) -> Result<(), String> {
        cmp_int!(self, <=)
    }
    fn op_and(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Bool(a), PSObject::Bool(b)) = (a, b) {
            self.op_stack.push(PSObject::Bool(a && b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    fn op_or(&mut self) -> Result<(), String> {
        let b = self.op_stack.pop().ok_or("stackunderflow")?;
        let a = self.op_stack.pop().ok_or("stackunderflow")?;
        if let (PSObject::Bool(a), PSObject::Bool(b)) = (a, b) {
            self.op_stack.push(PSObject::Bool(a || b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    fn op_not(&mut self) -> Result<(), String> {
        let v = self.op_stack.pop().ok_or("stackunderflow")?;
        if let PSObject::Bool(b) = v {
            self.op_stack.push(PSObject::Bool(!b));
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    // helper function for for loop
    fn pop_int(&mut self) -> Result<i32, String> {
        match self.op_stack.pop() {
            Some(PSObject::Int(n)) => Ok(n),
            Some(_) => Err("typecheck".into()),
            None => Err("stackunderflow".into()),
        }
    }

    // CONTROL STATEMENTS
    fn op_if(&mut self) -> Result<(), String> {
        let proc = self.op_stack.pop().ok_or("stackunderflow")?;
        let cond = self.op_stack.pop().ok_or("stackunderflow")?;
        if let PSObject::Bool(b) = cond {
            if b {
                self.exec_proc(proc)?;
            }
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    fn op_ifelse(&mut self) -> Result<(), String> {
        let proc_f = self.op_stack.pop().ok_or("stackunderflow")?;
        let proc_t = self.op_stack.pop().ok_or("stackunderflow")?;
        let cond = self.op_stack.pop().ok_or("stackunderflow")?;
        if let PSObject::Bool(b) = cond {
            self.exec_proc(if b { proc_t } else { proc_f })
        } else {
            Err("typecheck".into())
        }
    }

    fn op_repeat(&mut self) -> Result<(), String> {
        let proc = self.op_stack.pop().ok_or("stackunderflow")?;
        let count = match self.op_stack.pop() {
            Some(PSObject::Int(n)) if n >= 0 => n,
            _ => return Err("typecheck".into()),
        };
        for _ in 0..count {
            self.exec_proc(proc.clone())?;
        }
        Ok(())
    }

    fn op_for(&mut self) -> Result<(), String> {
        let proc = self.op_stack.pop().ok_or("stackunderflow")?;
        let limit = self.pop_int()?;
        let inc = self.pop_int()?;
        let mut var = self.pop_int()?;
        if inc == 0 {
            return Err("rangecheck".into());
        }
        let cmp: fn(i32, i32) -> bool = if inc > 0 {
            |v, l| v <= l
        } else {
            |v, l| v >= l
        };
        while cmp(var, limit) {
            self.op_stack.push(PSObject::Int(var));
            self.exec_proc(proc.clone())?;
            var += inc;
        }
        Ok(())
    }

    // lookup function in order to help lexical scoping
    fn lookup_from(&self, mut idx: usize, name: &str) -> Option<PSObject> {
        loop {
            let frame = &self.dict_stack[idx];
            if let Some(v) = frame.map.get(name) {
                return Some(v.clone());
            }
            if frame.parent == idx {
                // hit bottom (the system dict)
                return None;
            }
            idx = frame.parent; // follow static-link
        }
    }

    // executes our current object based on lexical / dynamic
    // our run function runs this
    fn execute_object(&mut self, obj: PSObject, start_from: usize) -> Result<(), String> {
        match obj {
            // check for operator
            PSObject::Name(ref n) if self.is_operator(n) => self.execute_operator(n),

            // check for function declare
            PSObject::Name(ref n) if n.starts_with('/') => {
                self.op_stack.push(PSObject::Name(n[1..].to_string()));
                Ok(())
            }

            // checks whether or not its lexical/ dyanmic
            PSObject::Name(ref n) => {
                // pick lookup strategy depending on current scoping mode
                let val = if self.scoping == Scoping::Dynamic {
                    self.lookup_name(n) // dynamic search
                } else {
                    self.lookup_from(start_from, n) // lexical/static search
                };
                val.ok_or_else(|| format!("undefined name {}", n))
                    .map(|v| self.op_stack.push(v))
            }

            // PSObject literals
            other => {
                self.op_stack.push(other);
                Ok(())
            }
        }
    }

    // helper function to help execute
    fn exec_proc(&mut self, proc_obj: PSObject) -> Result<(), String> {
        if let PSObject::Procedure { code, env } = proc_obj {
            // Will we push the snapshot?
            let mut pushed = false;
            let mut env_idx = self.dict_stack.len() - 1; // current top

            if self.scoping == Scoping::Lexical {
                // 1) push the captured frame
                let parent = env_idx;
                self.dict_stack.push(Frame {
                    map: env.clone(),
                    parent,
                });
                env_idx = self.dict_stack.len() - 1;
                pushed = true;
            }

            // 2) execute
            for obj in code {
                self.execute_object(obj, env_idx)?;
            }

            // 3) pop the temp frame if we pushed it
            if pushed {
                self.dict_stack.pop();
            }
            Ok(())
        } else {
            Err("typecheck".into())
        }
    }

    fn op_idiv(&mut self) -> Result<(), String> {
        let b = self.pop_int()?; // divisor
        if b == 0 {
            return Err("undefinedresult".into());
        }
        let a = self.pop_int()?; // dividend
        self.op_stack.push(PSObject::Int(a / b)); // trunc toward 0
        Ok(())
    }

    // MORE ARITHMETIC FUNCTIONS
    fn op_abs(&mut self) -> Result<(), String> {
        unary_int!(self, |n: i32| n.abs())
    }
    fn op_neg(&mut self) -> Result<(), String> {
        unary_int!(self, |n: i32| -n)
    }
    fn op_ceiling(&mut self) -> Result<(), String> {
        unary_int!(self, |n| n)
    }
    fn op_floor(&mut self) -> Result<(), String> {
        unary_int!(self, |n| n)
    }
    fn op_round(&mut self) -> Result<(), String> {
        unary_int!(self, |n| n)
    }

    fn op_sqrt(&mut self) -> Result<(), String> {
        let n = self.pop_int()?;
        if n < 0 {
            return Err("typecheck".into());
        }
        // truncate
        let root = (n as f64).sqrt() as i32;
        self.op_stack.push(PSObject::Int(root));
        Ok(())
    }
    // when executing
    fn op_exec(&mut self) -> Result<(), String> {
        let obj = self.op_stack.pop().ok_or("stackunderflow")?;
        match obj {
            PSObject::Procedure { .. } => {
                // run and leave nothing
                self.exec_proc(obj)?;
                Ok(())
            }
            _ => Err("typecheck".into()),
        }
    }
}
