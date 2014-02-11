//! State machine compiler.

use parse;
use parse::{Expr, Greedy, NonGreedy};
use vm::{Program, State, Want, Range};


/// Compile an AST into a `Program`.
pub fn compile(e: &Expr) -> Program {
    let mut b = Builder::new();
    let tails = compile_expr(&mut b, &[Initial], e);
    b.reify(tails)
}


#[deriving(Clone)]
enum Position {
    Initial,
    Index(uint)
}


struct Builder {
    initial: ~[uint],
    states: ~[State]
}

impl Builder {
    fn new() -> Builder {
        Builder {
            initial: ~[],
            states: ~[]
        }
    }

    fn push(&mut self, w: Want) -> ~[Position] {
        let tail = self.states.len();
        self.states.push((w, ~[]));
        ~[Index(tail)]
    }

    fn connect(&mut self, tails: &[Position]) {
        let head = self.states.len();
        for &tail in tails.iter() {
            match tail {
                Initial => self.initial.push(head),
                Index(index) => match self.states[index] {
                    (_, ref mut targets) => targets.push(head)
                }
            }
        }
    }

    fn reify(mut self, tails: &[Position]) -> (~[uint], ~[State]) {
        self.connect(tails);
        let Builder { initial, states } = self;
        (initial, states)
    }
}


fn compile_expr(b: &mut Builder, prev: &[Position], e: &Expr) -> ~[Position] {
    match *e {
        parse::Empty => prev.to_owned(),
        parse::Range(lo, hi) => {
            b.connect(prev);
            b.push(Range(lo, hi))
        },
        parse::Concatenate(ref inners) => {
            let mut tails = prev.to_owned();
            for inner in inners.iter() {
                tails = compile_expr(b, tails, inner);
            }
            tails
        },
        parse::Alternate(ref inners) => {
            let mut tails = ~[];
            for inner in inners.iter() {
                tails.push_all_move(compile_expr(b, prev, inner));
            }
            tails
        },
        //parse::Repeat(ref inner, min, max, greedy) => {},
        parse::Repeat(..) => fail!("repeats not implemented yet"),
        parse::Capture(..) => fail!("captures not implemented yet")
    }
}
