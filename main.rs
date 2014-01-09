#[feature(macro_rules)];

mod charclass;
mod compile;
mod parse;
mod vm;

fn main() {
    println!("{:?}", compile::compile(&parse::parse("(?:ab)*")));
}
