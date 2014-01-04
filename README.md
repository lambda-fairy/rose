Rose
====

**Rose** is a simple regular expression engine written in Rust.  Its
design is based mostly on the Pike VM, as described by [Russ Cox][1].

[1]: http://swtch.com/~rsc/regexp/regexp2.html

Rose aims to be simple, predictable, and correct above all else.  This
means obscure or unsound features, like [atomic groups][] or
[backreferences][] or [lookaround][], are not supported.  In return, it
does not suffer from the [exponential blowup][] that plagues other
engines.

[atomic groups]: http://www.regular-expressions.info/atomic.html
[backreferences]: http://www.regular-expressions.info/backref.html
[lookaround]: http://www.regular-expressions.info/lookaround.html
[exponential blowup]: http://swtch.com/~rsc/regexp/regexp1.html

See also:

* [rust-re](https://github.com/glennsl/rust-re), by Glenn Slotte
* [regex-rust](https://github.com/ferristseng/regex-rust), by Ferris Tseng
* [Issue #3591](https://github.com/mozilla/rust/issues/3591) – Add regular expressions to std
