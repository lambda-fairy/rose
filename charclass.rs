/// Character classes

use std::char;
use std::cmp::{min, max};


/// A range of codepoints.
#[deriving(Eq)]  // Used in tests
pub struct Range {
    priv lo: char,
    priv hi: char
}

impl Range {
    fn new(lo: char, hi: char) -> Range {
        if lo <= hi {
            Range {
                lo: lo,
                hi: hi
            }
        } else {
            fail!("invalid range")
        }
    }
}


/// A character class is a non-empty collection of `Range`s.
pub struct CharClass {
    priv ranges: ~[Range]
}

impl CharClass {
    fn new(mut r: ~[Range]) -> CharClass {
        r.sort_by(|a, b| {
            // Sort by start value ascending, end value descending
            match a.lo.cmp(&b.lo) {
                Equal => b.hi.cmp(&a.hi),
                o => o
            }
        });

        if r.len() > 0 {
            // Coalesce overlapping ranges in place
            let mut cursor = 0;
            for i in range(1, r.len()) {
                if r[cursor].hi == char::MAX {
                    break
                } else if r[i].lo <= next_char(r[cursor].hi) {
                    // Merge the two ranges
                    r[cursor] = Range::new(
                        min(r[cursor].lo, r[i].lo),
                        max(r[cursor].hi, r[i].hi));
                } else {
                    cursor = i;
                }
            }
            r.truncate(cursor + 1); r.shrink_to_fit();
            CharClass { ranges: r }
        } else {
            fail!("char class cannot be empty")
        }
    }
}


#[inline]
fn next_char(c: char) -> char {
    char::from_u32(c as u32 + 1).unwrap()
}


#[cfg(test)]
mod test {
    use super::{Range, CharClass};

    #[test]
    #[should_fail]
    fn invalid_range() {
        let _ = Range::new('b', 'a');
    }

    #[test]
    #[should_fail]
    fn empty_class() {
        let _ = CharClass::new(~[]);
    }

    #[test]
    fn class_sorted() {
        let c = CharClass::new(~[Range::new('y', 'z'), Range::new('a', 'b')]);
        assert_eq!(c.ranges, ~[Range::new('a', 'b'), Range::new('y', 'z')]);
    }

    #[test]
    fn class_optimize() {
        let c = CharClass::new(~[Range::new('a', 'b'), Range::new('c', 'd')]);
        assert_eq!(c.ranges, ~[Range::new('a', 'd')]);
    }
}
