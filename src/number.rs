// binc
// Copyright (C) 2022  Artem Ayrapetov
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt::{Display, Formatter};
use log::trace;
use std::mem::size_of;
use colored::{Colorize, Color};
use rand::prelude::SliceRandom;

#[derive(Debug, Copy, Clone)]
pub enum BitsIndex {
    HighestBit,
    LowestBit,
    IndexedBit(usize)
}

#[derive(Debug, Copy, Clone)]
pub struct BitsIndexRange(pub BitsIndex, pub BitsIndex);

#[derive(Debug, Copy, Clone)]
pub enum NumberType {
    Integer,
    Float,
    Fixed,
}

// TODO user struct BitUint from crate 'num-bigint'
type BufferType = u128;

// TODO implement std::str::FromStr
#[derive(Debug, Clone)]
pub struct Number {
    buffer: BufferType, // only right "effective_bits" are used
    effective_bits: usize,
    number_type: NumberType,
    is_signed: bool,
    carry: bool,
}

impl Number {
    pub fn new(number_type: NumberType, is_signed: bool, size: usize) -> Result<Self, String> {
        match next_power_of_two_rounded_up(size) {
            Ok(size) =>
                Ok(Self {
                    buffer: 0u128,
                    effective_bits: size,
                    number_type,
                    is_signed,
                    carry: false,
                }),
            Err(message) => Err(message)
        }
    }
    pub fn from_char(c: char) -> Result<Self, String> {
        Ok(
            Self {
                buffer: c.into(),
                effective_bits: c.len_utf8() * 8,
                number_type: NumberType::Integer,
                is_signed: false,
                carry: false,
            }
        )
    }
    pub fn from_str(number_literal: &str, radix: u32) -> Result<Self, String> {
        let radix= radix as u128;
        trace!("Number::from: parsing literal '{}', radix {}", number_literal, radix);
        let is_negative = number_literal.starts_with("-");
        // TODO floating, fixed
        let mut it = number_literal.chars();
        if is_negative {
            it.next();
        }
        // TODO use u128::from_str_radix() or BigUint::from_str_radix(...)
        let mut buffer = 0u128;
        while let Some(c) = it.next() {
            let n = match c {
                '0'..='9' => c as u32 - '0' as u32,
                'a'..='z' => c as u32 - 'a' as u32 + 10,
                'A'..='Z' => c as u32 - 'A' as u32 + 10,
                _ => return Err(format!("letter {} cannot represent a digit", c).to_owned())
            } as u128;
            if n < radix {
                buffer *= radix;
                buffer += n;
            } else {
                return Err(format!("Letter '{}' cannot be used for number notation in base {}", c, radix).to_owned());
            }
        }
        trace!("Number::from: parsed buffer 0b{:b}", buffer);

        let buffer_length = size_of::<BufferType>() * 8;
        let length_in_bits = match next_power_of_two_rounded_up(buffer_length - buffer.leading_zeros() as usize) {
            Ok(s) => s,
            Err(m) => return Err(m)
        };
        if is_negative && buffer != 0 {
            trace!("Number::from: negate {}", buffer);
            buffer = !buffer;
            buffer += 1;
        }
        Ok(
            Self {
                buffer: buffer & mask_n_ones_from_right(length_in_bits),
                effective_bits: length_in_bits,
                number_type: if number_literal.contains(".") { NumberType::Float } else { NumberType::Integer },
                is_signed: is_negative,
                carry: false,
            }
        )
    }

    fn with_range_do_arithmetics(&mut self, range: BitsIndexRange, arithmetica: Box<dyn Fn(u128) -> u128>) {
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        self.carry = false; // TODO
        let mul = arithmetica(self.get_bits(range));
        self.buffer = self.buffer & !mask_from_bit_to_bit(high_order_bit_index, low_order_bit_index);
        self.buffer = self.buffer | ((mul & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)) << low_order_bit_index);
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    pub fn range_add_bits(&mut self, range: BitsIndexRange, additive: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a.wrapping_add(additive)));
    }

    pub fn range_subtract_bits(&mut self, range: BitsIndexRange, subtractive: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a.wrapping_sub(subtractive)));
    }

    pub fn range_multiply_bits(&mut self, range: BitsIndexRange, multiplayer: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a.wrapping_mul(multiplayer)));
    }

    pub fn range_div_bits(&mut self, range: BitsIndexRange, divisor: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a.wrapping_div(divisor)));
    }

    pub fn range_pow_bits(&mut self, range: BitsIndexRange, magnitude: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a.wrapping_pow(magnitude as u32)));
    }

    /// Find the number X, which powered to N is A.
    /// x^n = a;
    /// ln(x^n) = ln(a);
    /// n*ln(x) = ln(a);
    /// ln(x) = ln(a)/n;
    /// exp(ln(x)) = exp(ln(a)/n);
    /// x = exp(ln(a)/n);
    pub fn range_root_bits(&mut self, range: BitsIndexRange, power: u128) {
        match self.number_type {
            NumberType::Integer => self.with_range_do_arithmetics(range, Box::new(move |a: u128| ((a as f64).ln() / power as f64).exp() as u128)),
            _ => todo!("root operation for float and fixed is not implemented")
        }
    }

    pub fn range_mod_bits(&mut self, range: BitsIndexRange, divisor: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a.wrapping_rem(divisor)));
    }
    pub fn range_xor_bits(&mut self, range: BitsIndexRange, second_operand: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a ^ second_operand));
    }
    pub fn range_and_bits(&mut self, range: BitsIndexRange, second_operand: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a & second_operand));
    }
    pub fn range_or_bits(&mut self, range: BitsIndexRange, second_operand: u128) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a | second_operand));
    }
    pub fn signed_shift_left(&mut self, range: BitsIndexRange, count: usize) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a << count))
    }
    /// pools sign bit (leftmost to right)
    pub fn signed_shift_right(&mut self, range: BitsIndexRange, count: usize) {
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        let bits_to_shift = (self.buffer & (mask_n_ones_from_right(high_index - low_index + 1) << low_index)) >> low_index;
        self.buffer = self.buffer & !(mask_n_ones_from_right(high_index - low_index + 1) << low_index);
        let left_bits = if bits_to_shift & mask_nth_bit(high_index - low_index) != 0 {
            mask_n_ones_from_right(count) << (high_index - low_index + 1 - count)
        } else {
            0
        };
        self.buffer = self.buffer | ((left_bits | (bits_to_shift >> count) & mask_n_ones_from_right(high_index - low_index + 1)) << low_index);
    }
    /// prepends with zeroes (leftmost)
    pub fn unsigned_shift_right(&mut self, range: BitsIndexRange, count: usize) {
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a >> count))
    }

    pub fn unsigned_cyclic_shift_left(&mut self, range: BitsIndexRange, count: usize) {
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| (a << count) | (a >> ((high_index - low_index + 1) - count))))
    }
    pub fn unsigned_cyclic_shift_right(&mut self, range: BitsIndexRange, count: usize) {
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| (a >> count) | (a << ((high_index - low_index + 1) - count))))
    }

    pub fn range_count_bits(&mut self, range: BitsIndexRange, one_or_zero: u8) -> usize {
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        let bits = self.get_bits(range);
        return match one_or_zero {
            0 => (bits | mask_from_bit_to_bit(127, high_index + 1 - low_index)).count_zeros() as usize,
            1 => (bits & mask_n_ones_from_right(high_index + 1 - low_index)).count_ones() as usize,
            _ => usize::MAX
        }
    }

    pub fn range_reverse_bits(&mut self, range: BitsIndexRange) {
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| a.reverse_bits() >> (size_of::<BufferType>()*8 - (high_index + 1 - low_index))))
    }

    pub fn range_shuffle_bits(&mut self, range: BitsIndexRange) {
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        let size = high_index + 1 - low_index;
        self.with_range_do_arithmetics(range, Box::new(move |a: u128| {
            let mut values: Vec<bool> = (0..size).map(|n| (a >> n) & 1 == 1).collect();

            let mut rng = rand::thread_rng();
            let size = values.len();
            values.partial_shuffle(&mut rng, size);

            let mut result = BufferType::default();
            for (i, &b) in values.iter().enumerate() {
                if b {
                    result = result | (1u128 << i);
                }
            }
            result
        }
        ))
    }

    pub fn negate(&mut self) {
        self.is_signed = true;
        let num = self.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit)) as i128;
        self.buffer = (-num) as u128;
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    fn resolve_bit_index(&self, bi: BitsIndex) -> usize {
        match bi {
            BitsIndex::IndexedBit(i) => i,
            BitsIndex::HighestBit => self.effective_bits - 1,
            BitsIndex::LowestBit => 0
        }
    }

    pub fn get_bits(&self, range: BitsIndexRange) -> u128 {
        trace!("Number::get_bits: range {:?}", range);
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        (self.buffer & mask_from_bit_to_bit(high_order_bit_index, low_order_bit_index)) >> low_order_bit_index
    }

    pub fn set_bits(&mut self, range: BitsIndexRange, source_bits: u128) {
        trace!("Number::set_bits: range {:?}, source {:b}", range, source_bits);
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        self.buffer = (self.buffer & !mask_from_bit_to_bit(high_index, low_index)) | ((source_bits & mask_n_ones_from_right(high_index - low_index + 1)) << low_index)
    }

    pub fn max_size(&self) -> usize {
        self.effective_bits
    }

    pub fn signed_extend_to(&mut self, new_max_size: usize) {
        // if this is signed and negative
        if self.is_negative() {
            self.buffer = self.buffer | (mask_n_ones_from_right(self.buffer.leading_zeros() as usize) << (size_of::<BufferType>()*8 - self.buffer.leading_zeros() as usize));
        }
        self.effective_bits = new_max_size;
    }
    pub fn convert(&mut self, number_type: NumberType, signed: bool, size: usize) {
        trace!("Number::convert {:?}, signed {}, size {}", number_type, signed, size);
        self.number_type = number_type;
        self.is_signed = signed;
        self.effective_bits = size;
        self.buffer = self.buffer & mask_n_ones_from_right(size);
    }
    pub fn to_usize(&self) -> usize {
        self.buffer as usize
    }
    pub fn to_u128(&self) -> u128 {
        self.buffer
    }
    pub fn to_string_as_char(&self) -> String {
        match char::from_u32(self.to_u128() as u32) {
            Some(c) => if !c.is_control() { format!("'{}'", c) } else { " ? ".to_owned() },
            None => " ? ".to_owned()
        }
    }
    pub fn to_string_prefixed(&self, radix: u32) -> String {
        self.to_string(radix, true)
    }
    pub fn to_string(&self, radix: u32, with_prefix: bool) -> String {
        let value = if self.is_negative() {
            !(self.buffer - 1) & mask_n_ones_from_right(self.effective_bits - 1)
        } else {
            self.buffer
        };
        let mut formatted = match radix {
            2 => format!("{:b}", value).to_owned(),
            8 => format!("{:o}", value).to_owned(),
            10 => format!("{}", value).to_owned(),
            16 => format!("{:x}", value).to_owned(),
            _ => panic!("cannot translate to number of radix {}", radix)
        };
        if with_prefix {
            match radix {
                2 => formatted.insert_str(0, "0b"),
                8 => formatted.insert_str(0, "0o"),
                10 => formatted.insert_str(0, "0d"),
                16 => formatted.insert_str(0, "0x"),
                _ => panic!("cannot choose prefix for radix {}", radix)
            }
        }
        if self.is_negative() {
            formatted.insert(0, '-');
        }
        formatted
    }
    pub fn is_negative(&self) -> bool {
        self.is_signed && self.buffer & mask_nth_bit(self.effective_bits - 1) != 0
    }
    pub fn number_of_digits_in_radix(&self, radix: u32) -> usize {
        (self.effective_bits as f32 * 2f32.log2() / (radix as f32).log2() + 1f32) as usize
    }
    pub fn signed(&self) -> bool {
        self.is_signed
    }
}

fn mask_nth_bit(n: usize) -> u128 {
    match n {
        127 => 1u128 << 127,
        _ => (2 as i128).pow(n as u32) as u128
    }
}

fn mask_n_ones_from_right(n: usize) -> u128 {
    match n {
        127 => u128::MAX >> 1,
        128 => u128::MAX,
        _ => (!-(2 as i128).pow(n as u32)) as u128
    }
}

fn mask_from_bit_to_bit(high_inclusive: usize, low: usize) -> u128 {
    mask_n_ones_from_right(high_inclusive + 1 - low) << low
}

const NUMBER_OF_CONVEX_DELTAHEDRON: i32 = 8;
const NUMBER_OF_BLACK_PRESIDENTS_OF_US: i32 = 1;
const NUMBER_OF_DEADLY_SINS: i32 = 7;

/// # The Book Of Dark Wizardry Arts: Hacker's Delight 2nd Edition
/// ## Chapter 3. Power-of-2 Boundaries
/// ### 3.1 Rounding Up/Down to a Multiple of a Known Power of 2
/// ### 3.2 Rounding Up/Down to the Next Power of 2
fn next_power_of_two_rounded_up(n: usize) -> Result<usize, String> {
    if 0 == n {
        Ok(8)
    } else if n < 513 {
        let abra = (n as i32 - NUMBER_OF_BLACK_PRESIDENTS_OF_US).leading_zeros() as i32;
        let cadabra = (0x80_00_00_00u32 >> (abra - NUMBER_OF_BLACK_PRESIDENTS_OF_US)) as i32;
        Ok(((cadabra + NUMBER_OF_DEADLY_SINS) & -NUMBER_OF_CONVEX_DELTAHEDRON) as usize)
    } else {
        Err(format!("error, length too big; length cannot be zero, given {}", n).to_owned())
    }
}

impl Display for Number {
    // TODO colored output
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // write first index line
        write!(f, "   ")?;
        let mut first_index_line = self.effective_bits as i32 - 1;
        while first_index_line >= 0 {
            write!(f, "{}", format!("{:<3}{:>6}  ", first_index_line, first_index_line - 7))?;
            first_index_line -= 8;
        }
        writeln!(f, "")?;

        // write bits
        let sign_char = if self.is_signed {
            if self.is_negative() { '-' } else { '+' }
        } else {
            'u'
        };
        write!(f, "{}  ", sign_char)?;
        let mut count = self.effective_bits as i32 - 1;
        let mut buffer = String::with_capacity(self.effective_bits + (self.effective_bits / 8) + (self.effective_bits / 4));
        while 0 <= count {
            buffer.push_str(format!("{}", (self.buffer & (1 << count)) >> count).as_str());

            if count % 4 == 0 {
                buffer.push_str(" ");
            }
            if count % 8 == 0 {
                buffer.push_str(" ");
            }
            count -= 1;
        }
        writeln!(f, "{}", buffer.color(Color::Red))?;

        // write second index line
        write!(f, "  {}", if self.carry { '1' } else { '0' })?;
        let mut second_index_line = self.effective_bits as i32 - 4;
        while second_index_line >= 0 {
            //  60 59
            write!(f, "{}", format!("{:>4} {:<4}  ", second_index_line, second_index_line - 1))?;
            second_index_line -= 8;
        }
        Ok(())
    }
}

#[test]
fn mask_n_ones_from_right_test() {
    assert_eq!(0, mask_n_ones_from_right(0));
    assert_eq!(1, mask_n_ones_from_right(1));
    assert_eq!(0b11, mask_n_ones_from_right(2));
    assert_eq!(0b111, mask_n_ones_from_right(3));
    assert_eq!(0b1111, mask_n_ones_from_right(4));
    assert_eq!(0b11111, mask_n_ones_from_right(5));
    assert_eq!(0xff_ff_ff_ff, mask_n_ones_from_right(32));
    assert_eq!(0x7f_ff_ff_ff_ff_ff_ff_ff, mask_n_ones_from_right(63));
    assert_eq!(0xff_ff_ff_ff_ff_ff_ff_ff, mask_n_ones_from_right(64));
    assert_eq!(0x3f_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff, mask_n_ones_from_right(126));
    assert_eq!(0x7f_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff, mask_n_ones_from_right(127));
    assert_eq!(0xff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff_ff, mask_n_ones_from_right(128));
}

#[test]
fn mask_from_bit_to_bit_test() {
    assert_eq!(0b1, mask_from_bit_to_bit(0, 0));
    assert_eq!(0b11, mask_from_bit_to_bit(1, 0));
    assert_eq!(0b111, mask_from_bit_to_bit(2, 0));
    assert_eq!(0b1111, mask_from_bit_to_bit(3, 0));
    assert_eq!(0b10, mask_from_bit_to_bit(1, 1));
    assert_eq!(0b110, mask_from_bit_to_bit(2, 1));
    assert_eq!(0b1110, mask_from_bit_to_bit(3, 1));
}

#[test]
fn from_char() {
    let n = Number::from_char('a').unwrap();
    assert_eq!(0b01100001u128, n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit)));

    let n = Number::from_char('λ').unwrap();
    assert_eq!(0b0000001110111011, n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit)));

    let n = Number::from_char('心').unwrap();
    assert_eq!(0b0101111111000011u128, n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit)));
}

#[test]
fn from_negative_str_r10() {
    let n = Number::from_str("-0", 10).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from_str("-1", 10).unwrap();
    assert_eq!(0b11111111, n.to_usize());
}

#[test]
fn from_str_r10() {
    let n = Number::from_str("0", 10).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from_str("1", 10).unwrap();
    assert_eq!(1, n.to_usize());

    let n = Number::from_str("2", 10).unwrap();
    assert_eq!(0b10, n.to_usize());

    let n = Number::from_str("9", 10).unwrap();
    assert_eq!(0b1001, n.to_usize());

    let n = Number::from_str("10", 10).unwrap();
    assert_eq!(0b1010, n.to_usize());

    let n = Number::from_str("15", 10).unwrap();
    assert_eq!(0b1111, n.to_usize());

    let n = Number::from_str("16", 10).unwrap();
    assert_eq!(0b10000, n.to_usize());

    let n = Number::from_str(&*u8::MAX.to_string(), 10).unwrap();
    assert_eq!(0b11111111, n.to_usize());

    let n = Number::from_str(&*u32::MAX.to_string(), 10).unwrap();
    assert_eq!("0b11111111111111111111111111111111", n.to_string_prefixed(2));

    let n = Number::from_str("2147483648", 10).unwrap();
    assert_eq!("0b10000000000000000000000000000000", n.to_string_prefixed(2));
}

#[test]
fn from_str_r2() {
    let n = Number::from_str("10000000000000000000000000000000", 2).unwrap();
    assert_eq!("0b10000000000000000000000000000000", n.to_string_prefixed(2));

    let n = Number::from_str("0", 2).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from_str("1", 2).unwrap();
    assert_eq!(0b1, n.to_usize());

    let n = Number::from_str("10", 2).unwrap();
    assert_eq!(0b10, n.to_usize());

    let n = Number::from_str("1010", 2).unwrap();
    assert_eq!(0b1010, n.to_usize());

    let n = Number::from_str("1111", 2).unwrap();
    assert_eq!(0b1111, n.to_usize());

    let n = Number::from_str("11111", 2).unwrap();
    assert_eq!(0b11111, n.to_usize());

    let n = Number::from_str("1111111111111111111111111111111111111111", 2).unwrap();
    assert_eq!("0b1111111111111111111111111111111111111111", n.to_string_prefixed(2));
}

#[test]
fn from_str_r8() {
    let n = Number::from_str("1111", 8).unwrap();
    assert_eq!(0b1001001001, n.to_usize());
}

#[test]
fn from_str_r16() {
    let n = Number::from_str("F", 16).unwrap();
    assert_eq!(0b1111, n.to_usize());

    let n = Number::from_str("10", 16).unwrap();
    assert_eq!(0b10000, n.to_usize());

    let n = Number::from_str("1F", 16).unwrap();
    assert_eq!(0b11111, n.to_usize());

    let n = Number::from_str("AF", 16).unwrap();
    assert_eq!(0b10101111, n.to_usize());
}

#[test]
fn number_get_bits() {
    let n = Number::from_str("F", 16).unwrap();
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!(0xf, bits);

    let n = Number::from_str("1E", 16).unwrap();
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(3), BitsIndex::IndexedBit(0)));
    assert_eq!(0b1110, bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(4), BitsIndex::IndexedBit(1)));
    assert_eq!(0xf, bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!(0b00011110, bits);
}

#[test]
fn number_set_bits() {
    let mut n = Number::from_str("0", 16).unwrap();
    n.set_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 0b11);
    assert_eq!(0b11, n.to_usize());
}

#[test]
fn number_to_usize() {
    let n = Number::from_str("0", 10).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from_str("1", 10).unwrap();
    assert_eq!(1, n.to_usize());

    let n = Number::from_str(&usize::MAX.to_string(), 10).unwrap();
    assert_eq!(usize::MAX, n.to_usize());
}

#[test]
fn number_range_add_bits() {
    let mut n = Number::from_str("0", 10).unwrap();
    assert_eq!(0, n.to_usize());
    n.range_add_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 1);
    assert_eq!(1, n.to_usize());

    let mut n = Number::from_str("ffff00", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::IndexedBit(23), BitsIndex::IndexedBit(8)), 1);
    assert_eq!(0x00_0000_00, n.to_usize());

    let mut n = Number::from_str("fffe00", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::IndexedBit(23), BitsIndex::IndexedBit(8)), 1);
    assert_eq!(0xffff00, n.to_usize());

    let mut n = Number::from_str("0", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::IndexedBit(0)), 1);
    assert_eq!(1, n.to_usize());

    let mut n = Number::from_str("0", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::IndexedBit(7), BitsIndex::IndexedBit(7)), 1);
    assert_eq!(0x80, n.to_usize());
}

#[test]
fn number_signed_shift_left() {
    let mut n = Number::from_str("1", 10).unwrap();
    assert_eq!(0b1, n.to_usize());
    n.signed_shift_left(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 1);
    assert_eq!(0b10, n.to_usize());
    n.signed_shift_left(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 1);
    assert_eq!(0b100, n.to_usize());

    let mut n = Number::new(NumberType::Integer, false, 32).unwrap();
    n.set_bits(BitsIndexRange(BitsIndex::HighestBit,BitsIndex::HighestBit), 1);
    n.signed_shift_left(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 1);
    assert_eq!(0, n.to_usize());

    let mut n = Number::new(NumberType::Integer, false, 32).unwrap();
    n.set_bits(BitsIndexRange(BitsIndex::IndexedBit(23),BitsIndex::IndexedBit(12)), 0xfff);
    n.signed_shift_left(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 1);
    assert_eq!(0x01_ff_e0_00, n.to_usize());

    n.signed_shift_left(BitsIndexRange(BitsIndex::IndexedBit(24), BitsIndex::IndexedBit(12)), 1);
    assert_eq!(0x01_ff_c0_00, n.to_usize());

    n.signed_shift_left(BitsIndexRange(BitsIndex::IndexedBit(24), BitsIndex::IndexedBit(12)), 1);
    assert_eq!(0x01_ff_80_00, n.to_usize());
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