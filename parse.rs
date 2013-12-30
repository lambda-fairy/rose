/// Parser

use std::char;

use charclass::CharClass;
use charclass::ascii;


/// A regular expression AST.
#[deriving(ToStr)]
pub enum Expr {
    Empty,
    Range(char, char),
    Concatenate(~[Expr]),
    Alternate(~[Expr]),
    Repeat(~Expr, uint, Option<uint>, Greedy),
    Capture(~Expr),
    Assertion(AssertType, ~Expr)
}


#[deriving(ToStr)]
pub enum Greedy {
    NonGreedy,
    Greedy
}


#[deriving(ToStr)]
pub enum AssertType {
    Positive,
    Negative
}


/// Parse a regular expression into an AST.  Fails on invalid syntax.
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


/// The parser state.  This tracks the position in the input string.
#[deriving(Clone)]
struct State<'a> {
    input: &'a str,
    prev: Option<&'a str>  // See `State::retreat`
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
    /// can only be called immediately after `advance`.
    fn retreat(&mut self) {
        self.input = self.prev.expect("nowhere to retreat");
        self.prev = None;
    }

    /// Return `true` if there is input remaining.
    fn has_input(&self) -> bool {
        self.input.len() > 0
    }
}


///
/// Parse alternation, e.g. `ducks|geese|swans`.
///
/// An alternation consists of zero or more concatenations, separated by
/// vertical bars `|`.
///
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


/// Parse concatenation, e.g. `abc`.
fn p_concatenate(s: &mut State) -> Expr {
    let mut items: ~[Expr] = ~[];

    loop {
        match s.advance() {
            Some(c) => match c {
                '|' | ')' => { s.retreat(); break },
                '(' => push_ignore_empty(&mut items, p_group(s)),
                '.' => items.push(Range('\0', char::MAX)),
                '\\' => items.push(cc_to_expr(p_escape(s))),
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


///
/// Parse a counted repetition (e.g. `a{2,3}`), sans the opening brace.
///
/// The following syntaxes are accepted:
///
/// * `{N}` – match exactly N repetitions;
/// * `{M,}` – at least M;
/// * `{,N}` – at most N;
/// * `{M,N}` – from M to N inclusive;
/// * `{,}` – zero or more (synonymous with `*`).
///
/// If parsing fails, return `None` without consuming input.  This
/// matches Python behavior, where invalid repetitions are ignored.
///
fn p_repetition(s_outer: &mut State) -> Option<(uint, Option<uint>)> {
    // Clone the parser state, so we can backtrack on failure
    let mut s = s_outer.clone();

    let result = {
        let min = p_number(&mut s);
        match s.advance() {
            Some(',') => {
                let max = p_number(&mut s);
                match s.advance() {
                    // {} or {M,} or {,N} or {M,N}
                    Some('}') => {
                        let min_ = min.unwrap_or(0);
                        if check_repeat(min_, max) {
                            Some((min_, max))
                        } else {
                            fail!("bad repeat interval")
                        }
                    },
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


#[inline]
fn check_repeat(min: uint, max: Option<uint>) -> bool {
    match max {
        Some(max_) => min <= max_,
        None => true
    }
}


/// Parse a non-negative integer, returning `None` on failure.
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


/// Parse a group (e.g. `(hello)`), sans the opening parenthesis.
fn p_group(s: &mut State) -> Expr {
    let result = match s.advance() {
        Some('?') => match s.advance() {
            Some(c) => match c {
                ':' => p_alternate(s),
                '#' => p_comment(s),
                '=' => Assertion(Positive, ~p_alternate(s)),
                '!' => Assertion(Negative, ~p_alternate(s)),
                _ => fail!(format!("unknown extension: ?{}", c))
            },
            None => fail!("unexpected end of pattern")
        },
        _ => { s.retreat(); Capture(~p_alternate(s)) }
    };

    // Match the closing paren
    match s.advance() {
        Some(')') => result,
        _ => fail!("mismatched parenthesis")
    }
}


/// Consume all input up to the first closing parenthesis, and return
/// `Empty`.
fn p_comment(s: &mut State) -> Expr {
    loop {
        match s.advance() {
            Some(c) if c != ')' => continue,
            _ => { s.retreat(); break }
        }
    }
    Empty
}


/// Parse an escape sequence (e.g. `\d`), sans the leading backslash.
fn p_escape(s: &mut State) -> CharClass {
    match s.advance() {
        Some(c) => match c {
            'n' => CharClass::from_char('\n'),
            'r' => CharClass::from_char('\r'),
            't' => CharClass::from_char('\t'),

            'd' => ascii::digit,
            's' => ascii::space,
            'w' => ascii::word,

            'x' => p_hex_escape(s, 2),
            'u' => p_hex_escape(s, 4),
            'U' => p_hex_escape(s, 8),

            _ => fail!("invalid escape")
        },
        None => fail!("invalid escape")
    }
}


fn p_hex_escape(s: &mut State, n_digits: uint) -> CharClass {
    let mut acc = 0u32;
    for _ in range(0, n_digits) {
        acc = 16 * acc + match s.advance() {
            Some(c) => c.to_digit(16).expect("invalid escape") as u32,
            _ => fail!("invalid escape")
        };
    }
    let c = char::from_u32(acc).expect("character out of range");
    CharClass::from_char(c)
}


/// Reify a character class as an `Expr`.
fn cc_to_expr(cc: CharClass) -> Expr {
    Alternate(cc.ranges().iter().map(|&(lo, hi)| Range(lo, hi)).collect())
}
