//! State machine compiler.

use std::mem::replace;
use std::uint::MAX;

use parse;
use parse::{Expr, Greedy, NonGreedy};
use vm::{Program, State, Want, Nothing, Range};


/// Compile an AST into a `Program`.
pub fn compile(e: &Expr) -> Program {
    let (mut b, prev) = Builder::new();
    let tails = compile_expr(&mut b, prev, e);
    b.reify(tails)
}


type Pos = (uint, uint);


struct Builder {
    states: ~[State]
}

impl Builder {
    fn new() -> (Builder, ~[Pos]) {
        let mut b = Builder { states: ~[(Nothing, ~[])] };
        let tails = ~[b.reserve(0)];
        (b, tails)
    }

    fn push(&mut self, prev: &[Pos], w: Want) -> uint {
        let end = self.states.len();
        self.states.push((w, ~[]));
        self.connect(prev, end);
        end
    }

    fn push_empty(&mut self, prev: &[Pos]) -> uint {
        self.push(prev, Nothing)
    }

    fn reserve(&mut self, index: uint) -> Pos {
        match self.states[index] {
            (_, ref mut exits) => {
                let end = exits.len();
                exits.push(MAX);
                (index, end)
            }
        }
    }

    fn connect(&mut self, prev: &[Pos], here: uint) {
        for &(state, exit) in prev.iter() {
            match self.states[state] {
                (_, ref mut exits) => replace(&mut exits[exit], here)
            };
        }
    }

    fn reify(mut self, prev: &[Pos]) -> ~[State] {
        let end = self.states.len();
        self.connect(prev, end);
        let Builder { states } = self;
        states
    }
}


fn compile_expr(b: &mut Builder, prev: &[Pos], e: &Expr) -> ~[Pos] {
    match *e {
        parse::Empty => prev.to_owned(),
        parse::Range(lo, hi) => {
            let end = b.push(prev, Range(lo, hi));
            ~[b.reserve(end)]
        },
        parse::Concatenate(ref inners) => {
            let mut last = prev.to_owned();
            for inner in inners.iter() {
                last = compile_expr(b, last, inner);
            }
            last
        },
        parse::Alternate(ref inners) => {
            let fork = {
                let end = b.push_empty(prev);
                ~[b.reserve(end)]
            };
            let mut tails = ~[];
            for inner in inners.iter() {
                tails.push_all_move(compile_expr(b, fork, inner));
            }
            tails
        },
        parse::Repeat(ref inner, min, Some(max), greedy) => {
            let mut tails = ~[];
            let mut last = prev.to_owned();
            for i in range(0, max) {
                let last_ =
                    if i < min {
                        last
                    } else {
                        let end = b.push_empty(last);
                        let x = b.reserve(end); let y = b.reserve(end);
                        match greedy {
                            NonGreedy => { tails.push(x); ~[y] },
                            Greedy    => { tails.push(y); ~[x] }
                        }
                    };
                last = compile_expr(b, last_, *inner);
            }
            tails.push_all_move(last);
            tails
        },
        parse::Repeat(..) => fail!("repeats not implemented yet"),
        parse::Capture(..) => fail!("captures not implemented yet")
    }
}
