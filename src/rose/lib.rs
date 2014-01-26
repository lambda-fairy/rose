#[crate_id = "github.com/lfairy/rose"];
#[license = "MIT"];
#[comment = "Super simple regular expression library"];

//!
//! ~~~
//! extern mod rose;
//!
//! let names = rose::compile(r"Bon Bon|Lyra");
//! match names.exec("Lyra Heartstrings") {
//!     Some(result) => println!("Found: {}", result.group(0)),
//!     None => fail!("Oh noes!")
//! }
//! ~~~
//!
//!
//! # A simple regular expression engine
//!
//! Rose is a library that executes regular expressions.  A subset of
//! PCRE notation is supported, excluding backreferences and lookaround
//! which cannot be implemented efficiently.
//!
//! To get started, prepare a pattern using [compile](fn.compile.html).
//! This creates a [Regex](struct.Regex.html) object, which can then be
//! run using its `exec` and `search` methods.
//!

#[feature(macro_rules)];

mod charclass;
pub mod compile;
pub mod parse;
pub mod vm;


/// A compiled regular expression.  Use [compile](fn.compile.html) to
/// create one of these.
pub struct Regex {
    priv code: vm::Code
}

impl Regex {
    /// Create a `Regex` from a code block.
    pub fn from_code(code: vm::Code) -> Regex {
        Regex {
            code: code
        }
    }

    /// Check if the regex matches the given string.
    pub fn matches(&self, s: &str) -> bool {
        let mut vm = vm::VM::new(self.code);
        for c in s.chars() {
            vm.feed(c);
            if vm.is_match() {
                return true;
            }
        }
        false
    }
}


/// Compile a regular expression.  Fails on invalid syntax.
pub fn compile(regex: &str) -> Regex {
    Regex::from_code(compile::compile(&parse::parse(regex)))
}
