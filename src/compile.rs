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
    /// Construct a `Builder`.
    fn new() -> (Builder, ~[Pos]) {
        let mut b = Builder { states: ~[(Nothing, ~[])] };
        let tails = ~[b.reserve(0)];
        (b, tails)
    }

    /// Push a new state, and connect the given edges to it.
    fn push(&mut self, prev: &[Pos], w: Want) -> uint {
        let end = self.states.len();
        self.states.push((w, ~[]));
        self.connect(prev, end);
        end
    }

    /// Push an empty transition.
    fn push_empty(&mut self, prev: &[Pos]) -> uint {
        self.push(prev, Nothing)
    }

    /// Draw a dangling edge coming out of the specified state.  The
    /// returned `Pos` should eventually be linked using `connect`.
    fn reserve(&mut self, index: uint) -> Pos {
        match self.states[index] {
            (_, ref mut exits) => {
                let end = exits.len();
                exits.push(MAX);  // Placeholder
                (index, end)
            }
        }
    }

    /// Connect all given dangling edges to a point.
    fn connect(&mut self, prev: &[Pos], here: uint) {
        for &(state, exit) in prev.iter() {
            match self.states[state] {
                (_, ref mut exits) => {
                    assert!(exits[exit] == MAX, "edge has not yet been connected");
                    replace(&mut exits[exit], here)
                }
            };
        }
    }

    /// Return the completed machine, consuming itself in the process.
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
        parse::Repeat(ref inner, min, max, greedy) => compile_repeat(b, prev, *inner, min, max, greedy),
        parse::Capture(..) => fail!("captures not implemented yet")
    }
}


fn compile_repeat(b: &mut Builder, prev: &[Pos], inner: &Expr, min: u32, max: Option<u32>, greedy: Greedy) -> ~[Pos] {
    match max {
        Some(max_) => {
            // `A{3,6}` => `AAA(A(A(A?)?)?`
            let last = compile_times(b, prev, inner, min);
            fn helper(b: &mut Builder, prev: &[Pos], inner: &Expr, count: u32, greedy: Greedy) -> ~[Pos] {
                if count == 0 {
                    prev.to_owned()
                } else {
                    compile_optional(b, prev, |b_, prev_| {
                        let last = compile_expr(b_, prev_, inner);
                        helper(b_, last, inner, count - 1, greedy)
                    }, greedy)
                }
            }
            helper(b, last, inner, max_ - min, greedy)
        },
        None =>
            if min == 0 {
                // `A*` => `(A+)?`
                compile_optional(b, prev, |b_, prev_| compile_plus(b_, prev_, inner, greedy), greedy)
            } else {
                // `A{3,}` => `AA(A+)`
                let last = compile_times(b, prev, inner, min - 1);
                compile_plus(b, last, inner, greedy)
            }
    }
}


fn compile_times(b: &mut Builder, prev: &[Pos], inner: &Expr, times: u32) -> ~[Pos] {
    let mut last = prev.to_owned();
    for _ in range(0, times) {
        last = compile_expr(b, last, inner);
    }
    last
}


fn compile_optional(b: &mut Builder, prev: &[Pos], inner: |b_: &mut Builder, prev_: &[Pos]| -> ~[Pos], greedy: Greedy) -> ~[Pos] {
    let fork = b.push_empty(prev);
    let x = b.reserve(fork); let y = b.reserve(fork);

    let mut tails = ~[];
    let prev_ = &[match greedy {
        NonGreedy => { tails.push(x); y },
        Greedy    => { tails.push(y); x }
    }];

    tails.push_all_move(inner(b, prev_));

    tails
}


fn compile_plus(b: &mut Builder, prev: &[Pos], inner: &Expr, greedy: Greedy) -> ~[Pos] {
    let start = b.states.len();
    let last = compile_expr(b, prev, inner);

    let fork = b.push_empty(last);
    let x = b.reserve(fork); let y = b.reserve(fork);

    let mut tails = ~[];
    let loopback = &[match greedy {
        NonGreedy => { tails.push(x); y },
        Greedy    => { tails.push(y); x }
    }];

    b.connect(loopback, start);

    tails
}
