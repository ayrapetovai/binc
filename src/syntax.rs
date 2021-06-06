use crate::number::{Number, BitsIndexRange, BitsIndex};
use crate::operators::Operator;
use crate::operators::operator_show_help;
use crate::operators::operator_assign;
use crate::operators::operator_sum;
use crate::operators::operator_sub;
use crate::operators::operator_mul;
use crate::operators::operator_div;
use crate::operators::operator_mod;
use crate::operators::operator_signed_shift_left;
use crate::operators::operator_signed_shift_right;
use crate::operators::operator_unsigned_shift_right;
use crate::operators::operator_int_bits_width;
use crate::operators::operator_signed;
use crate::operators::operator_unsigned;
use crate::operators::operator_greater;
use crate::operators::operator_less;
use log::{info, trace, warn};
use std::iter::FromIterator;

#[derive(Debug)]
pub enum NamedAccess {
    Exponent,
    Fraction,
    Carry,
    None,
}

#[derive(Debug)]
pub enum LeftOperandSource {
    RangeSource(BitsIndexRange),
    NamedAccessSource(NamedAccess),
}

#[derive(Debug)]
pub enum RightOperandSource {
    RangeSource(BitsIndexRange),
    NamedAccessSource(NamedAccess),
    DirectSource(Number),
    Empty,
}

pub struct ParsingIterator {
    source: Vec<char>,
    offset: usize,
}

impl ParsingIterator {
    pub fn from(source_string: &str) -> Result<Self, &str> {
        let source: Vec<char> = source_string.chars().collect();
        // skip leading whitespaces
        let mut offset = 0;
        while offset < source.len() && (source[offset]).is_whitespace() {
            offset += 1;
        }
        Ok(Self { source, offset })
    }
    pub fn current(&self) -> Option<char> {
        if self.offset < self.source.len() {
            Some(self.source[self.offset])
        } else {
            None
        }
    }
    pub fn match_from_current(&self, sequence: &str) -> bool {
        let bytes: Vec<char> = sequence.chars().collect();
        for i in 0..sequence.len() {
            if self.offset + i >= self.source.len() || self.source[self.offset + i] != bytes[i] &&
                !(self.source[self.offset + i]).is_whitespace()
            {
                return false
            }
        }
        true
    }
    pub fn next(&mut self) -> Option<char> {
        if self.offset < self.source.len() {
            self.offset += 1;
        }
        while self.offset < self.source.len() && (self.source[self.offset]).is_whitespace() {
            self.offset += 1;
        }
        self.current()
    }
    pub fn rewind_n(mut self, n: usize) -> Self {
        let mut skip_counter = 0;
        while self.offset < self.source.len() && skip_counter < n {
            self.offset += 1;
            if self.offset < self.source.len() && !(self.source[self.offset]).is_whitespace() {
                skip_counter += 1;
            }
        }
        self
    }
    pub fn rewind(self) -> Self {
        self.rewind_n(1)
    }
    fn rest(&self) -> &[char] {
        &self.source[self.offset..]
    }
}

pub fn syntax_index(mut it: ParsingIterator) -> (ParsingIterator, Option<usize>) {
    let mut acc = 0usize;
    match it.current() {
        Some(c) => {
            if !('0'..='9').contains(&c) {
                return (it, None);
            }
        }
        None => return (it, None)
    }
    while let Some(c) = it.current() {
        match c {
            '0'..='9' => acc = acc * 10 + (c as usize - '0' as usize),
            _ => break
        }
        it.next();
    }
    (it, Some(acc))
}

fn syntax_range(it: ParsingIterator) -> (ParsingIterator, BitsIndexRange) {
    let (it_after_index, range_left_index) = match syntax_index(it) {
        (it, Some(i)) => (it, BitsIndex::IndexedBit(i)),
        (it, None) => (it, BitsIndex::HighestBit)
    };
    let (it_after_index, range_right_index) = if let Some(c) = it_after_index.current() {
        if c == ':' {
            match syntax_index(it_after_index.rewind()) {
                (it, Some(i)) => (it, BitsIndex::IndexedBit(i)),
                (it, None) => (it, BitsIndex::LowestBit)
            }
        } else {
            (it_after_index, range_left_index)
        }
    } else {
        (it_after_index, BitsIndex::LowestBit)
    };
    trace!("syntax_range: resulting range ({:?}, {:?})", range_left_index, range_right_index);
    (it_after_index, BitsIndexRange(range_left_index, range_right_index))
}

fn syntax_accessor(it: ParsingIterator) -> Result<(ParsingIterator, Option<BitsIndexRange>), String> {
    match it.current() {
        Some(c) => match c {
            '[' => {
                let (current_it, range) = syntax_range(it.rewind());
                if let Some(c) = current_it.current() {
                    if c == ']' {
                        Ok((current_it.rewind(), Some(range)))
                    } else {
                        Err("Accessor [] is not closed with ']'".to_owned())
                    }
                } else {
                    Err("Command is not complete".to_owned())
                }
            }
            // TODO named ranges
            _ => Ok((it, None))
        }
        None => Ok((it, None))
    }
}

fn syntax_operator(it: ParsingIterator) -> (ParsingIterator, Option<Operator>) {
    match it.current() {
        Some('u') if it.match_from_current("unsigned") => (it.rewind_n(8), Some(operator_unsigned as Operator)),
        Some('s') if it.match_from_current("signed") => (it.rewind_n(6), Some(operator_signed as Operator)),
        Some('h') if it.match_from_current("help") => (it.rewind_n(4), Some(operator_show_help as Operator)),
        Some('i') if it.match_from_current("int") => (it.rewind_n(3), Some(operator_int_bits_width as Operator)),
        Some('>') if it.match_from_current(">>>") => (it.rewind_n(3), Some(operator_unsigned_shift_right as Operator)),
        Some('>') if it.match_from_current(">>") => (it.rewind_n(2), Some(operator_signed_shift_right as Operator)),
        Some('<') if it.match_from_current("<<") => (it.rewind_n(2), Some(operator_signed_shift_left as Operator)),
        Some('?') => (it.rewind(), Some(operator_show_help as Operator)),
        Some('=') => (it.rewind(), Some(operator_assign as Operator)),
        Some('+') => (it.rewind(), Some(operator_sum as Operator)),
        Some('-') => (it.rewind(), Some(operator_sub as Operator)),
        Some('*') => (it.rewind(), Some(operator_mul as Operator)),
        Some('/') => (it.rewind(), Some(operator_div as Operator)),
        Some('%') => (it.rewind(), Some(operator_mod as Operator)),
        Some('>') => (it.rewind(), Some(operator_greater as Operator)),
        Some('<') => (it.rewind(), Some(operator_less as Operator)),
        _ => (it, None)
    }
}

fn syntax_negative_number(it: ParsingIterator) -> Result<(ParsingIterator, RightOperandSource), String> {
    match it.current() {
        Some(c) => match c {
            '1'..='9' => syntax_number(it, 10, true),
            '0' => syntax_radix_number(it.rewind(), true),
            _ => Err("bad number syntax".to_owned())
        }
        None => Err("bad negative number syntax".to_owned())
    }
}

// FIXME refactor parsing numbers
fn syntax_rvalue(it: ParsingIterator) -> Result<(ParsingIterator, RightOperandSource), String> {
    trace!("syntax_rvalue: with current symbol '{:?}'", it.current());
    match it.current() {
        Some(c) => match c {
            '[' => match syntax_accessor(it) {
                Ok((it, Some(ops))) => Ok((it, RightOperandSource::RangeSource(ops))),
                Ok((_, None)) => Err("range access in right value must be correct".to_owned()),
                Err(message) => Err(message)
            },
            '1'..='9' => syntax_number(it, 10, false),
            '0' => syntax_radix_number(it.rewind(), false),
            '-' => syntax_negative_number(it.rewind()),
            _ => Err(format!("number or range had been expected, but '{}' was found", String::from_iter(it.rest())).to_owned())
        }
        None => Ok((it, RightOperandSource::Empty))
    }
}

fn syntax_radix_number(it: ParsingIterator, is_negative: bool) -> Result<(ParsingIterator, RightOperandSource), String> {
    trace!("syntax_radix_number: with current symbol '{:?}'", it.current());
    match it.current() {
        Some(c) => match c {
            'b' | 'B' => syntax_number(it.rewind(), 2, is_negative),
            'o' | 'O' => syntax_number(it.rewind(), 8, is_negative),
            'd' | 'D' => syntax_number(it.rewind(), 10, is_negative),
            'h' | 'H' | 'x' | 'X' => syntax_number(it.rewind(), 16, is_negative),
            '(' =>
                if let (it_after_arbitrary_radix, Some(radix)) = syntax_index(it.rewind()) {
                    match it_after_arbitrary_radix.current() {
                        Some(')') => {
                            if radix > 37 { // length('0'..='9') + length('a'..='z')
                                return Err(format!("Arbitrary radix is too big - {}, must be small enough to write numbers in it with '0'..'9' + 'a'..'z'", { radix }))
                            }
                            syntax_number(it_after_arbitrary_radix.rewind(), radix as u32, is_negative)
                        }
                        _ => Err("Arbitrary radix must be closed with ')'".to_owned())
                    }
                } else {
                    Err("Arbitrary radix must not be empty".to_owned())
                }
            _ => Err(format!("bad radix letter '{}'", c))
        },
        None => Ok((it, RightOperandSource::DirectSource(Number::from("0", 10).unwrap())))
    }
}

fn syntax_number(mut it: ParsingIterator, radix: u32, is_negative: bool) -> Result<(ParsingIterator, RightOperandSource), String> {
    trace!("syntax_number: with current symbol {:?}, radix {}", it.current(), radix);
    let mut number_literal = String::with_capacity(64);
    if is_negative {
        number_literal.push('-')
    }
    while let Some(c) = it.current() {
        match c {
            '0'..='9' => number_literal.push(c),
            'a'..='f' | 'A'..='F' => number_literal.push(c),
            _ => break
        }
        it.next();
    }
    trace!("syntax_number: number literal '{}'", number_literal);
    match Number::from(&number_literal, radix) {
        Ok(number) => Ok((it, RightOperandSource::DirectSource(number))),
        Err(message) => Err(message)
    }
}

pub fn parse(cmd: &str) -> Result<(LeftOperandSource, Operator, RightOperandSource), String> {
    trace!("parse: command '{}'", cmd);
    let (it_after_first_operand, left_operand_source) = match syntax_accessor(
        match ParsingIterator::from(&cmd) {
            Err(msg) => return Err(format!("Cannot create parser for command '{}': ", cmd) + msg),
            Ok(it) => it
        }
    ) {
        Ok((it, Some(ops))) => (it, LeftOperandSource::RangeSource(ops)),
        Ok((it, None)) => (it, LeftOperandSource::RangeSource(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit))),
        Err(message) => return Err(message)
    };
    let (it_after_operator, operator_handler) = match syntax_operator(it_after_first_operand) {
        (it, Some(h)) => (it, h),
        (it, None) => (it, operator_assign as Operator)
    };
    let (it_after_second_operand, right_operand_source) = match syntax_rvalue(it_after_operator) {
        Ok((it, rop)) => (it, rop),
        Err(message) => return Err(message)
    };
    trace!("parse: resulting operands {:?} {:?}", left_operand_source, right_operand_source);
    if it_after_second_operand.current() != None {
        return Err(format!("Could not parse all symbols in command, left '{}'", String::from_iter(it_after_second_operand.rest())).to_owned())
    }
    Ok((left_operand_source, operator_handler, right_operand_source))
}

#[test]
fn parsing_iterator_from_non_ascii() {
    ParsingIterator::from("これは変な文です").unwrap();
}

#[test]
fn parsing_iterator_from() {
    let mut it = ParsingIterator::from("").unwrap();
    assert_eq!(0, it.offset);
    match it.current() {
        None => {}, // success
        _ => panic!("next value for iterator with empty string must be None")
    }
    assert_eq!(0, it.offset);
    match it.next() {
        None => {}, // success
        _ => panic!("next value for iterator with empty string must be None")
    }
    assert_eq!(0, it.offset);

    let mut it = ParsingIterator::from("abc").unwrap();
    assert_eq!(0, it.offset);
    match it.current() {
        Some('a') => {}, // success
        Some(_) => panic!("current value of fresh iterator must be the first letter in source string"),
        None => panic!("current value of fresh iterator with non-empty string must be Some letter")
    }
    match it.next() {
        Some('b') => {}, // success
        x => panic!("next value for iterator with empty string must be Some letter, was {:?}, index {}", x, it.offset)
    }
    assert_eq!(1, it.offset);
}

#[test]
fn parsing_iterator_match_from_current() {
    let mut it = ParsingIterator::from("abc").unwrap();
    assert_eq!(0, it.offset);
    assert!(it.match_from_current("a"));
    assert!(it.match_from_current("ab"));
    assert!(it.match_from_current("abc"));

    assert_eq!(Some('b'), it.next());
    assert_eq!(Some('b'), it.current());

    assert!(!it.match_from_current("a"));
    assert!(it.match_from_current("b"));
    assert!(it.match_from_current("bc"));

    assert_eq!(Some('c'), it.next());
    assert_eq!(Some('c'), it.current());

    assert!(!it.match_from_current("b"));
    assert!(it.match_from_current("c"));

    assert_eq!(None, it.next());
    assert_eq!(None, it.current());
    assert!(!it.match_from_current("c"));
    assert!(!it.match_from_current("abc"));
}

#[test]
fn parsing_iterator_skip_whitespaces() {
    let mut it = ParsingIterator::from("   ").unwrap();
    assert_eq!(None, it.current());
    assert_eq!(None, it.next());

    let mut it = ParsingIterator::from("  a  ").unwrap();
    assert_eq!(Some('a'), it.current());
    assert_eq!(None, it.next());

    let mut it = ParsingIterator::from(" 12\t 3  ").unwrap();
    assert!(it.match_from_current("123"));

    while let Some(c) = it.current() {
        assert_ne!(' ', c);
        assert_ne!('\t', c);
        it.next();
    }

    let mut it = ParsingIterator::from(" 12\t 3  ").unwrap();
    assert_ne!(Some(' '), it.current());
    while let Some(c) = it.next() {
        assert_ne!(' ', c);
        assert_ne!('\t', c);
    }
    let mut it = ParsingIterator::from(" 12\t 3abc").unwrap();
    let it = it.rewind_n(3);
    assert_eq!(6, it.offset);
    assert!(it.match_from_current("abc"));

    let mut it = ParsingIterator::from(" \t[ 0 ]   =  1").unwrap();
    let pat = "[0]=1".as_bytes();
    assert_eq!(Some(*pat.first().unwrap() as char), it.current());
    for i in 1..pat.len() {
        it = it.rewind();
        assert_eq!(Some(pat[i] as char), it.current());
    }
}

#[test]
fn syntax_index_test() {
    match syntax_index(ParsingIterator::from("").unwrap()) {
        (_, Some(_)) => panic!("syntax_index() must return no value if source string was empty"),
        (it, None) if it.current() == None => (), // success
        _ => panic!("syntax_index() must exhaust iterator with empty string")
    }
    match syntax_index(ParsingIterator::from("0").unwrap()) {
        (it, Some(parsed)) if it.current() == None => assert_eq!(0, parsed),
        (_, None) => panic!("syntax_index() must parse 0"),
        _ => panic!("syntax_index() must exhaust iterator with string containing one number")
    }
    match syntax_index(ParsingIterator::from(&usize::MAX.to_string()).unwrap()) {
        (it, Some(parsed)) if it.current() == None => assert_eq!(usize::MAX, parsed),
        (_, None) => panic!("syntax_index() must parse usize::MAX"),
        _ => panic!("syntax_index() must exhaust iterator with empty string")
    }
    let test_string = "a1fasd";
    match syntax_index(ParsingIterator::from(test_string).unwrap()) {
        (_, Some(_)) => panic!("syntax_index() must return no value if source string has no leading digits"),
        (mut it, None) => {
            let mut count = 0;
            while let Some(_) = it.next() {
                count += 1;
            }
            assert_eq!(test_string.len() - 1, count);
        }
    }
    match syntax_index(ParsingIterator::from("123adf").unwrap()) {
        (mut it, Some(parsed)) => {
            assert_eq!(123, parsed);
            assert_eq!(Some('a'), it.current());
            let mut count = 0;
            while let Some(_) = it.next() {
                count += 1;
            }
            assert_eq!("adf".len() - 1, count);
        },
        (_, None) => panic!("syntax_index() must return value if source string has leading digits")
    }
}
