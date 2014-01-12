/// ASCII character classes

use super::{CharClass, CCStatic};


pub static digit: CharClass = CCStatic([
    ('\x30', '\x39'),
]);

pub static punct: CharClass = CCStatic([
    ('\x21', '\x2f'),
    ('\x3a', '\x40'),
    ('\x5b', '\x60'),
    ('\x7b', '\x7e'),
]);

pub static space: CharClass = CCStatic([
    ('\x09', '\x0d'),
    ('\x20', '\x20'),
]);

pub static word: CharClass = CCStatic([
    ('\x30', '\x39'),
    ('\x41', '\x5a'),
    ('\x5f', '\x5f'),
    ('\x61', '\x7a'),
]);
