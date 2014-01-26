//! Regular expression virtual machine.

use std;
use std::trie::TrieSet;
use std::util::{replace, swap};


/// A block of code, ready to execute.
pub type Code = ~[Inst];


/// An instruction in the virtual machine.
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


struct Thread {
    pc: uint
}

impl Thread {
    fn new(pc: uint) -> Thread {
        Thread {
            pc: pc
        }
    }

    fn next_pc(&self) -> Thread {
        Thread {
            pc: 1 + self.pc
        }
    }

    fn with_pc(&self, pc: uint) -> Thread {
        Thread {
            pc: pc
        }
    }
}


struct ThreadList {
    threads: ~[Thread],
    indices: TrieSet
}

impl ThreadList {
    /// Create a new, empty, `ThreadList`.
    fn new() -> ThreadList {
        ThreadList {
            threads: ~[],
            indices: TrieSet::new()
        }
    }

    /// Clear all threads from the list.
    fn clear(&mut self) {
        self.threads.clear();
        self.indices.clear();
    }

    /// Add a thread to the list, if one with the same `pc` is not
    /// already present.
    fn add(&mut self, t: Thread) {
        if self.indices.insert(t.pc) {
            self.threads.push(t);
        }
    }

    /// Iterate over the list of threads.
    fn iter<'a>(&'a self) -> std::vec::Items<'a, Thread> {
        self.threads.iter()
    }
}


/// A regular expression virtual machine, loosely based on the Pike VM.
pub struct VM<'a> {
    priv code: &'a [Inst],
    priv threads: ThreadList,
    priv next: ThreadList,

    /// Whether we are in a matching state, i.e., one of the threads
    /// is on a `Match` instruction.
    priv matched: bool
}

impl<'a> VM<'a> {
    pub fn new(code: &'a [Inst]) -> VM<'a> {
        let mut vm = VM {
            code: code,
            threads: ThreadList::new(),
            next: ThreadList::new(),
            matched: false
        };

        // Add the initial thread
        follow(Thread::new(0), vm.code, &mut vm.threads);

        vm
    }

    /// Feed a character into the automaton.
    pub fn feed(&mut self, c: char) {
        self.matched = false;

        // Run through all the threads
        for &t in self.threads.iter() {
            match self.code[t.pc] {
                Range(lo, hi) => if lo <= c && c <= hi {
                    if follow(t.next_pc(), self.code, &mut self.next) {
                        self.matched = true;
                        // Cut off lower priority threads
                        break;
                    }
                },
                Match => (),
                _ => unreachable!()
            }
        }

        // Swap the thread buffers
        swap(&mut self.threads, &mut self.next);
        self.next.clear();
    }

    /// Determine if we have a match, given the existing input.
    pub fn is_match(&self) -> bool {
        assert!(self.is_match_slow() == self.matched,
                "the matching flag tells the truth");
        self.matched
    }

    /// A slower, but guaranteed correct, version of `is_match`.  Used
    /// for testing.
    fn is_match_slow(&self) -> bool {
        // Since `feed` breaks on finding a match, we're more likely to
        // spot it at the end of the list.  Hence `rev`.
        self.threads.iter().rev().any(|t| {
            match self.code[t.pc] {
                Match => true,
                _ => false
            }
        })
    }
}


/// Recursively add the targets of the current instruction to the list.
/// Returns `true` if a `Match` is encountered; otherwise `false`.
fn follow(t: Thread, code: &[Inst], threads: &mut ThreadList) -> bool {
    macro_rules! recurse(
        ($t:expr) => (follow($t, code, threads))
    )

    // Use bitwise OR (`|`), not logical OR (`||`), because we don't
    // want to short circuit.
    match code[t.pc] {
        Empty => recurse!(t.next_pc()),
        Jump(there) => recurse!(t.with_pc(there)),
        Fork(other) => recurse!(t.next_pc()) | recurse!(t.with_pc(other)),
        GFork(other) => recurse!(t.with_pc(other)) | recurse!(t.next_pc()),
        Range(..) => { threads.add(t); false },
        Match => { threads.add(t); true }
    }
}
