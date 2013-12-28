/// ASCII character classes

use super::{CharClass, CCStatic};

pub static alnum: CharClass = CCStatic([
    ('\x30', '\x39'),
    ('\x41', '\x5a'),
    ('\x61', '\x7a'),
]);
pub static alpha: CharClass = CCStatic([
    ('\x41', '\x5a'),
    ('\x61', '\x7a'),
]);
pub static ascii: CharClass = CCStatic([
    ('\x00', '\x7f'),
]);
pub static blank: CharClass = CCStatic([
    ('\x09', '\x09'),
    ('\x20', '\x20'),
]);
pub static cntrl: CharClass = CCStatic([
    ('\x00', '\x1f'),
    ('\x7f', '\x7f'),
]);
pub static digit: CharClass = CCStatic([
    ('\x30', '\x39'),
]);
pub static graph: CharClass = CCStatic([
    ('\x21', '\x7e'),
]);
pub static lower: CharClass = CCStatic([
    ('\x61', '\x7a'),
]);
pub static print: CharClass = CCStatic([
    ('\x20', '\x7e'),
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
pub static upper: CharClass = CCStatic([
    ('\x41', '\x5a'),
]);
pub static word: CharClass = CCStatic([
    ('\x30', '\x39'),
    ('\x41', '\x5a'),
    ('\x5f', '\x5f'),
    ('\x61', '\x7a'),
]);
pub static xdigit: CharClass = CCStatic([
    ('\x30', '\x39'),
    ('\x41', '\x46'),
    ('\x61', '\x66'),
]);
