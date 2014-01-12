#[crate_id = "github.com/lfairy/rose"];
#[license = "MIT"];
#[comment = "Super simple regular expression library"];

#[feature(macro_rules)];

mod charclass;
pub mod compile;
pub mod parse;
pub mod vm;


/// A compiled regular expression.
pub type Regex = vm::Regex;


/// Compile a regular expression.  Fails on invalid syntax.
pub fn compile(regex: &str) -> Regex {
    compile::compile(&parse::parse(regex))
}
