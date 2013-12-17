/// Regex VM

pub enum Inst {
    Range(u8, u8),
    Jump(uint),
    Fork(uint, uint),
    Done,
}

pub fn execute(prog: &[Inst], input: &str) -> bool {

    macro_rules! recurse(
        ($new_p:expr, $new_i:expr) => (
            inner(prog, input, $new_p, $new_i)
        );
    )

    fn inner(prog: &[Inst], input: &str, p: uint, i: uint) -> bool {
        if p < prog.len() {
            let inst = prog[p];
            match inst {
                Range(lo, hi) => {
                    if i < input.len() && lo <= input[i] && input[i] <= hi {
                        recurse!(p+1, i+1)
                    } else {
                        false
                    }
                },
                Jump(to) => recurse!(to, i),
                Fork(left, right) => {
                    if recurse!(left, i) {
                        true
                    } else {
                        recurse!(right, i)
                    }
                },
                Done => true,
            }
        } else {
            fail!("Bytecode invalid! Gasp!")
        }
    }

    // Start it all going
    recurse!(0, 0)
}
