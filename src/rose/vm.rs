//! The regular expression virtual machine.

use std::util::replace;


/// A block of code, ready to execute.
pub type Code = ~[Inst];


/// Represents an instruction in the virtual machine.
pub enum Inst {
    /// No-op.  Used as a placeholder in the compiler.
    Empty,

    /// Match any character in the range, inclusive.
    Range(char, char),

    /// Jump to the specified instruction.
    Jump(uint),

    /// Spawn a new thread with a lower priority.  This is equivalent to
    /// `Split(pc+1, x)` in the original paper.
    Fork(uint),

    /// Spawn a new thread with a higher priority.  This is equivalent
    /// to `Split(x, pc+1)` in the original paper.
    GFork(uint),

    /// We've found a match!
    Match
}

impl Inst {
    /// Replace an `Empty` instruction with something else.  Fails if
    /// the current instruction is not `Empty`.
    pub fn replace(&mut self, other: Inst) {
        match *self {
            Empty => { replace(self, other); },
            _ => fail!(format!("unexpected instruction: {:?}", *self))
        }
    }
}
