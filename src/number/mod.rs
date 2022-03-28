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
    max_size: usize,
    number_type: NumberType,
    is_signed: bool,
    carry: bool,
}

impl Number {
    pub fn new(number_type: NumberType, is_signed: bool, max_size: usize) -> Self {
        Self {
            buffer: vec![false; max_size],
            max_size,
            number_type,
            is_signed,
            carry: false,
        }
    }
    pub fn from(number_literal: &str, radix: u32) -> Self {
        // TODO floating point, + and - notations
        let mut new_number = Self {
            buffer: vec![false; 1],
            max_size: usize::MAX,
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
        new_number.max_size = (new_number.buffer.len() / 8 + if new_number.buffer.len() % 8 != 0 { 1 } else { 0 }) * 8;
        let mut buf = vec![false; new_number.max_size];
        for i in 0..new_number.buffer.len() {
            buf[i] = new_number.buffer[i];
        }
        new_number.buffer = buf;
        new_number
    }

    fn add_number(&mut self, additive: u32) {
        let additive_length = signing_bits(additive);
        let mut i = 0;
        let mut additive_mask = 0x1u32;
        let mut carry = false;
        while !(!carry && i >= additive_length) && i < self.max_size {
            if i >= self.buffer.len() {
                self.buffer.push(false);
            }
            let current_mul_bit = additive & additive_mask != 0;
            let new_carry = ((current_mul_bit ^ self.buffer[i]) & carry) | (current_mul_bit & self.buffer[i]);
            self.buffer[i] = (current_mul_bit ^ self.buffer[i]) ^ carry;
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

    fn add_bools(&mut self, additive: &[bool]) {
        let mut carry = false;
        if additive.len() > self.buffer.len() {
            self.buffer.reserve(additive.len() - self.buffer.len());
        }
        let mut i = 0;
        while !(!carry && i >= additive.len()) && i < self.max_size {
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

    pub fn assign_value(&mut self, other: &Number) {
        self.buffer = Vec::with_capacity(other.buffer.len());
        for i in 0..other.buffer.len() {
            self.buffer.push(other.buffer[i]);
        }
        self.max_size = other.max_size;
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
        self.max_size
    }

    pub fn extend_to(&mut self, new_size: usize) {
        self.max_size = new_size;
        let mut buf = vec![false; self.max_size];
        for i in 0..self.buffer.len() {
            buf[i] = self.buffer[i];
        }
        self.buffer = buf;
    }

    // fn to_string(&self, radix: u32) -> String {
    //     todo!()
    // }
}

fn signing_bits(n: u32) -> usize {
    let mut additive_length = 32usize;
    let mut additive_length_mask = 0x80_00__00_00;
    while n & additive_length_mask == 0 && additive_length_mask > 0 {
        additive_length -= 1;
        additive_length_mask >>= 1;
    }
    additive_length
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0b").unwrap();
        for _ in self.buffer.len()..self.max_size {
            write!(f, "{}", 0).unwrap()
        }
        for b in self.buffer.iter().take(self.buffer.len()).rev() {
            match b {
                true => write!(f, "{}", 1),
                false => write!(f, "{}", 0)
            }.unwrap();
        }
        Ok(())
    }
}

#[test]
fn signing_bits_test() {
    assert_eq!(0, signing_bits(0));
    assert_eq!(1, signing_bits(1));
    assert_eq!(4, signing_bits(0b1000));
    assert_eq!(32, signing_bits(u32::MAX));
}

#[test]
fn add_test_add_by_one() {
    let mut n = Number::from("0", 10);
    assert_eq!("0b00000000", n.to_string());
    n.add_number(1);
    assert_eq!("0b00000001", n.to_string());
    n.add_number(1);
    assert_eq!("0b00000010", n.to_string());
    n.add_number(1);
    assert_eq!("0b00000011", n.to_string());
    n.add_number(1);
    assert_eq!("0b00000100", n.to_string());

    let mut n = Number::from("0", 10);
    n.add_number(u32::MAX);
    // assert_eq!("0b11111111111111111111111111111111", n.to_string());
    assert_eq!("0b11111111", n.to_string());

    let mut n = Number::from("1", 10);
    n.add_number(u32::MAX);
    // assert_eq!("0b00000000000000000000000000000000", n.to_string());
    assert_eq!("0b00000000", n.to_string());
}

#[test]
fn add_test_add_three() {
    let mut n = Number::from("0", 10);
    assert_eq!("0b00000000", n.to_string());
    n.add_number(3);
    assert_eq!("0b00000011", n.to_string());
}

#[test]
fn add_bools_test() {
    let mut n = Number::from("0", 10);
    n.add_bools(&[true]);
    assert_eq!("0b00000001", n.to_string());
    assert_eq!(1, n.buffer.len());
    n.add_bools(&[true]);
    assert_eq!("0b00000010", n.to_string());
    assert_eq!(2, n.buffer.len());
    n.add_bools(&[true]);
    assert_eq!("0b00000011", n.to_string());
    assert_eq!(2, n.buffer.len());
    n.add_bools(&[true]);
    assert_eq!("0b00000100", n.to_string());
    assert_eq!(3, n.buffer.len());

    let mut n = Number::from("15", 10);
    n.add_bools(&[true]);
    assert_eq!("0b00010000", n.to_string());
    assert_eq!(5, n.buffer.len());

    n.add_bools(&[true, false]);
    assert_eq!("0b00010001", n.to_string());
    assert_eq!(5, n.buffer.len());

    let mut n = Number::from("0", 10);
    n.add_bools(&[true, true]);
    assert_eq!("0b00000011", n.to_string());
    assert_eq!(2, n.buffer.len());

    let mut n = Number::from("1", 10);
    n.add_bools(&[false, true]);
    assert_eq!("0b00000011", n.to_string());
    assert_eq!(2, n.buffer.len());
}

#[test]
fn add_vecs_test_add_all_false() {
    let mut n = Number::from("0", 10);
    assert_eq!("0b00000000", n.to_string());
    n.add_number(0);
    assert_eq!("0b00000000", n.to_string());

    let mut n = Number::from("0", 10);
    n.add_number(1);
    assert_eq!("0b00000001", n.to_string());

    let mut n = Number::from("0", 10);
    n.add_number(10);
    assert_eq!("0b00001010", n.to_string());

    let mut n = Number::from("1", 10);
    n.add_number(10);
    assert_eq!("0b00001011", n.to_string());
}

#[test]
fn mul_number_test() {
    let mut n = Number::from("0", 10);
    n.mul_number(142);
    assert_eq!("0b00000000", n.to_string());

    let mut n = Number::from("10", 10);
    n.mul_number(1);
    assert_eq!("0b00001010", n.to_string());

    let mut n = Number::from("1", 10);
    assert_eq!("0b00000001", n.to_string());
    n.mul_number(10);
    assert_eq!("0b00001010", n.to_string());

    let mut n = Number::from("1", 10);
    n.mul_number(16);
    assert_eq!("0b00010000", n.to_string());
}

#[test]
fn from_str_r10() {
    let n = Number::from("0", 10);
    assert_eq!("0b00000000", n.to_string());

    let n = Number::from("1", 10);
    assert_eq!("0b00000001", n.to_string());

    let n = Number::from("2", 10);
    assert_eq!("0b00000010", n.to_string());

    let n = Number::from("9", 10);
    assert_eq!("0b00001001", n.to_string());

    let n = Number::from("10", 10);
    assert_eq!("0b00001010", n.to_string());

    let n = Number::from("15", 10);
    assert_eq!("0b00001111", n.to_string());

    let n = Number::from("16", 10);
    assert_eq!("0b00010000", n.to_string());

    let n = Number::from(&*u8::MAX.to_string(), 10);
    assert_eq!("0b11111111", n.to_string());

    let n = Number::from(&*u32::MAX.to_string(), 10);
    assert_eq!("0b11111111111111111111111111111111", n.to_string());

    let n = Number::from("2147483648", 10);
    assert_eq!("0b10000000000000000000000000000000", n.to_string());
}

#[test]
fn from_str_r2() {
    let n = Number::from("10000000000000000000000000000000", 2);
    assert_eq!("0b10000000000000000000000000000000", n.to_string());

    let n = Number::from("0", 2);
    assert_eq!("0b00000000", n.to_string());

    let n = Number::from("1", 2);
    assert_eq!("0b00000001", n.to_string());

    let n = Number::from("10", 2);
    assert_eq!("0b00000010", n.to_string());

    let n = Number::from("1010", 2);
    assert_eq!("0b00001010", n.to_string());

    let n = Number::from("1111", 2);
    assert_eq!("0b00001111", n.to_string());

    let n = Number::from("11111", 2);
    assert_eq!("0b00011111", n.to_string());

    let n = Number::from("1111111111111111111111111111111111111111", 2);
    assert_eq!("0b1111111111111111111111111111111111111111", n.to_string());
}

#[test]
fn from_str_r8() {
    let n = Number::from("1111", 8);
    assert_eq!("0b0000001001001001", n.to_string());
}

#[test]
fn from_str_r16() {
    let n = Number::from("F", 16);
    assert_eq!("0b00001111", n.to_string());

    let n = Number::from("10", 16);
    assert_eq!("0b00010000", n.to_string());

    let n = Number::from("1F", 16);
    assert_eq!("0b00011111", n.to_string());

    let n = Number::from("AF", 16);
    assert_eq!("0b10101111", n.to_string());
}

#[test]
fn number_get_bits() {
    let n = Number::from("F", 16);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!([true; 4], bits);

    let n = Number::from("1E", 16);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(3), BitsIndex::IndexedBit(0)));
    assert_eq!([false, true, true, true], bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(4), BitsIndex::IndexedBit(1)));
    assert_eq!([true; 4], bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!([false, true, true, true, true], bits);
}

#[test]
fn number_set_bits() {
    let mut n = Number::from("0", 16);
    n.set_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), &[true, true]);
    assert_eq!(vec![true; 8], n.buffer);
}
