use crate::syntax::{LeftOperandSource, RightOperandSource};
use crate::number::{Number, BitsIndexRange, BitsIndex, NumberType};
use colored::{Colorize, Color};
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

// TODO colored output
pub fn operator_show_help(_: &mut Number, _: LeftOperandSource, _: RightOperandSource) -> OperationResult {
    let mut buffer = String::with_capacity(400);

    buffer.push_str(&"X operator Y:".color(Color::BrightGreen).to_string());
    buffer.push_str(" >> << + - >>> * / % > < ^ & | <<~ ~>> == = <> pow sqrt count\r\n");

    buffer.push_str(&"operator X:".color(Color::BrightGreen).to_string());
    buffer.push_str(" ! ~ random shuffle reverse\r\n");

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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
                    buffer.range_div_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range);
                    buffer.range_div_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_not(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(_) => return Err("no second operand allowed!".to_owned()),
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

pub fn operator_signed_shift_left(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_operand) => {
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
    match right {
        RightOperandSource::DirectSource(mut second_operand) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    buffer.signed_shift_right(target_range, second_operand.to_usize());
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let count = buffer.get_bits(source_range) as usize;
                    buffer.signed_shift_right(target_range, count);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::NamedAccessSource(_) => {},
        RightOperandSource::Empty => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
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
        RightOperandSource::DirectSource(mut second_operand) => {
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
        RightOperandSource::DirectSource(mut second_operand) => {
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
        RightOperandSource::DirectSource(mut second_operand) => {
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
        _ => return Err("Bit width is necessary argument".to_owned())
    }
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
        RightOperandSource::DirectSource(mut second_operand) => {
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_less(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_operand) => {
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_equals(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut second_operand) => {
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_swap(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(_) => return Err("cannot swap with rvalue!".to_owned()),
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
        RightOperandSource::Empty => return Err("no second operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_negate(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    if let LeftOperandSource::NamedAccessSource(_) = left {
        return Err("only [max:min] range is acceptable for negation.".to_owned());
    }
    if let LeftOperandSource::RangeSource(range) = left {
        if let BitsIndex::HighestBit = range.0 {
            if let BitsIndex::LowestBit = range.1 {} else {
                return Err("right bound of range can be only lowest index.".to_owned());
            }
        } else {
            return Err("left bound of range can be only highest index.".to_owned());
        }
    }
    if let RightOperandSource::Empty = right {} else {
        return Err("negation takes no second operand.".to_owned());
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

