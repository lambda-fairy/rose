//! Regular expression virtual machine.

use std;
use std::mem::swap;
use std::trie::TrieSet;


/// A compiled regular expression, ready to execute.
pub type Program = ~[Inst];


/// A single instruction in the program.
pub enum Inst {
    /// Jump to all locations in the list simultaneously.  This
    /// corresponds to `jmp` and `split` in the original paper.
    Jump(~[uint]),

    /// Match any code point in the range, inclusive.
    Range(char, char)
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
    priv states: &'a [Inst],
    priv threads: ThreadList,
    priv next: ThreadList,
    priv matched: bool
}

impl<'a> VM<'a> {
    pub fn new(states: &'a Program) -> VM<'a> {
        let mut vm = VM {
            states: *states,
            threads: ThreadList::new(),
            next: ThreadList::new(),
            matched: false
        };

        // Add the initial thread
        vm.matched = follow(Thread::new(0), vm.states, &mut vm.threads);

        vm
    }

    /// Feed a character into the automaton.
    pub fn feed(&mut self, c: char) {
        self.matched = false;

        // Run through all the threads
        for &t in self.threads.iter() {
            match self.states[t.pc] {
                Range(lo, hi) => if lo <= c && c <= hi {
                    if follow(t.with_pc(1 + t.pc), self.states, &mut self.next) {
                        self.matched = true;
                        // Cut off lower priority threads
                        break
                    }
                },
                Jump(..) => unreachable!()
            }
        }

        // Swap the thread buffers
        swap(&mut self.threads, &mut self.next);
        self.next.clear();
    }

    /// Determine if we have a match, given the existing input.
    pub fn is_match(&self) -> bool {
        self.matched
    }
}


/// Add all targets of the given thread to the thread list.
/// Returns `true` if a matching state is reached; otherwise `false`.
fn follow(t: Thread, states: &[Inst], threads: &mut ThreadList) -> bool {
    if t.pc == states.len() {
        true
    } else {
        match states[t.pc] {
            Jump(ref exits) => {
                let mut matched = false;
                for &exit in exits.iter() {
                    matched |= follow(t.with_pc(exit), states, threads);
                }
                matched
            },
            Range(..) => { threads.add(t); false }
        }
    }
}
