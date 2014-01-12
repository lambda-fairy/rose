//! Byte code compiler.

use parse;
use parse::{Expr, Greedy, NonGreedy};
use vm::{Code, Inst, Empty, Range, Jump, Fork, GFork, Match};


/// Compile an AST into byte code.
pub fn compile(e: &Expr) -> Code {
    let mut code: ~[Inst] = ~[];
    compile_expr(&mut code, e);
    code.push(Match);
    code
}


macro_rules! placeholder(
    () => ({
        let pos = code.len();
        code.push(Empty);
        pos
    })
)


fn compile_expr(code: &mut ~[Inst], e: &Expr) {
    match *e {
        parse::Empty => (),
        parse::Range(lo, hi) => code.push(Range(lo, hi)),
        parse::Concatenate(ref inners) => {
            for inner in inners.iter() {
                compile_expr(code, inner);
            }
        },
        parse::Alternate(ref inners) => compile_alternate(code, *inners),
        parse::Repeat(ref inner, min, max, greedy) => compile_repeat(code, *inner, min, max, greedy),
        parse::Capture(..) => fail!("captures not implemented yet")
    }
}


fn compile_alternate(code: &mut ~[Inst], inners: &[Expr]) {
    //
    // `a|b|c` compiles to:
    //
    //   A: fork B
    //      <a>
    //      jump END
    //   B: fork C
    //      <b>
    //      jump END
    //   C: <c>
    // END:
    //

    let mut forks: ~[uint] = ~[];
    let mut jumps: ~[uint] = ~[];
    let mut ends: ~[uint] = ~[];

    for inner in inners.slice_to(inners.len()-1).iter() {
        forks.push(placeholder!());
        compile_expr(code, inner);
        jumps.push(placeholder!());
        ends.push(code.len());
    }
    // The last subexpr shouldn't have any forks or jumps in it, so we
    // treat it separately
    compile_expr(code, &inners[inners.len()-1]);

    for (&fork, &next) in forks.iter().zip(ends.iter()) {
        // We either continue matching the current branch, or skip to
        // the next one
        code[fork].replace(Fork(next));
    }

    let end = code.len();
    for &jump in jumps.iter() {
        // After successfully matching a branch, jump to the end
        code[jump].replace(Jump(end));
    }
}


fn compile_repeat(code: &mut ~[Inst], inner: &Expr, min: u32, max: Option<u32>, greedy: Greedy) {
    // FIXME: This uses exponential memory (#2)
    match (min, max) {
        (_, Some(max_)) => {
            // Compile `min` repetitions
            for _ in range(0, min) {
                compile_expr(code, inner);
            }

            // After we've seen `min` repetitions, the remaining
            // `max - min` are optional
            let mut forks = ~[];
            for _ in range(min, max_) {
                forks.push(placeholder!());
                compile_expr(code, inner);
            }

            let end = code.len();
            for &fork in forks.iter() {
                code[fork].replace(make_fork(!greedy)(end));
            }
        },
        (0, None) => {
            // `A*` is equivalent to `(A+)?`, so we treat it that way
            let fork = placeholder!();
            compile_repeat(code, inner, 1, None, greedy);
            let len = code.len(); code[fork].replace(make_fork(!greedy)(len));
        },
        (_, None) => {
            // Compile `min` repetitions, then draw a loop around the
            // last instance
            let mut loop_start = 0u;
            for _ in range(0, min) {
                loop_start = code.len();
                compile_expr(code, inner);
            }
            let fork = placeholder!();
            code[fork].replace(make_fork(greedy)(loop_start));
        }
    }
}


fn make_fork(greedy: Greedy) -> fn(uint) -> Inst {
    match greedy {
        NonGreedy => Fork,
        Greedy => GFork
    }
}
