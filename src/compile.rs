//! State machine compiler.

use std::mem::replace;

use parse;
use parse::{Expr, NonGreedy};
use vm::{Program, State, Want, Range};


/// Compile an AST into a `Program`.
pub fn compile(e: &Expr) -> Program {
    let mut b = Builder::new();
    let tails = compile_expr(&mut b, &[(Initial, Low)], e);
    b.reify(tails)
}


type Position = (Pos, Prec);

#[deriving(Clone)]
enum Pos {
    Initial,
    Index(uint)
}

#[deriving(Clone)]
enum Prec {
    Low,
    High
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
        ~[(Index(tail), Low)]
    }

    fn connect(&mut self, tails: &[Position]) {
        let head = self.states.len();
        for &(pos, prec) in tails.iter() {
            let targets: &mut ~[uint] = match pos {
                Initial => &mut self.initial,
                Index(index) => match self.states[index] {
                    (_, ref mut targets) => targets
                }
            };
            match prec {
                Low => targets.push(head),
                High => targets.insert(0, head)
            };
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
        parse::Repeat(ref inner, min, Some(max), greedy) => {
            let mut tails = ~[];
            let mut last = prev.to_owned();
            for i in range(0, max) {
                if i >= min {
                    tails.push_all(last);
                }
                last = compile_expr(b, last, *inner);
            }

            if greedy == NonGreedy {
                // Give all early-exit transitions higher priority
                for tail in tails.mut_iter() {
                    let &(pos, _) = tail;
                    replace(tail, (pos, High));
                }
            }

            tails.push_all(last);
            tails
        },
        parse::Repeat(..) => fail!("repeats not implemented yet"),
        parse::Capture(..) => fail!("captures not implemented yet")
    }
}
