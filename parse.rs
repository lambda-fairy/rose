/// Parser

use std::char;


#[deriving(ToStr)]
pub enum Expr {
    Empty,
    Range(char, char),
    Concatenate(~[Expr]),
    Alternate(~[Expr]),
    Repeat(~Expr, uint, Option<uint>, Greedy)
}


#[deriving(ToStr)]
pub enum Greedy {
    NonGreedy,
    Greedy
}


/// Parse a regular expression into an AST.
pub fn parse(input: &str) -> Expr {
    let mut s = State::new(input);
    let e = p_alternate(&mut s);
    if s.has_input() {
        // p_alternate() only terminates on an empty string or an extra
        // paren.  Since the string isn't empty, we infer the latter.
        fail!("unbalanced parenthesis")
    } else {
        e
    }
}


/// The parser state.
struct State<'a> {
    input: &'a str,
    prev: Option<&'a str>
}


impl<'a> State<'a> {
    fn new<'a>(input: &'a str) -> State<'a> {
        State {
            input: input,
            prev: None
        }
    }

    /// Consume and return the next character in the input, returning
    /// `None` if empty.
    fn advance(&mut self) -> Option<char> {
        self.prev = Some(self.input);
        if self.has_input() {
            let (c, input_) = self.input.slice_shift_char();
            self.input = input_;
            Some(c)
        } else {
            None
        }
    }

    /// Push the previously read character back onto the input.  This
    /// can only be called immediately after `shift`.
    fn retreat(&mut self) {
        self.input = self.prev.expect("nowhere to retreat");
        self.prev = None;
    }

    /// Return `true` if there is input remaining.
    fn has_input(&self) -> bool {
        self.input.len() > 0
    }
}


fn p_alternate(s: &mut State) -> Expr {
    let mut items: ~[Expr] = ~[];

    loop {
        items.push(p_concatenate(s));
        match s.advance() {
            Some(c) => {
                match c {
                    ')' => { s.retreat(); break },
                    '|' => continue,
                    _ => fail!("something bad happened; it's really bad")
                }
            },
            None => break
        }
    }

    match items {
        [] => Empty,
        [e] => e,
        _ => Alternate(items)
    }
}


fn p_concatenate(s: &mut State) -> Expr {
    let mut items: ~[Expr] = ~[];

    loop {
        match s.advance() {
            Some(c) => match c {
                '|' | ')' => { s.retreat(); break },
                '(' => {
                    // Parse inside the parens
                    let e = p_alternate(s);
                    // Match the closing paren
                    match s.advance() {
                        Some(')') => push_ignore_empty(&mut items, e),
                        _ => fail!("mismatched parenthesis")
                    }
                },
                '.' => items.push(Range('\0', char::MAX)),
                _ => items.push(Range(c, c))
            },
            None => break
        }
    }

    match items {
        [] => Empty,
        [e] => e,
        _ => Concatenate(items)
    }
}


#[inline]
fn push_ignore_empty(items: &mut ~[Expr], e: Expr) {
    match e {
        Empty => {},
        _ => items.push(e)
    }
}
