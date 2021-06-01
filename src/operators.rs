use crate::syntax::{LeftOperandSource, RightOperandSource};
use crate::number::{Number, BitsIndexRange, BitsIndex, NumberType};
use colored::{Colorize, Color};
use log::trace;

pub type OperationResult = Result<(HandlerResult, Option<String>), String>;

#[derive(PartialEq)]
pub enum HandlerResult {
    Historical,
    Nonhistorical
}
use HandlerResult::Nonhistorical;
use HandlerResult::Historical;

pub type Operator = fn(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult;

// TODO colored output
pub fn operator_show_help(_: &mut Number, _: LeftOperandSource, _: RightOperandSource) -> OperationResult {
    let mut buffer = String::with_capacity(400);

    buffer.push_str(format!("{}", "X operator Y:".color(Color::BrightGreen)).as_str());
    buffer.push_str(" >> << >>> <<< + - * / pow sqrt > < s<< s>> s>>> s<<< ^ & | <> == = count\r\n");

    buffer.push_str(format!("{}", "operator X:".color(Color::BrightGreen)).as_str());
    buffer.push_str(" ! random shuffle reverse ~\r\n");

    buffer.push_str(format!("{}", "X and Y can be:".color(Color::BrightGreen)).as_str());
    buffer.push_str(" [] [i] [:] [i:] [:j] [i:j] c; e f\r\n");

    buffer.push_str(format!("{}", "only Y can be:".color(Color::BrightGreen)).as_str());
    buffer.push_str(" 1 3.14 -0; -inf +inf NaN eps; 'a'\r\n");

    buffer.push_str(format!("{}", "commands:".color(Color::BrightGreen)).as_str());
    buffer.push_str(" intX floatX fixedX printf signed unsigned history undo redo about ?");

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

pub fn operator_unsigned_shift_left(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    Ok((Historical, Some("not implemented".to_owned())))
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