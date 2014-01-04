#[feature(macro_rules)];

mod charclass;
mod parse;

fn main() {
    println!("{:?}", parse::parse("(?:a|b){,234}?c"))
}
