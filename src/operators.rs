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

use crate::syntax::{LeftOperandSource, RightOperandSource};
use crate::number::{Number, BitsIndexRange, BitsIndex, NumberType};
use colored::{Colorize, Color};
use rand::random;
use log::trace;

pub type OperationResult = Result<(HandlerResult, Option<String>), String>;
#[derive(PartialEq)]
pub enum HandlerResult {
    Historical,
    Nonhistorical,
    Undo,
    Redo
}
use HandlerResult::Nonhistorical;
use HandlerResult::Historical;
use HandlerResult::Undo;
use HandlerResult::Redo;

pub type Operator = fn(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult;

pub fn operator_show_help(_: &mut Number, _: LeftOperandSource, _: RightOperandSource) -> OperationResult {
    let mut buffer = String::with_capacity(400);

    buffer.push_str(&"X operator Y:".color(Color::BrightGreen).to_string());
    buffer.push_str(" >> << + - >>> * / % > < ^ & | <<~ ~>> == = <> pow root cnt\r\n");

    buffer.push_str(&"operator X:".color(Color::BrightGreen).to_string());
    buffer.push_str(" ! ~ rnd shf rev\r\n");

    buffer.push_str(&"X and Y can be:".color(Color::BrightGreen).to_string());
    buffer.push_str(" [] [i] [:] [i:] [:j] [i:j] c; e f\r\n");

    buffer.push_str(&"only Y can be:".color(Color::BrightGreen).to_string());
    buffer.push_str(" 1 3.14 -0; -inf +inf NaN eps; 'a'\r\n");

    buffer.push_str(&"commands:".color(Color::BrightGreen).to_string());
    buffer.push_str(" intX floatX fixedX printf signed unsigned undo redo about ?");

    Ok((Nonhistorical, Some(buffer)))
}

pub fn operator_assign(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut other_number) => {
            other_number.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = other_number.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_assign: get bits: {:b}", bits);
                    buffer.set_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {}
            }
        },
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range).to_owned();
                    buffer.set_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_sum(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_sum: get bits: {:?}", bits);
                    buffer.range_add_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    trace!("operator_sum: get bits: {:?}", bits);
                    buffer.range_add_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_sub(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_subtract_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_subtract_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_mul(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_multiply_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_multiply_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_div(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    if bits != 0u128 {
                        buffer.range_div_bits(target_range, bits);
                    } else {
                        return Err("Cannot divide by 0".to_owned());
                    }
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    if bits != 0u128 {
                        buffer.range_div_bits(target_range, bits);
                    } else {
                        return Err("There are only 0 bits in given range, cannot divide by 0".to_owned())
                    }
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_mod(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_mod_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_mod_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_pow(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_operand) => {
            second_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_pow_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_pow_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.range_pow_bits(target_range, 2);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_root(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_operand) => {
            second_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_root_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_root_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.range_root_bits(target_range, 2);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_xor(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_xor_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_xor_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_and(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_and_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_and_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_or(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    buffer.range_or_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_or_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_not(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(_) => return Err("No second operand allowed!".to_owned()),
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = !buffer.get_bits(source_range);
                    buffer.set_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = !buffer.get_bits(target_range);
                    buffer.set_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_reverse(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(_) => return Err("No second operand allowed!".to_owned()),
        RightOperandSource::RangeSource(_) => return Err("No second operand allowed!".to_owned()),
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.range_reverse_bits(target_range);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_random(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(_) => return Err("No second operand allowed!".to_owned()),
        RightOperandSource::RangeSource(_) => return Err("No second operand allowed!".to_owned()),
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.set_bits(target_range, random::<u128>());
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_shuffle(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(_) => return Err("No second operand allowed!".to_owned()),
        RightOperandSource::RangeSource(_) => return Err("No second operand allowed!".to_owned()),
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.range_shuffle_bits(target_range);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_signed_shift_left(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    trace!("operator_signed_shift_left: {:?} {:?} {:?}", buffer, left, right);
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.signed_shift_left(target_range, second_operand.to_usize());
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let count = buffer.get_bits(source_range) as usize;
                    buffer.signed_shift_left(target_range, count);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.signed_shift_left(target_range, 1);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_signed_shift_right(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    trace!("operator_signed_shift_right: {:?} {:?} {:?}", buffer, left, right);
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    trace!("operator_signed_shift_right: {:?} {}", target_range, second_operand.to_usize());
                    buffer.signed_shift_right(target_range, second_operand.to_usize());
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let count = buffer.get_bits(source_range) as usize;
                    trace!("operator_signed_shift_right: {:?} {:?}, count {}", target_range, source_range, count);
                    buffer.signed_shift_right(target_range, count);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    trace!("operator_signed_shift_right: {:?} shit one", target_range);
                    buffer.signed_shift_right(target_range, 1);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_unsigned_shift_right(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.unsigned_shift_right(target_range, second_operand.to_usize());
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let count = buffer.get_bits(source_range) as usize;
                    buffer.unsigned_shift_right(target_range, count);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.unsigned_shift_right(target_range, 1);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_unsigned_cyclic_shift_right(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.unsigned_cyclic_shift_right(target_range, second_operand.to_usize());
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let count = buffer.get_bits(source_range) as usize;
                    buffer.unsigned_cyclic_shift_right(target_range, count);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.unsigned_cyclic_shift_right(target_range, 1);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_unsigned_cyclic_shift_left(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.unsigned_cyclic_shift_left(target_range, second_operand.to_usize());
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let count = buffer.get_bits(source_range) as usize;
                    buffer.unsigned_cyclic_shift_left(target_range, count);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.unsigned_cyclic_shift_left(target_range, 1);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_int_bits_width(buffer: &mut Number, _: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(number) => {
            buffer.convert(NumberType::Integer, buffer.signed(), number.to_usize());
            Ok((Historical, None))
        }
        _ => return Err("Bit width is a necessary argument".to_owned())
    }
}

pub fn operator_count(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let count = match second_operand.to_u128() {
                        0 => buffer.range_count_bits(target_range, 0),
                        1 => buffer.range_count_bits(target_range, 1),
                        _ => return Err("Counting only 1 and 0".to_owned())
                    };
                    return Ok((Nonhistorical, Some(count.to_string())));
                }
                _ => todo!("not yet implemented for float and fixed")
            }
        }
        RightOperandSource::RangeSource(_) => {
            return Err("Count operation does not read range, specify 1 or 0".to_owned());
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    return Ok((Nonhistorical, Some(buffer.range_count_bits(target_range, 1).to_string())));
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
    }
    Ok((Historical, None))
}

pub fn operator_signed(buffer: &mut Number, _: LeftOperandSource, _: RightOperandSource) -> OperationResult {
    buffer.convert(NumberType::Integer, true, buffer.max_size());
    Ok((Historical, None))
}

pub fn operator_unsigned(buffer: &mut Number, _: LeftOperandSource, _: RightOperandSource) -> OperationResult {
    buffer.convert(NumberType::Integer, false, buffer.max_size());
    Ok((Historical, None))
}

pub fn operator_greater(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits_second_op = second_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    let bits_first_op = buffer.get_bits(target_range);
                    return Ok((Nonhistorical, Some((if bits_first_op > bits_second_op { "yes" } else { "no" }).to_owned())));
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits_second_op = buffer.get_bits(source_range);
                    let bits_first_op = buffer.get_bits(target_range);
                    return Ok((Nonhistorical, Some((if bits_first_op > bits_second_op { "yes" } else { "no" }).to_owned())));
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_less(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits_second_op = second_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    let bits_first_op = buffer.get_bits(target_range);
                    return Ok((Nonhistorical, Some((if bits_first_op < bits_second_op { "yes" } else { "no" }).to_owned())));
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits_second_op = buffer.get_bits(source_range);
                    let bits_first_op = buffer.get_bits(target_range);
                    return Ok((Nonhistorical, Some((if bits_first_op < bits_second_op { "yes" } else { "no" }).to_owned())));
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_equals(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits_second_op = second_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    let bits_first_op = buffer.get_bits(target_range);
                    return Ok((Nonhistorical, Some((if bits_first_op == bits_second_op { "yes" } else { "no" }).to_owned())));
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits_second_op = buffer.get_bits(source_range);
                    let bits_first_op = buffer.get_bits(target_range);
                    return Ok((Nonhistorical, Some((if bits_first_op == bits_second_op { "yes" } else { "no" }).to_owned())));
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_swap(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(_) => return Err("Cannot swap with rvalue!".to_owned()),
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits_second_op = buffer.get_bits(source_range);
                    let bits_first_op = buffer.get_bits(target_range);
                    buffer.set_bits(target_range, bits_second_op);
                    buffer.set_bits(source_range, bits_first_op);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("No second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_negate(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    if let LeftOperandSource::NamedAccessSource(_) = left {
        return Err("Only [max:min] range is acceptable for negation.".to_owned());
    }
    if let LeftOperandSource::RangeSource(range) = left {
        if let BitsIndex::HighestBit = range.0 {
            if let BitsIndex::LowestBit = range.1 {} else {
                return Err("Right bound of range can be only lowest index.".to_owned());
            }
        } else {
            return Err("Left bound of range can be only highest index.".to_owned());
        }
    }
    if let RightOperandSource::Empty = right {} else {
        return Err("Negation takes no second operand.".to_owned());
    }
    buffer.negate();
    Ok((Historical, None))
}

pub fn operator_undo(_: &mut Number, _: LeftOperandSource, _: RightOperandSource) -> OperationResult {
    Ok((Undo, None))
}

pub fn operator_redo(_: &mut Number, _: LeftOperandSource, _: RightOperandSource) -> OperationResult {
    Ok((Redo, None))
}

