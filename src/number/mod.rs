use std::fmt::{Display, Formatter};
use log::trace;

#[derive(Debug, Copy, Clone)]
pub enum BitsIndex {
    HighestBit,
    LowestBit,
    IndexedBit(usize)
}

#[derive(Debug)]
pub struct BitsIndexRange(pub BitsIndex, pub BitsIndex);

#[derive(Debug, Copy, Clone)]
pub enum NumberType {
    Integer,
    Float,
    Fixed,
}

#[derive(Debug)]
pub struct Number {
    buffer: Vec<bool>,
    number_type: NumberType,
    is_signed: bool,
    carry: bool,
}

impl Number {
    pub fn new(number_type: NumberType, is_signed: bool, max_size: usize) -> Self {
        Self {
            buffer: vec![false; max_size],
            number_type,
            is_signed,
            carry: false,
        }
    }
    pub fn from(number_literal: &str, radix: u32) -> Self {
        trace!("Number::from: parsing literal '{}', radis {}", number_literal, radix);
        // TODO floating point, + and - notations
        let mut new_number = Self {
            buffer: vec![false; 1],
            number_type: if number_literal.contains(".") { NumberType::Float } else { NumberType::Integer },
            is_signed: if number_literal.starts_with("-") { true } else { false },
            carry: false,
        };
        for c in number_literal.chars() {
            let n = match c {
                '0'..='9' => c as u32 - '0' as u32,
                'a'..='z' => c as u32 - 'a' as u32 + 10,
                'A'..='Z' => c as u32 - 'A' as u32 + 10,
                _ => panic!("letter {} cannot represent a digit", c)
            };
            if n < radix {
                new_number.mul_number(radix);
                new_number.add_number(n);
            } else {
                panic!("Letter '{}' cannot be used for number notation in base {}", c, radix);
            }
        }
        trace!("Number::from: parsed buffer {:?}", new_number.buffer);
        let mut buf = vec![false; power_of_two(new_number.buffer.len())];
        for i in 0..new_number.buffer.len() {
            buf[i] = new_number.buffer[i];
        }
        new_number.buffer = buf;
        trace!("Number::from: reajusted buffer is {:?}", new_number.buffer);
        new_number
    }

    fn add_number(&mut self, additive: u32) {
        trace!("add_number: additive is {}, number.buffer.len is {}", additive, self.buffer.len());
        let additive_length = 32usize - additive.leading_zeros() as usize;
        let mut i = 0;
        let mut additive_mask = 0x1u32;
        let mut carry = false;
        while !(!carry && i >= additive_length) {
            if i >= self.buffer.len() {
                self.buffer.push(false);
            }
            let current_mul_bit = additive & additive_mask != 0;
            trace!("add_number: adding bit {} of additive to ith bit of number {}", current_mul_bit,  self.buffer[i]);
            let new_carry = ((current_mul_bit ^ self.buffer[i]) & carry) | (current_mul_bit & self.buffer[i]);
            trace!("add_number: new_carry {}", new_carry);
            self.buffer[i] = (current_mul_bit ^ self.buffer[i]) ^ carry;
            trace!("add_number: new ith bit {}", self.buffer[i]);
            carry = new_carry;
            i += 1;
            additive_mask <<= 1;
        }
        self.carry = carry;
    }

    fn mul_number(&mut self, multiplier: u32) {
        let additive = self.buffer.clone();
        for _ in 1..multiplier {
            self.add_bools(&additive[..]);
        }
    }

    // fixme make private
    fn add_bools(&mut self, additive: &[bool]) {
        let mut carry = false;
        if additive.len() > self.buffer.len() {
            self.buffer.reserve(additive.len() - self.buffer.len());
        }
        let mut i = 0;
        while !(!carry && i >= additive.len()) {
            if i >= self.buffer.len() {
                self.buffer.push(false);
            }
            let additive_ith = if i >= additive.len() { false } else { additive[i] };
            let new_carry = ((additive_ith ^ self.buffer[i]) && carry) || (additive_ith && self.buffer[i]);
            self.buffer[i] = (additive_ith ^ self.buffer[i]) ^ carry;
            carry = new_carry;
            i += 1;
        }
    }

    pub fn add_bits(&mut self, additive: &[bool]) {
        let mut carry = false;
        let mut i = 0;
        while !(!carry && i >= additive.len()) && i < self.buffer.len() {
            let additive_ith = if i >= additive.len() { false } else { additive[i] };
            let new_carry = ((additive_ith ^ self.buffer[i]) && carry) || (additive_ith && self.buffer[i]);
            self.buffer[i] = (additive_ith ^ self.buffer[i]) ^ carry;
            carry = new_carry;
            i += 1;
        }
    }

    pub fn assign_value(&mut self, other: &Number) {
        self.buffer = Vec::with_capacity(other.buffer.len());
        for i in 0..other.buffer.len() {
            self.buffer.push(other.buffer[i]);
        }
        self.number_type = other.number_type;
        self.is_signed = other.is_signed;
        self.carry = false;
    }

    pub fn get_bits(&self, range: BitsIndexRange) -> &[bool] {
        let high_order_bit_index = match range.0 {
            BitsIndex::IndexedBit(i) => i,
            BitsIndex::HighestBit => &self.buffer.len()  - 1,
            BitsIndex::LowestBit => 0

        };
        let low_order_bit_index = match range.1 {
            BitsIndex::IndexedBit(i) => i,
            BitsIndex::HighestBit => &self.buffer.len() - 1,
            BitsIndex::LowestBit => 0
        };
        &self.buffer[low_order_bit_index..=high_order_bit_index]
    }

    pub fn set_bits(&mut self, range: BitsIndexRange, source_bits: &[bool]) {
        trace!("number.set_bits: {:?}", range);
        let high_index = match range.0 {
            BitsIndex::IndexedBit(i) => i,
            BitsIndex::HighestBit => &self.buffer.len()  - 1,
            BitsIndex::LowestBit => 0

        };
        let low_index = match range.1 {
            BitsIndex::IndexedBit(i) => i,
            BitsIndex::HighestBit => &self.buffer.len()  - 1,
            BitsIndex::LowestBit => 0
        };
        let mut source_index = 0;
        let mut target_index = low_index;
        while target_index <= high_index {
            if target_index >= self.buffer.len() {
                self.buffer.push(false);
            }
            if source_index < source_bits.len() {
                self.buffer[target_index] = source_bits[source_index];
                source_index += 1;
                target_index += 1;
            } else {
                break;
            }
        }
    }

    pub fn max_size(&self) -> usize {
        self.buffer.len()
    }

    pub fn signed_extend_to(&mut self, new_max_size: usize) {
        let mut buf = vec![false; new_max_size];
        for i in 0..self.buffer.len() {
            buf[i] = self.buffer[i];
        }
        let last_bit = *self.buffer.last().unwrap();
        if self.is_signed && self.buffer.len() < buf.len() {
            for i in self.buffer.len()..buf.len() {
                buf[i] = last_bit;
            }
        }
        self.buffer = buf;
    }

    fn to_string(&self, radix: u32) -> String {
        if radix != 2 {
            todo!("implement radix base formatting");
        }
        let mut res = String::with_capacity(self.buffer.len());
        for b in self.buffer.iter().take(self.buffer.len()).rev() {
            match b {
                true => res.push('1'),
                false => res.push('0')
            }
        }
        res
    }
}

// TODO make an arithmetic expression
fn power_of_two(length: usize) -> usize {
    match length {
        0..=8 => 8,
        9..=16 => 16,
        17..=32 => 32,
        33..=64 => 64,
        65..=128 => 128,
        129..=256 => 256,
        257..=512 => 512,
        _ => panic!("number is too big")
    }
}

impl Display for Number {
    // TODO colored output
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // write first index line
        write!(f, "   ")?;
        let mut first_index_line = self.buffer.len() as i32 - 1;
        while first_index_line >= 0 {
            write!(f, "{}", format!("{:<2}{:>7}  ", first_index_line, first_index_line - 7))?;
            first_index_line -= 8;
        }
        writeln!(f, "")?;

        // write bits
        let sign_char = match self.buffer.last() {
            Some(true) if self.is_signed => '-',
            Some(false) if self.is_signed => '+',
            _ => 'u'
        };
        write!(f, "{}  ", sign_char)?;
        for _ in self.buffer.len()..self.buffer.len() {
            write!(f, "{}", 0)?;
        }
        let mut count = 0;
        for b in self.buffer.iter().take(self.buffer.len()).rev() {
            count += 1;
            match b {
                true => write!(f, "{}", 1),
                false => write!(f, "{}", 0)
            }?;
            if count % 4 == 0 {
                write!(f, " ")?;
            }
            if count % 8 == 0 {
                write!(f, " ")?;
            }
        }
        writeln!(f, "")?;

        // write second index line
        write!(f, "c {}", if self.carry { '1' } else { '0' })?;
        let mut second_index_line = self.buffer.len() as i32 - 4;
        while second_index_line >= 0 {
            //  60 59
            write!(f, "{}", format!("{:>4} {:<4}  ", second_index_line, second_index_line - 1))?;
            second_index_line -= 8;
        }
        Ok(())
    }
}

#[test]
fn add_test_add_by_one() {
    let mut n = Number::from("0", 10);
    assert_eq!("00000000", n.to_string(2));
    n.add_number(1);
    assert_eq!("00000001", n.to_string(2));
    n.add_number(1);
    assert_eq!("00000010", n.to_string(2));
    n.add_number(1);
    assert_eq!("00000011", n.to_string(2));
    n.add_number(1);
    assert_eq!("00000100", n.to_string(2));

    let mut n = Number::from("0", 10);
    n.add_number(u32::MAX);
    assert_eq!("11111111111111111111111111111111", n.to_string(2));
    // assert_eq!("11111111", n.to_string(2));

    let mut n = Number::from("1", 10);
    n.add_number(u32::MAX);
    assert_eq!("100000000000000000000000000000000", n.to_string(2));
    // assert_eq!("00000000", n.to_string(2));
}

#[test]
fn add_test_add_three() {
    let mut n = Number::from("0", 10);
    assert_eq!("00000000", n.to_string(2));
    n.add_number(3);
    assert_eq!("00000011", n.to_string(2));
}

#[test]
fn add_bools_test() {
    let mut n = Number::from("0", 10);
    n.add_bools(&[true]);
    assert_eq!("00000001", n.to_string(2));
    n.add_bools(&[true]);
    assert_eq!("00000010", n.to_string(2));
    n.add_bools(&[true]);
    assert_eq!("00000011", n.to_string(2));
    n.add_bools(&[true]);
    assert_eq!("00000100", n.to_string(2));

    let mut n = Number::from("15", 10);
    n.add_bools(&[true]);
    assert_eq!("00010000", n.to_string(2));

    n.add_bools(&[true, false]);
    assert_eq!("00010001", n.to_string(2));

    let mut n = Number::from("0", 10);
    n.add_bools(&[true, true]);
    assert_eq!("00000011", n.to_string(2));

    let mut n = Number::from("1", 10);
    n.add_bools(&[false, true]);
    assert_eq!("00000011", n.to_string(2));
}

#[test]
fn add_vecs_test_add_all_false() {
    let mut n = Number::from("0", 10);
    assert_eq!("00000000", n.to_string(2));
    n.add_number(0);
    assert_eq!("00000000", n.to_string(2));

    let mut n = Number::from("0", 10);
    n.add_number(1);
    assert_eq!("00000001", n.to_string(2));

    let mut n = Number::from("0", 10);
    n.add_number(10);
    assert_eq!("00001010", n.to_string(2));

    let mut n = Number::from("1", 10);
    n.add_number(10);
    assert_eq!("00001011", n.to_string(2));
}

#[test]
fn mul_number_test() {
    let mut n = Number::from("0", 10);
    n.mul_number(142);
    assert_eq!("00000000", n.to_string(2));

    let mut n = Number::from("10", 10);
    n.mul_number(1);
    assert_eq!("00001010", n.to_string(2));

    let mut n = Number::from("1", 10);
    assert_eq!("00000001", n.to_string(2));
    n.mul_number(10);
    assert_eq!("00001010", n.to_string(2));

    let mut n = Number::from("1", 10);
    n.mul_number(16);
    assert_eq!("00010000", n.to_string(2));
}

#[test]
fn from_str_r10() {
    let n = Number::from("0", 10);
    assert_eq!("00000000", n.to_string(2));

    let n = Number::from("1", 10);
    assert_eq!("00000001", n.to_string(2));

    let n = Number::from("2", 10);
    assert_eq!("00000010", n.to_string(2));

    let n = Number::from("9", 10);
    assert_eq!("00001001", n.to_string(2));

    let n = Number::from("10", 10);
    assert_eq!("00001010", n.to_string(2));

    let n = Number::from("15", 10);
    assert_eq!("00001111", n.to_string(2));

    let n = Number::from("16", 10);
    assert_eq!("00010000", n.to_string(2));

    let n = Number::from(&*u8::MAX.to_string(), 10);
    assert_eq!("11111111", n.to_string(2));

    let n = Number::from(&*u32::MAX.to_string(), 10);
    assert_eq!("11111111111111111111111111111111", n.to_string(2));

    let n = Number::from("2147483648", 10);
    assert_eq!("10000000000000000000000000000000", n.to_string(2));
}

#[test]
fn from_str_r2() {
    let n = Number::from("10000000000000000000000000000000", 2);
    assert_eq!("10000000000000000000000000000000", n.to_string(2));

    let n = Number::from("0", 2);
    assert_eq!("00000000", n.to_string(2));

    let n = Number::from("1", 2);
    assert_eq!("00000001", n.to_string(2));

    let n = Number::from("10", 2);
    assert_eq!("00000010", n.to_string(2));

    let n = Number::from("1010", 2);
    assert_eq!("00001010", n.to_string(2));

    let n = Number::from("1111", 2);
    assert_eq!("00001111", n.to_string(2));

    let n = Number::from("11111", 2);
    assert_eq!("00011111", n.to_string(2));

    let n = Number::from("1111111111111111111111111111111111111111", 2);
    assert_eq!("0000000000000000000000001111111111111111111111111111111111111111", n.to_string(2));
}

#[test]
fn from_str_r8() {
    let n = Number::from("1111", 8);
    assert_eq!("0000001001001001", n.to_string(2));
}

#[test]
fn from_str_r16() {
    let n = Number::from("F", 16);
    assert_eq!("00001111", n.to_string(2));

    let n = Number::from("10", 16);
    assert_eq!("00010000", n.to_string(2));

    let n = Number::from("1F", 16);
    assert_eq!("00011111", n.to_string(2));

    let n = Number::from("AF", 16);
    assert_eq!("10101111", n.to_string(2));
}

#[test]
fn number_get_bits() {
    let n = Number::from("F", 16);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!([true, true, true, true, false, false, false, false], bits);

    let n = Number::from("1E", 16);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(3), BitsIndex::IndexedBit(0)));
    assert_eq!([false, true, true, true], bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(4), BitsIndex::IndexedBit(1)));
    assert_eq!([true; 4], bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!([false, true, true, true, true, false, false, false], bits);
}

#[test]
fn number_set_bits() {
    let mut n = Number::from("0", 16);
    n.set_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), &[true, true]);
    assert_eq!(vec![true, true, false, false, false, false, false, false], n.buffer);
}
