use crate::number::{Number, BitsIndexRange, BitsIndex};
use crate::operators::{Operator, operator_assign, operator_sum, operator_unsigned_shift_left};
use log::{info, trace, warn};

#[derive(Debug)]
pub enum NamedAccess {
    Exponent,
    Fraction,
    Carry,
    None,
}

#[derive(Debug)]
pub enum OperandSource {
    RangeSource(BitsIndexRange),
    NamedAccessSource(NamedAccess),
    DirectSource(Number),
}

pub struct ParsingIterator<'a> {
    source: &'a [u8],
    offset: usize,
}

impl<'a> ParsingIterator<'a> {
    pub fn from(source_string: &'a str) -> Result<Self, &str> {
        let bytes = source_string.as_bytes();
        if !bytes.is_ascii() {
            Err("given string is not ascii")
        } else {
            Ok(
                Self {
                    source: bytes,
                    offset: 0,
                }
            )
        }
    }
    pub fn current(&self) -> Option<char> {
        if self.offset < self.source.len() {
            Some(self.source[self.offset] as char)
        } else {
            None
        }
    }
    // TODO remove
    pub fn current_2(&self) -> Option<(char, char)> {
        if self.offset + 1 < self.source.len() {
            Some((self.source[self.offset] as char, self.source[self.offset + 1] as char))
        } else {
            None
        }
    }
    pub fn match_from_current(&self, sequence: &str) -> bool {
        let bytes = sequence.as_bytes();
        for i in 0..sequence.len() {
            if self.offset + i >= self.source.len() || self.source[self.offset + i] != bytes[i] {
                return false
            }
        }
        true
    }
    pub fn next(&mut self) -> Option<char> {
        if self.offset < self.source.len() {
            self.offset += 1;
        }
        self.current()
    }
    pub fn rewind_n(mut self, n: usize) -> Self {
        self.offset += n;
        self
    }
    pub fn rewind(mut self) -> Self {
        self.rewind_n(1)
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
    trace!("syntax_range: ({:?}, {:?})", range_left_index, range_right_index);
    (it_after_index, BitsIndexRange(range_left_index, range_right_index))
}

fn syntax_accessor(it: ParsingIterator) -> Result<(ParsingIterator, Option<OperandSource>), String> {
    match it.current() {
        Some(c) => match c {
            '[' => {
                let (current_it, range) = syntax_range(it.rewind());
                if let Some(c) = current_it.current() {
                    if c == ']' {
                        Ok((current_it.rewind(), Some(OperandSource::RangeSource(range))))
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
    if it.match_from_current("<<") {
        return (it.rewind_n(2), Some(operator_unsigned_shift_left as Operator));
    }
    match it.current() {
        Some('=') => return (it.rewind(), Some(operator_assign as Operator)),
        Some('+') => return (it.rewind(), Some(operator_sum as Operator)),
        _ => (it, None)
    }
}

fn syntax_rvalue(it: ParsingIterator) -> Result<(ParsingIterator, OperandSource), String> {
    trace!("syntax_rvalue, with current '{:?}'", it.current());
    match it.current() {
        Some(c) => match c {
            '[' => match syntax_accessor(it) {
                Ok((it, Some(ops))) => Ok((it, ops)),
                Ok((_, None)) => Err("range access in right value must be correct".to_owned()),
                Err(message) => Err(message)
            },
            '1'..='9' => syntax_number(it, 10),
            '0' => syntax_radix_number(it.rewind()),
            _ => Err("bad number syntax".to_owned())
        }
        None => Err("no second operand".to_owned())
    }
}

fn syntax_radix_number(it: ParsingIterator) -> Result<(ParsingIterator, OperandSource), String> {
    trace!("syntax_radix_number, with current '{:?}'", it.current());
    match it.current() {
        Some(c) => match c {
            'b' | 'B' => syntax_number(it.rewind(), 2),
            'o' | 'O' => syntax_number(it.rewind(), 8),
            'd' | 'D' => syntax_number(it.rewind(), 10),
            'h' | 'H' | 'x' | 'X' => syntax_number(it.rewind(), 16),
            '(' =>
                if let (it_after_arbitrary_radix, Some(radix)) = syntax_index(it.rewind()) {
                    match it_after_arbitrary_radix.current() {
                        Some(')') => {
                            if radix > 37 { // length('0'..='9') + length('a'..='z')
                                return Err(format!("Arbitrary radix is too big - {}, must be small enough to write numbers in it with '0'..'9' + 'a'..'z'", { radix }))
                            }
                            syntax_number(it_after_arbitrary_radix.rewind(), radix as u32)
                        }
                        _ => Err("Arbitrary radix must be closed with ')'".to_owned())
                    }
                } else {
                    Err("Arbitrary radix must not be empty".to_owned())
                }
            _ => Err(format!("bad radix letter '{}'", c))
        },
        None => Ok((it, OperandSource::DirectSource(Number::from("0", 10))))
    }
}

fn syntax_number(mut it: ParsingIterator, radix: u32) -> Result<(ParsingIterator, OperandSource), String> {
    trace!("syntax_number");
    let mut number_literal = String::with_capacity(64);
    while let Some(c) = it.current() {
        match c {
            '0'..='9' => number_literal.push(c),
            'a'..='f' | 'A'..='F' => number_literal.push(c),
            _ => break
        }
        it.next();
    }
    trace!("number '{}'", number_literal);
    Ok((it, OperandSource::DirectSource(Number::from(&number_literal, radix))))
}

pub fn parse(cmd: &str) -> Result<(OperandSource, Operator, OperandSource), String> {
    trace!("parse");
    let space_free_cmd: String = cmd.chars().filter(| c| !c.is_whitespace()).collect();
    let (it_after_first_operand, left_operand_source) = match syntax_accessor(
        ParsingIterator::from(&space_free_cmd).expect("Cannot create iterator for command: ")
    ) {
        Ok((it, Some(ops))) => (it, ops),
        Ok((it, None)) => (it, OperandSource::RangeSource(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit))),
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
    trace!("parse {:?} {:?}", left_operand_source, right_operand_source);
    if it_after_second_operand.current() != None {
        return Err("Could not parse all symbols in command".to_owned())
    }
    Ok((left_operand_source, operator_handler, right_operand_source))
}

#[test]
#[should_panic]
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
        _ => panic!("next value for iterator with empty string must be Some letter")
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
