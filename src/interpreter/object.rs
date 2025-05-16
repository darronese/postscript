use std::collections::HashMap;
use std::fmt;

// enumerated type PSStack to contain all necessary default values of PS
// we NEED the debug here for us to print out the values of psobject, and when we want to make
// clones of our objects
#[derive(Debug, Clone, PartialEq)]
pub enum PSObject {
    Int(i32),
    Bool(bool),
    Real(f64),
    String(String),
    Dict(HashMap<String, PSObject>),
    Array(Vec<PSObject>),
    // represents literal and executable names
    Name(String),

    // CARRIES A STATIC LINK IN PROCEDURES
    Procedure {
        code: Vec<PSObject>,
        env: HashMap<String, PSObject>,
    },
}

impl fmt::Display for PSObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PSObject::Int(n) => write!(f, "{n}"),
            PSObject::Real(r) => write!(f, "{r}"),
            PSObject::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            PSObject::String(s) => write!(f, "({})", s.replace(')', "\\)")),
            PSObject::Name(n) => write!(f, "{}", n),
            PSObject::Array(arr) => {
                write!(f, "[")?;
                for (i, obj) in arr.iter().enumerate() {
                    if i != 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{obj}")?;
                }
                write!(f, "]")
            }
            PSObject::Dict(d) => {
                write!(f, "<<")?;
                // print “/key value ” for each entry
                for (k, v) in d {
                    write!(f, "/{k} {v} ")?;
                }
                write!(f, ">>")
            }
            PSObject::Procedure { code, .. } => {
                write!(f, "{{")?;
                for obj in code {
                    write!(f, "{obj} ")?;
                }
                write!(f, "}}")
            }
        }
    }
}
