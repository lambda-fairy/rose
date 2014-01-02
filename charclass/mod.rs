/// Character classes

use std::char;
use std::cmp::{min, max};


/// A range of codepoints.
pub type Range = (char, char);


/// A character class is a non-empty collection of ranges.
pub enum CharClass {
    priv CCOwned(~[Range]),
    priv CCStatic(&'static [Range])
}

impl CharClass {
    /// Create a class from a vector of ranges.
    pub fn new(mut r: ~[Range]) -> CharClass {
        if r.iter().any(|&(lo, hi)| lo > hi) {
            fail!("invalid range");
        }

        r.sort_by(|a, b| {
            // Sort by start value ascending, end value descending
            match a.lo().cmp(&b.lo()) {
                Equal => b.hi().cmp(&a.hi()),
                o => o
            }
        });

        if r.len() > 0 {
            // Coalesce overlapping ranges in place
            let mut cursor = 0;
            for i in range(1, r.len()) {
                if r[cursor].hi() == char::MAX {
                    break
                } else if r[i].lo() <= next_char(r[cursor].hi()) {
                    // Merge the two ranges
                    r[cursor] = (
                        min(r[cursor].lo(), r[i].lo()),
                        max(r[cursor].hi(), r[i].hi()));
                } else {
                    // Create a new range
                    cursor += 1;
                    r[cursor] = r[i];
                }
            }
            r.truncate(cursor + 1); r.shrink_to_fit();
            CCOwned(r)
        } else {
            fail!("char class cannot be empty")
        }
    }

    /// Create a class matching a single code point.
    pub fn from_char(c: char) -> CharClass {
        CharClass::new(~[(c, c)])
    }

    /// Create a class from a range of code points.
    pub fn from_range(lo: char, hi: char) -> CharClass {
        CharClass::new(~[(lo, hi)])
    }

    /// Combine several classes into one that subsumes them all.
    pub fn combine(classes: &[CharClass]) -> CharClass {
        let mut ranges: ~[Range] = ~[];
        for cc in classes.iter() {
            for r in cc.ranges().iter() {
                ranges.push(*r);
            }
        }
        CharClass::new(ranges)
    }

    ///
    /// Get the list of ranges contained in the character class.
    ///
    /// The returned vector is always sorted, with no overlapping
    /// ranges.
    ///
    pub fn ranges<'a>(&'a self) -> &'a [Range] {
        match *self {
            CCOwned(ref r) => {
                let r_borrow: &'a [Range] = *r;
                r_borrow
            },
            CCStatic(r) => {
                let r_borrow: &'a [Range] = r;
                r_borrow
            }
        }
    }

    /// If the class matches a single code point, return it; otherwise
    /// return `None`.
    pub fn to_char(&self) -> Option<char> {
        match self.ranges() {
            [(lo, hi)] if lo == hi => Some(lo),
            _ => None
        }
    }

    /// Determine if the character is contained in the class.
    pub fn includes(&self, c: char) -> bool {
        self.ranges().bsearch(|&(lo, hi)| {
            if c < lo {
                Greater
            } else if hi < c {
                Less
            } else {
                Equal
            }
        }).is_some()
    }

    /// Return the negation of the character class.
    pub fn negate(&self) -> CharClass {
        let ranges = self.ranges();
        let mut result: ~[Range] = ~[];

        let first_lo = ranges[0].lo();
        if first_lo != '\0' {
            result.push(('\0', prev_char(first_lo)));
        }

        let mut last_hi = ranges[0].hi();
        for &(lo, hi) in ranges.slice_from(1).iter() {
            result.push((next_char(last_hi), prev_char(lo)));
            last_hi = hi;
        }

        if last_hi != char::MAX {
            result.push((next_char(last_hi), char::MAX));
        }

        CCOwned(result)
    }
}


trait RangeUtils {
    fn lo(&self) -> char;
    fn hi(&self) -> char;
}

impl RangeUtils for (char, char) {
    #[inline]
    fn lo(&self) -> char {
        let &(lo, _) = self;
        lo
    }

    #[inline]
    fn hi(&self) -> char {
        let &(_, hi) = self;
        hi
    }
}


#[inline]
fn prev_char(c: char) -> char {
    char::from_u32(c as u32 - 1).unwrap()
}

#[inline]
fn next_char(c: char) -> char {
    char::from_u32(c as u32 + 1).unwrap()
}


#[cfg(test)]
mod test {
    use std::char;
    use super::CharClass;

    #[test]
    #[should_fail]
    fn invalid_range() {
        let _ = CharClass::new(~[('b', 'a')]);
    }

    #[test]
    #[should_fail]
    fn empty_class() {
        let _ = CharClass::new(~[]);
    }

    #[test]
    fn class_sorted() {
        let c = CharClass::new(~[('y', 'z'), ('a', 'b')]);
        assert_eq!(c.ranges(), [('a', 'b'), ('y', 'z')]);
    }

    #[test]
    fn class_optimize() {
        let c1 = CharClass::new(~[('a', 'b'), ('c', 'd')]);  // touching
        let c2 = CharClass::new(~[('a', 'c'), ('b', 'd')]);  // overlapping
        let c3 = CharClass::new(~[('a', 'd'), ('b', 'c')]);  // contained
        assert_eq!(c1.ranges(), [('a', 'd')]);
        assert_eq!(c2.ranges(), [('a', 'd')]);
        assert_eq!(c3.ranges(), [('a', 'd')]);
    }

    #[test]
    fn class_includes() {
        // This will fail if the bsearch is incorrect
        let c = CharClass::new(~[('a', 'b'), ('i', 'j'), ('y', 'z')]);
        assert!(c.includes('b'));
        assert!(c.includes('y'));
    }

    #[test]
    fn negate_simple() {
        let c = CharClass::new(~[('T', 's'), ('☻', '♪')]).negate();
        assert_eq!(c.ranges(), [('\0', 'S'), ('t', '☺'), ('♫', char::MAX)]);
    }

    #[test]
    fn negate_edge_case() {
        let c = CharClass::new(~[('\0', char::MAX)]).negate();
        assert_eq!(c.ranges(), []);
    }

    #[test]
    fn issue_1() {
        // See <https://github.com/lfairy/rose/issues/1>
        let c = CharClass::new(~[('c', 'c'), ('d', 'd'), ('z', 'z')]);
        assert_eq!(c.ranges(), [('c', 'd'), ('z', 'z')]);
    }
}

pub mod ascii;
