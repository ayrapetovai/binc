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
    buffer: Vec<bool>, // FIXME make it Vec<u8>
    number_type: NumberType,
    is_signed: bool,
    carry: bool,
}

impl Number {
    pub fn new(number_type: NumberType, is_signed: bool, max_size: usize) -> Result<Self, String> {
        match next_power_of_two_rounded_up(max_size) {
            Ok(size) =>
                Ok(Self {
                    buffer: vec![false; size],
                    number_type,
                    is_signed,
                    carry: false,
                }),
            Err(message) => Err(message)
        }
    }
    pub fn from(number_literal: &str, radix: u32) -> Result<Self, String> {
        trace!("Number::from: parsing literal '{}', radix {}", number_literal, radix);
        let is_negative = number_literal.starts_with("-");
        // TODO floating point, + and - notations
        let mut new_number = Self {
            buffer: vec![false; 1],
            number_type: if number_literal.contains(".") { NumberType::Float } else { NumberType::Integer },
            is_signed: is_negative,
            carry: false,
        };
        let mut it = number_literal.chars();
        if is_negative {
            it.next();
        }
        while let Some(c) = it.next() {
            let n = match c {
                '0'..='9' => c as u32 - '0' as u32,
                'a'..='z' => c as u32 - 'a' as u32 + 10,
                'A'..='Z' => c as u32 - 'A' as u32 + 10,
                _ => panic!("letter {} cannot represent a digit", c)
            };
            if n < radix {
                new_number.mul_number(radix);
                new_number.add_number(n, usize::MAX);
            } else {
                return Err(format!("Letter '{}' cannot be used for number notation in base {}", c, radix).to_owned());
            }
        }
        trace!("Number::from: parsed buffer {:?}", new_number.buffer);
        let max_size = match next_power_of_two_rounded_up(new_number.buffer.len()) {
            Ok(s) => s,
            Err(m) => return Err(m)
        };
        let mut buf = vec![false; max_size];
        for i in 0..new_number.buffer.len() {
            buf[i] = new_number.buffer[i];
        }
        new_number.buffer = buf;
        trace!("Number::from: reajusted buffer is {:?}", new_number.buffer);

        if is_negative {
            trace!("Number::from: negate");
            new_number.flip_all();
            new_number.add_number(1, new_number.buffer.len());
        }
        new_number.carry = false;
        Ok(new_number)
    }

    fn add_number(&mut self, additive: u32, max_size: usize) {
        trace!("add_number: additive is {}, number.buffer.len is {}", additive, self.buffer.len());
        let additive_length = 32usize - additive.leading_zeros() as usize;
        let mut i = 0;
        let mut additive_mask = 0x1u32;
        self.carry = false;
        while !(!self.carry && i >= additive_length) && i < max_size {
            if i >= self.buffer.len() {
                self.buffer.push(false);
            }
            let current_mul_bit = additive & additive_mask != 0;
            trace!("add_number: adding bit {} of additive to ith bit of number {}", current_mul_bit,  self.buffer[i]);
            let new_carry = ((current_mul_bit ^ self.buffer[i]) & self.carry) | (current_mul_bit & self.buffer[i]);
            trace!("add_number: new_carry {}", new_carry);
            self.buffer[i] = (current_mul_bit ^ self.buffer[i]) ^ self.carry;
            trace!("add_number: new ith bit {}", self.buffer[i]);
            self.carry = new_carry;
            i += 1;
            additive_mask <<= 1;
        }
    }

    fn mul_number(&mut self, multiplier: u32) {
        let additive = self.buffer.clone();
        for _ in 1..multiplier {
            self.add_bools(&additive[..]);
        }
    }

    fn add_bools(&mut self, additive: &[bool]) {
        self.carry = false;
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
        self.carry = carry;
    }

    pub fn add_bits(&mut self, additive: &[bool]) {
        self.carry = false;
        let mut i = 0;
        while !(!self.carry && i >= additive.len()) && i < self.buffer.len() {
            let additive_ith = if i >= additive.len() { false } else { additive[i] };
            let new_carry = ((additive_ith ^ self.buffer[i]) && self.carry) || (additive_ith && self.buffer[i]);
            self.buffer[i] = (additive_ith ^ self.buffer[i]) ^ self.carry;
            self.carry = new_carry;
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

    fn resolve_bit_index(&self, bi: BitsIndex) -> usize {
        match bi {
            BitsIndex::IndexedBit(i) => i,
            BitsIndex::HighestBit => &self.buffer.len()  - 1,
            BitsIndex::LowestBit => 0
        }
    }

    pub fn get_bits(&self, range: BitsIndexRange) -> &[bool] {
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        &self.buffer[low_order_bit_index..=high_order_bit_index]
    }

    pub fn set_bits(&mut self, range: BitsIndexRange, source_bits: &[bool]) {
        trace!("Number::set_bits: {:?}, source {:?}", range, source_bits);
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        let mut source_index = 0;
        let mut target_index = low_index;
        while target_index <= high_index {
            if source_index < source_bits.len() && target_index < self.buffer.len() {
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

    // TODO check weather the number is signed inside method, do not let caller to decide
    pub fn signed_extend_to(&mut self, new_max_size: usize) {
        if self.buffer.len() < new_max_size {
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
    }
    pub fn convert(&mut self, number_type: NumberType, signed: bool, size: usize) {
        self.number_type = number_type;
        self.is_signed = signed;
        let mut buf = vec![false; next_power_of_two_rounded_up(size).unwrap()];
        for i in 0..buf.len() {
            if i < self.buffer.len() {
                buf[i] = self.buffer[i];
            }
        }
        self.buffer = buf;
    }
    pub fn to_usize(&self) -> usize {
        let mut result = 0;
        let mut mask = 1;
        for bit in &self.buffer {
            result = result | if *bit { mask } else { 0 };
            mask <<= 1;
        }
        result
    }
    pub fn flip_all(&mut self) {
        for b in &mut self.buffer {
            *b = !*b;
        }
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

const NUMBER_OF_CONVEX_DELTAHEDRON: i32 = 8;
const NUMBER_OF_BLACK_PRESIDENTS_OF_US: i32 = 1;
const NUMBER_OF_DEADLY_SINS: i32 = 7;

/// #The Book Of Dark Wizardry Arts: Hacker's Delight 2nd Edition
/// ##Chapter 3. Power-of-2 Boundaries
/// 3.1 Rounding Up/Down to a Multiple of a Known Power of 2
/// 3.2 Rounding Up/Down to the Next Power of 2
fn next_power_of_two_rounded_up(length: usize) -> Result<usize, String> {
    if 0 < length && length < 513 {
        let abra = (length as i32 - NUMBER_OF_BLACK_PRESIDENTS_OF_US).leading_zeros() as i32;
        let cadabra = (0x80_00_00_00u32 >> (abra - NUMBER_OF_BLACK_PRESIDENTS_OF_US)) as i32;
        Ok(((cadabra + NUMBER_OF_DEADLY_SINS) & -NUMBER_OF_CONVEX_DELTAHEDRON) as usize)
    } else {
        Err(format!("error, length too big; length cannot be zero, given {}", length).to_owned())
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
    let mut n = Number::from("0", 10).unwrap();
    assert_eq!("00000000", n.to_string(2));
    n.add_number(1, usize::MAX);
    assert_eq!("00000001", n.to_string(2));
    n.add_number(1, usize::MAX);
    assert_eq!("00000010", n.to_string(2));
    n.add_number(1, usize::MAX);
    assert_eq!("00000011", n.to_string(2));
    n.add_number(1, usize::MAX);
    assert_eq!("00000100", n.to_string(2));

    let mut n = Number::from("0", 10).unwrap();
    n.add_number(u32::MAX, usize::MAX);
    assert_eq!("11111111111111111111111111111111", n.to_string(2));
    // assert_eq!("11111111", n.to_string(2));

    let mut n = Number::from("1", 10).unwrap();
    n.add_number(u32::MAX, usize::MAX);
    assert_eq!("100000000000000000000000000000000", n.to_string(2));
    // assert_eq!("00000000", n.to_string(2));

    let mut n = Number::from("1", 10).unwrap();
    n.add_number(u32::MAX, n.max_size());
    assert_eq!("00000000", n.to_string(2));
}

#[test]
fn add_test_add_three() {
    let mut n = Number::from("0", 10).unwrap();
    assert_eq!("00000000", n.to_string(2));
    n.add_number(3, n.max_size());
    assert_eq!("00000011", n.to_string(2));
}

#[test]
fn add_bools_test() {
    let mut n = Number::from("0", 10).unwrap();
    n.add_bools(&[true]);
    assert_eq!("00000001", n.to_string(2));
    n.add_bools(&[true]);
    assert_eq!("00000010", n.to_string(2));
    n.add_bools(&[true]);
    assert_eq!("00000011", n.to_string(2));
    n.add_bools(&[true]);
    assert_eq!("00000100", n.to_string(2));

    let mut n = Number::from("15", 10).unwrap();
    n.add_bools(&[true]);
    assert_eq!("00010000", n.to_string(2));

    n.add_bools(&[true, false]);
    assert_eq!("00010001", n.to_string(2));

    let mut n = Number::from("0", 10).unwrap();
    n.add_bools(&[true, true]);
    assert_eq!("00000011", n.to_string(2));

    let mut n = Number::from("1", 10).unwrap();
    n.add_bools(&[false, true]);
    assert_eq!("00000011", n.to_string(2));
}

#[test]
fn add_vecs_test_add_all_false() {
    let mut n = Number::from("0", 10).unwrap();
    assert_eq!("00000000", n.to_string(2));
    n.add_number(0, usize::MAX);
    assert_eq!("00000000", n.to_string(2));

    let mut n = Number::from("0", 10).unwrap();
    n.add_number(1, usize::MAX);
    assert_eq!("00000001", n.to_string(2));

    let mut n = Number::from("0", 10).unwrap();
    n.add_number(10, usize::MAX);
    assert_eq!("00001010", n.to_string(2));

    let mut n = Number::from("1", 10).unwrap();
    n.add_number(10, usize::MAX);
    assert_eq!("00001011", n.to_string(2));
}

#[test]
fn mul_number_test() {
    let mut n = Number::from("0", 10).unwrap();
    n.mul_number(142);
    assert_eq!("00000000", n.to_string(2));

    let mut n = Number::from("10", 10).unwrap();
    n.mul_number(1);
    assert_eq!("00001010", n.to_string(2));

    let mut n = Number::from("1", 10).unwrap();
    assert_eq!("00000001", n.to_string(2));
    n.mul_number(10);
    assert_eq!("00001010", n.to_string(2));

    let mut n = Number::from("1", 10).unwrap();
    n.mul_number(16);
    assert_eq!("00010000", n.to_string(2));
}

#[test]
fn from_negative_str_r10() {
    let n = Number::from("-0", 10).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from("-1", 10).unwrap();
    assert_eq!("11111111", n.to_string(2));
}

#[test]
fn from_str_r10() {
    let n = Number::from("0", 10).unwrap();
    assert_eq!("00000000", n.to_string(2));

    let n = Number::from("1", 10).unwrap();
    assert_eq!("00000001", n.to_string(2));

    let n = Number::from("2", 10).unwrap();
    assert_eq!("00000010", n.to_string(2));

    let n = Number::from("9", 10).unwrap();
    assert_eq!("00001001", n.to_string(2));

    let n = Number::from("10", 10).unwrap();
    assert_eq!("00001010", n.to_string(2));

    let n = Number::from("15", 10).unwrap();
    assert_eq!("00001111", n.to_string(2));

    let n = Number::from("16", 10).unwrap();
    assert_eq!("00010000", n.to_string(2));

    let n = Number::from(&*u8::MAX.to_string(), 10).unwrap();
    assert_eq!("11111111", n.to_string(2));

    let n = Number::from(&*u32::MAX.to_string(), 10).unwrap();
    assert_eq!("11111111111111111111111111111111", n.to_string(2));

    let n = Number::from("2147483648", 10).unwrap();
    assert_eq!("10000000000000000000000000000000", n.to_string(2));
}

#[test]
fn from_str_r2() {
    let n = Number::from("10000000000000000000000000000000", 2).unwrap();
    assert_eq!("10000000000000000000000000000000", n.to_string(2));

    let n = Number::from("0", 2).unwrap();
    assert_eq!("00000000", n.to_string(2));

    let n = Number::from("1", 2).unwrap();
    assert_eq!("00000001", n.to_string(2));

    let n = Number::from("10", 2).unwrap();
    assert_eq!("00000010", n.to_string(2));

    let n = Number::from("1010", 2).unwrap();
    assert_eq!("00001010", n.to_string(2));

    let n = Number::from("1111", 2).unwrap();
    assert_eq!("00001111", n.to_string(2));

    let n = Number::from("11111", 2).unwrap();
    assert_eq!("00011111", n.to_string(2));

    let n = Number::from("1111111111111111111111111111111111111111", 2).unwrap();
    assert_eq!("0000000000000000000000001111111111111111111111111111111111111111", n.to_string(2));
}

#[test]
fn from_str_r8() {
    let n = Number::from("1111", 8).unwrap();
    assert_eq!("0000001001001001", n.to_string(2));
}

#[test]
fn from_str_r16() {
    let n = Number::from("F", 16).unwrap();
    assert_eq!("00001111", n.to_string(2));

    let n = Number::from("10", 16).unwrap();
    assert_eq!("00010000", n.to_string(2));

    let n = Number::from("1F", 16).unwrap();
    assert_eq!("00011111", n.to_string(2));

    let n = Number::from("AF", 16).unwrap();
    assert_eq!("10101111", n.to_string(2));
}

#[test]
fn number_get_bits() {
    let n = Number::from("F", 16).unwrap();
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!([true, true, true, true, false, false, false, false], bits);

    let n = Number::from("1E", 16).unwrap();
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(3), BitsIndex::IndexedBit(0)));
    assert_eq!([false, true, true, true], bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(4), BitsIndex::IndexedBit(1)));
    assert_eq!([true; 4], bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!([false, true, true, true, true, false, false, false], bits);
}

#[test]
fn number_set_bits() {
    let mut n = Number::from("0", 16).unwrap();
    n.set_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), &[true, true]);
    assert_eq!(vec![true, true, false, false, false, false, false, false], n.buffer);
}

#[test]
fn number_to_usize() {
    let n = Number::from("0", 10).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from("1", 10).unwrap();
    assert_eq!(1, n.to_usize());

    let n = Number::from(&usize::MAX.to_string(), 10).unwrap();
    assert_eq!(usize::MAX, n.to_usize());
}

#[test]
fn number_flip_all() {
    let mut n = Number::from("1", 10).unwrap();
    n.flip_all();
    assert_eq!("11111110", n.to_string(2));

    let mut n = Number::from(&u8::MAX.to_string(), 10).unwrap();
    n.flip_all();
    assert_eq!("00000000", n.to_string(2));
}

#[test]
fn rounding_up_to_the_next_power_of_two() {
    assert_eq!(8, next_power_of_two_rounded_up(1).unwrap());
    assert_eq!(8, next_power_of_two_rounded_up(2).unwrap());
    assert_eq!(8, next_power_of_two_rounded_up(7).unwrap());
    assert_eq!(8, next_power_of_two_rounded_up(8).unwrap());
    assert_eq!(16, next_power_of_two_rounded_up(9).unwrap());
    assert_eq!(16, next_power_of_two_rounded_up(16).unwrap());
    assert_eq!(32, next_power_of_two_rounded_up(17).unwrap());
    assert_eq!(32, next_power_of_two_rounded_up(32).unwrap());
    assert_eq!(64, next_power_of_two_rounded_up(33).unwrap());
    assert_eq!(64, next_power_of_two_rounded_up(64).unwrap());
    assert_eq!(128, next_power_of_two_rounded_up(65).unwrap());
    assert_eq!(128, next_power_of_two_rounded_up(128).unwrap());
    assert_eq!(256, next_power_of_two_rounded_up(129).unwrap());
    assert_eq!(256, next_power_of_two_rounded_up(256).unwrap());
    assert_eq!(512, next_power_of_two_rounded_up(257).unwrap());
    assert_eq!(512, next_power_of_two_rounded_up(512).unwrap());
}