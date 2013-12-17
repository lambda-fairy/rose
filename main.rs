#[allow(dead_code)];
#[feature(macro_rules)];

//mod vm;
mod parse;

/*
fn main() {
    let prog = [
        vm::Fork(1, 3),
        vm::Range('a' as u8, 'a' as u8),
        vm::Jump(3),
        vm::Range('b' as u8, 'b' as u8),
        vm::Done,
        ];
    println!("{:?}", vm::recursive(prog, "b"));
}
*/

fn main() {
    println!("{:?}", parse::parse("(a|b)c"))
}
