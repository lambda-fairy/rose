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
#[deriving(Clone)]
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
                '?' => {
                    let e = pop_expr(&mut items);
                    items.push(match e {
                        Repeat(_, _, _, NonGreedy) => fail!("multiple repeat"),
                        Repeat(inner, min, max, Greedy) =>
                            Repeat(inner, min, max, NonGreedy),
                        _ => Repeat(~e, 0, Some(1), Greedy)
                    })
                },
                '+' => add_repeat(&mut items, 1, None),
                '*' => add_repeat(&mut items, 0, None),
                '{' => match p_repetition(s) {
                    Some((min, max)) => add_repeat(&mut items, min, max),
                    None => items.push(Range(c, c))
                },
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


#[inline]
fn pop_expr(items: &mut ~[Expr]) -> Expr {
    items.pop_opt().expect("nothing to repeat")
}


#[inline]
fn add_repeat(items: &mut ~[Expr], min: uint, max: Option<uint>) {
    let e = pop_expr(items);
    items.push(match e {
        Repeat(..) => fail!("multiple repeat"),
        _ => Repeat(~e, min, max, Greedy)
    })
}


fn p_repetition(s_outer: &mut State) -> Option<(uint, Option<uint>)> {
    // Clone the parser state, so we can backtrack on failure
    let mut s = s_outer.clone();

    let result = {
        let min = p_number(&mut s);
        match s.advance() {
            Some(',') => {
                let max = p_number(&mut s);
                match s.advance() {
                    // {} or {N,} or {,M} or {N,M}
                    Some('}') => Some((min.unwrap_or(0), max)),
                    _ => None
                }
            },
            Some('}') => match min {
                // {N}
                Some(n) => Some((n, Some(n))),
                _ => None
            },
            _ => None
        }
    };

    if result.is_some() {
        // Only consume input if parsing was successful
        s_outer.clone_from(&s);
    }

    result
}


fn p_number(s: &mut State) -> Option<uint> {
    let mut acc = None;
    loop {
        match s.advance() {
            Some(c) if '0' <= c && c <= '9' => {
                let digit = c as uint - '0' as uint;
                acc = Some(match acc {
                    Some(n) => 10 * n + digit,
                    None => digit
                });
            },
            _ => { s.retreat(); break }
        }
    }
    acc
}
