use std::fmt::{Display, Formatter};
use log::trace;
use std::mem::size_of;
use colored::{Colorize, Color};

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

type BufferType = u128;

#[derive(Debug)]
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
    pub fn from(number_literal: &str, radix: u32) -> Result<Self, String> {
        trace!("Number::from: parsing literal '{}', radix {}", number_literal, radix);
        let is_negative = number_literal.starts_with("-");
        // TODO floating, fixed
        let mut it = number_literal.chars();
        if is_negative {
            it.next();
        }
        let mut buffer = 0u128;
        while let Some(c) = it.next() {
            let n = match c {
                '0'..='9' => c as u32 - '0' as u32,
                'a'..='z' => c as u32 - 'a' as u32 + 10,
                'A'..='Z' => c as u32 - 'A' as u32 + 10,
                _ => return Err(format!("letter {} cannot represent a digit", c).to_owned())
            };
            if n < radix {
                buffer *= radix as u128;
                buffer += n as u128;
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

    pub fn add_bits(&mut self, additive: u128) {
        self.carry = false; // TODO
        self.buffer = self.buffer.wrapping_add(additive);
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    pub fn range_add_bits(&mut self, range: BitsIndexRange, additive: u128) {
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        self.carry = false; // TODO
        let sum = ((self.buffer >> low_order_bit_index) & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)).wrapping_add(additive);
        self.buffer = self.buffer & !mask_from_bit_to_bit(high_order_bit_index, low_order_bit_index);
        self.buffer = self.buffer | ((sum & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)) << low_order_bit_index);
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    pub fn range_subtract_bits(&mut self, range: BitsIndexRange, subtractive: u128) {
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        self.carry = false; // TODO
        let sub = ((self.buffer >> low_order_bit_index) & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)).wrapping_sub(subtractive);
        self.buffer = self.buffer & !(mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1) << low_order_bit_index);
        self.buffer = self.buffer | ((sub & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)) << low_order_bit_index);
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    pub fn range_multiply_bits(&mut self, range: BitsIndexRange, multiplayer: u128) {
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        self.carry = false; // TODO
        let mul = ((self.buffer >> low_order_bit_index) & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)).wrapping_mul(multiplayer);
        self.buffer = self.buffer & !(mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1) << low_order_bit_index);
        self.buffer = self.buffer | ((mul & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)) << low_order_bit_index);
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    pub fn range_div_bits(&mut self, range: BitsIndexRange, multiplayer: u128) {
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        self.carry = false; // TODO
        let mul = ((self.buffer >> low_order_bit_index) & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)).wrapping_div(multiplayer);
        self.buffer = self.buffer & !(mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1) << low_order_bit_index);
        self.buffer = self.buffer | ((mul & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)) << low_order_bit_index);
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    pub fn range_mod_bits(&mut self, range: BitsIndexRange, multiplayer: u128) {
        let high_order_bit_index = self.resolve_bit_index(range.0);
        let low_order_bit_index = self.resolve_bit_index(range.1);
        self.carry = false; // TODO
        let mul = ((self.buffer >> low_order_bit_index) & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)).wrapping_rem(multiplayer);
        self.buffer = self.buffer & !(mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1) << low_order_bit_index);
        self.buffer = self.buffer | ((mul & mask_n_ones_from_right(high_order_bit_index - low_order_bit_index + 1)) << low_order_bit_index);
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }

    pub fn assign_value(&mut self, other: &Number) {
        // self.effective_bits must not be changed
        self.buffer = other.buffer;
        self.number_type = other.number_type;
        self.is_signed = other.is_signed;
        self.carry = false;
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
        self.buffer = (self.buffer & !mask_from_bit_to_bit(high_index, low_index)) | ((source_bits & mask_n_ones_from_right((high_index - low_index + 1))) << low_index)
    }

    pub fn max_size(&self) -> usize {
        self.effective_bits as usize
    }

    pub fn signed_extend_to(&mut self, new_max_size: usize) {
        // if this is signed and negative
        if self.is_signed && self.buffer & mask_nth_bit(self.effective_bits - 1) != 0 {
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
    pub fn flip_all(&mut self) {
        self.buffer = !self.buffer;
        self.buffer = self.buffer & mask_n_ones_from_right(self.effective_bits);
    }
    pub fn to_string(&self, radix: u32) -> String {
        if self.is_negative() {
            let value = !(self.buffer - 1) & mask_n_ones_from_right(self.effective_bits - 1);
            match radix {
                2 => format!("-0b{:b}", value).to_owned(),
                8 => format!("-0o{:o}", value).to_owned(),
                10 => format!("-0d{}", value).to_owned(),
                16 => format!("-0x{:x}", value).to_owned(),
                _ => panic!("cannot translate to number of radix {}", radix)
            }
        } else {
            match radix {
                2 => format!("0b{:b}", self.buffer).to_owned(),
                8 => format!("0o{:o}", self.buffer).to_owned(),
                10 => format!("0d{}", self.buffer).to_owned(),
                16 => format!("0x{:x}", self.buffer).to_owned(),
                _ => panic!("cannot translate to number of radix {}", radix)
            }
        }
    }
    pub fn signed_shift_left(&mut self, range: BitsIndexRange, count: usize) {
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        let bits_to_shift = (self.buffer & (mask_n_ones_from_right(high_index - low_index + 1) << low_index)) >> low_index;
        self.buffer = self.buffer & !(mask_n_ones_from_right(high_index - low_index + 1) << low_index);
        self.buffer = self.buffer | (((bits_to_shift << count) & mask_n_ones_from_right(high_index - low_index + 1))<< low_index);
        // cyclic: self.buffer = self.buffer | ((bits_to_shift << count | bits_to_shift >> (self.effective_bits - count)) << low_index);
    }
    /// pools sign bit (leftmost)
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
        let high_index = self.resolve_bit_index(range.0);
        let low_index = self.resolve_bit_index(range.1);
        let bits_to_shift = (self.buffer & (mask_n_ones_from_right(high_index - low_index + 1) << low_index)) >> low_index;
        self.buffer = self.buffer & !(mask_n_ones_from_right(high_index - low_index + 1) << low_index);
        self.buffer = self.buffer | (((bits_to_shift >> count) & mask_n_ones_from_right(high_index - low_index + 1)) << low_index);
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
    mask_n_ones_from_right((high_inclusive + 1 - low)) << low
}

const NUMBER_OF_CONVEX_DELTAHEDRON: i32 = 8;
const NUMBER_OF_BLACK_PRESIDENTS_OF_US: i32 = 1;
const NUMBER_OF_DEADLY_SINS: i32 = 7;

/// #The Book Of Dark Wizardry Arts: Hacker's Delight 2nd Edition
/// ##Chapter 3. Power-of-2 Boundaries
/// 3.1 Rounding Up/Down to a Multiple of a Known Power of 2
/// 3.2 Rounding Up/Down to the Next Power of 2
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

fn next_power_of_two(n: usize) -> usize {
    (0x80_00_00_00u32 >> ((n as i32 - 1).leading_zeros() as i32 - 1)) as usize
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
        write!(f, "c {}", if self.carry { '1' } else { '0' })?;
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
fn next_power_of_two_test() {
    // assert_eq!(1, next_power_of_two(0));
    assert_eq!(1, next_power_of_two(1));
    assert_eq!(2, next_power_of_two(2));
    assert_eq!(4, next_power_of_two(3));
    assert_eq!(4, next_power_of_two(4));
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
fn from_negative_str_r10() {
    let n = Number::from("-0", 10).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from("-1", 10).unwrap();
    assert_eq!(0b11111111, n.to_usize());
}

#[test]
fn from_str_r10() {
    let n = Number::from("0", 10).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from("1", 10).unwrap();
    assert_eq!(1, n.to_usize());

    let n = Number::from("2", 10).unwrap();
    assert_eq!(0b10, n.to_usize());

    let n = Number::from("9", 10).unwrap();
    assert_eq!(0b1001, n.to_usize());

    let n = Number::from("10", 10).unwrap();
    assert_eq!(0b1010, n.to_usize());

    let n = Number::from("15", 10).unwrap();
    assert_eq!(0b1111, n.to_usize());

    let n = Number::from("16", 10).unwrap();
    assert_eq!(0b10000, n.to_usize());

    let n = Number::from(&*u8::MAX.to_string(), 10).unwrap();
    assert_eq!(0b11111111, n.to_usize());

    let n = Number::from(&*u32::MAX.to_string(), 10).unwrap();
    assert_eq!("0b11111111111111111111111111111111", n.to_string(2));

    let n = Number::from("2147483648", 10).unwrap();
    assert_eq!("0b10000000000000000000000000000000", n.to_string(2));
}

#[test]
fn from_str_r2() {
    let n = Number::from("10000000000000000000000000000000", 2).unwrap();
    assert_eq!("0b10000000000000000000000000000000", n.to_string(2));

    let n = Number::from("0", 2).unwrap();
    assert_eq!(0, n.to_usize());

    let n = Number::from("1", 2).unwrap();
    assert_eq!(0b1, n.to_usize());

    let n = Number::from("10", 2).unwrap();
    assert_eq!(0b10, n.to_usize());

    let n = Number::from("1010", 2).unwrap();
    assert_eq!(0b1010, n.to_usize());

    let n = Number::from("1111", 2).unwrap();
    assert_eq!(0b1111, n.to_usize());

    let n = Number::from("11111", 2).unwrap();
    assert_eq!(0b11111, n.to_usize());

    let n = Number::from("1111111111111111111111111111111111111111", 2).unwrap();
    assert_eq!("0b1111111111111111111111111111111111111111", n.to_string(2));
}

#[test]
fn from_str_r8() {
    let n = Number::from("1111", 8).unwrap();
    assert_eq!(0b1001001001, n.to_usize());
}

#[test]
fn from_str_r16() {
    let n = Number::from("F", 16).unwrap();
    assert_eq!(0b1111, n.to_usize());

    let n = Number::from("10", 16).unwrap();
    assert_eq!(0b10000, n.to_usize());

    let n = Number::from("1F", 16).unwrap();
    assert_eq!(0b11111, n.to_usize());

    let n = Number::from("AF", 16).unwrap();
    assert_eq!(0b10101111, n.to_usize());
}

#[test]
fn number_get_bits() {
    let n = Number::from("F", 16).unwrap();
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!(0xf, bits);

    let n = Number::from("1E", 16).unwrap();
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(3), BitsIndex::IndexedBit(0)));
    assert_eq!(0b1110, bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::IndexedBit(4), BitsIndex::IndexedBit(1)));
    assert_eq!(0xf, bits);
    let bits = n.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
    assert_eq!(0b00011110, bits);
}

#[test]
fn number_set_bits() {
    let mut n = Number::from("0", 16).unwrap();
    n.set_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 0b11);
    assert_eq!(0b11, n.to_usize());
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
    assert_eq!(0b11111110, n.to_usize());

    let mut n = Number::from(&u8::MAX.to_string(), 10).unwrap();
    n.flip_all();
    assert_eq!(0, n.to_usize());
}

#[test]
fn number_range_add_bits() {
    let mut n = Number::from("0", 10).unwrap();
    assert_eq!(0, n.to_usize());
    n.range_add_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit), 1);
    assert_eq!(1, n.to_usize());

    let mut n = Number::from("ffff00", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::IndexedBit(23), BitsIndex::IndexedBit(8)), 1);
    assert_eq!(0x00_0000_00, n.to_usize());

    let mut n = Number::from("fffe00", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::IndexedBit(23), BitsIndex::IndexedBit(8)), 1);
    assert_eq!(0xffff00, n.to_usize());

    let mut n = Number::from("0", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::IndexedBit(0)), 1);
    assert_eq!(1, n.to_usize());

    let mut n = Number::from("0", 16).unwrap();
    n.range_add_bits(BitsIndexRange(BitsIndex::IndexedBit(7), BitsIndex::IndexedBit(7)), 1);
    assert_eq!(0x80, n.to_usize());
}

#[test]
fn number_signed_shift_left() {
    let mut n = Number::from("1", 10).unwrap();
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