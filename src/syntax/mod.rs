use crate::number::{Number, BitsIndexRange, BitsIndex};
use crate::operators::{Operator, operator_assign, operator_sum};
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

pub struct ParsingIterator {
    iterator: Option<Box<dyn Iterator<Item = char>>>,
    current: Option<char>,
    offset: usize,
}

impl ParsingIterator {
    pub fn from(s: &str) -> Self {
        let mut iter = s.chars().collect::<Vec<_>>().into_iter();
        let current = iter.next();
        Self {
            iterator: Some(Box::new(iter)),
            current,
            offset: 1,
        }
    }
    pub fn current(&self) -> Option<char> {
        self.current
    }
    pub fn next(&mut self) -> Option<char> {
        match &mut self.iterator {
            Some(it) => {
                self.offset += 1;
                self.current = it.next();
                self.current
            },
            None => None
        }
    }
    pub fn skip(mut self, n: usize) -> Self {
        let mut count = n;
        while count > 0 {
            self.current = match &mut self.iterator {
                Some(i) => {
                    count -= 1;
                    self.offset += 1;
                    i.next()
                },
                None => {
                    count -= 1;
                    None
                }
            }
        }
        self
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
            match syntax_index(it_after_index.skip(1)) {
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
                let (current_it, range) = syntax_range(it.skip(1));
                if let Some(c) = current_it.current() {
                    if c == ']' {
                        Ok((current_it.skip(1), Some(OperandSource::RangeSource(range))))
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
        Some('=') => return (it.skip(1), Some(operator_assign as Operator)),
        Some('+') => return (it.skip(1), Some(operator_sum as Operator)),
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
            '0' => syntax_radix_number(it.skip(1)),
            _ => Err("bad number syntax".to_owned())
        }
        None => Err("no second operand".to_owned())
    }
}

fn syntax_radix_number(it: ParsingIterator) -> Result<(ParsingIterator, OperandSource), String> {
    trace!("syntax_radix_number, with current '{:?}'", it.current());
    match it.current() {
        Some(c) => match c {
            'b' | 'B' => syntax_number(it.skip(1), 2),
            'o' | 'O' => syntax_number(it.skip(1), 8),
            'd' | 'D' => syntax_number(it.skip(1), 10),
            'h' | 'H' | 'x' | 'X' => syntax_number(it.skip(1), 16),
            '(' =>
                if let (it_after_arbitrary_radix, Some(radix)) = syntax_index(it.skip(1)) {
                    match it_after_arbitrary_radix.current() {
                        Some(')') => {
                            if radix > 37 { // length('0'..='9') + length('a'..='z')
                                return Err(format!("Arbitrary radix is too big - {}, must be small enough to write numbers in it with '0'..'9' + 'a'..'z'", { radix }))
                            }
                            syntax_number(it_after_arbitrary_radix.skip(1), radix as u32)
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
    let (it_after_first_operand, left_operand_source) = match syntax_accessor(ParsingIterator::from(&space_free_cmd)) {
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
fn syntax_index_test() {
    match syntax_index(ParsingIterator::from("")) {
        (_, Some(_)) => panic!("syntax_index() must return no value if source string was empty"),
        (it, None) if it.current() == None => (), // success
        _ => panic!("syntax_index() must exhaust iterator with empty string")
    }
    match syntax_index(ParsingIterator::from("0")) {
        (it, Some(parsed)) if it.current() == None => assert_eq!(0, parsed),
        (_, None) => panic!("syntax_index() must parse 0"),
        _ => panic!("syntax_index() must exhaust iterator with string containing one number")
    }
    match syntax_index(ParsingIterator::from(&usize::MAX.to_string())) {
        (it, Some(parsed)) if it.current() == None => assert_eq!(usize::MAX, parsed),
        (_, None) => panic!("syntax_index() must parse usize::MAX"),
        _ => panic!("syntax_index() must exhaust iterator with empty string")
    }
    let test_string = "a1fasd";
    match syntax_index(ParsingIterator::from(test_string)) {
        (_, Some(_)) => panic!("syntax_index() must return no value if source string has no leading digits"),
        (mut it, None) => {
            let mut count = 0;
            while let Some(_) = it.next() {
                count += 1;
            }
            assert_eq!(test_string.len() - 1, count);
        }
    }
    match syntax_index(ParsingIterator::from("123 adf")) {
        (mut it, Some(parsed)) => {
            assert_eq!(123, parsed);
            assert_eq!(Some(' '), it.current());
            let mut count = 0;
            while let Some(_) = it.next() {
                count += 1;
            }
            assert_eq!(" adf".len() - 1, count);
        },
        (_, None) => panic!("syntax_index() must return value if source string has leading digits")
    }
}
