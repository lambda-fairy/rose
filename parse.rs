/// Parser

use std::char;


#[deriving(ToStr)]
pub enum Expr {
    Empty,
    Range(char, char),
    Concatenate(~[Expr]),
    Alternate(~[Expr]),
    Repeat(~Expr, uint, Option<uint>, Greedy),
}


#[deriving(ToStr)]
pub enum Greedy {
    NonGreedy,
    Greedy,
}


/// Parse a regular expression into an AST.
pub fn parse(s: &str) -> Expr {
    let (e, s_) = p_alternate(s);
    if s_.len() > 0 {
        // p_alternate() only terminates on an empty string or an extra
        // paren.  Since the string isn't empty, we infer the latter.
        fail!("unbalanced parenthesis")
    } else {
        e
    }
}


fn p_alternate<'a>(s_init: &'a str) -> (Expr, &'a str) {
    let mut items: ~[Expr] = ~[];
    let mut s: &'a str = s_init;

    loop {
        let (e, s_) = p_concatenate(s);
        items.push(e);
        s = s_;
        match uncons(s_) {
            Some((c, s_1)) => {
                match c {
                    '|' => { s = s_1; },
                    ')' => break,
                    _ => fail!("something bad happened; it's really bad")
                }
            },
            None => break
        }
    }

    (match items {
        [] => Empty,
        [e] => e,
        _ => Alternate(items)
    }, s)
}


fn p_concatenate<'a>(s_init: &'a str) -> (Expr, &'a str) {
    let mut items: ~[Expr] = ~[];
    let mut s: &'a str = s_init;

    loop { match uncons(s) {
        Some((c, s1)) => match c {
            '|' | ')' => break,
            '(' => {
                // Parse inside the parens
                let (e, s_) = p_alternate(s1);
                // Match the closing paren
                match uncons(s_) {
                    Some((')', s_1)) => {
                        push_ignore_empty(&mut items, e);
                        s = s_1;
                    },
                    _ => fail!("mismatched parenthesis")
                }
            },
            '.' => {
                items.push(Range('\0', char::MAX));
                s = s1;
            },
            _ => {
                items.push(Range(c, c));
                s = s1;
            }
        },
        None => break
    }}

    (match items {
        [] => Empty,
        [e] => e,
        _ => Concatenate(items)
    }, s)
}


#[inline]
fn push_ignore_empty(items: &mut ~[Expr], e: Expr) {
    match e {
        Empty => {},
        _ => items.push(e)
    }
}


/// Return the first element and the rest of a `str`, or `None` if
/// empty.
fn uncons<'a>(s: &'a str) -> Option<(char, &'a str)> {
    if s.len() > 0 {
        Some(s.slice_shift_char())
    } else {
        None
    }
}
