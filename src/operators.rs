use crate::syntax::{LeftOperandSource, RightOperandSource};
use crate::number::{Number, BitsIndexRange, BitsIndex, NumberType};
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
    Ok((
        Nonhistorical,
        Some(
"X operator Y: >> << >>> <<< + - * / pow sqrt > < s<< s>> s>>> s<<< ^ & | <> == = count
operator X: ! random shuffle reverse ~
X and Y can be: [] [i] [:] [i:] [:j] [i:j] c; e f
only Y can be: 1 3.14 -0; -inf +inf NaN eps; 'a'
commands: intX floatX fixedX printf signed unsigned history undo redo about ?".to_owned()
        )
    ))
}

pub fn operator_assign(buffer: &mut Number, left: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(mut other_number) => {
            other_number.signed_extend_to(buffer.max_size());
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = other_number.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_assign: get bits: {:?}", bits);
                    buffer.set_bits(target_range, bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {}
            }
        },
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range).to_owned();
                    buffer.set_bits(target_range, &bits[..]);
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
                    buffer.add_bits(bits);
                },
                LeftOperandSource::NamedAccessSource(_) => {},
            }
        }
        RightOperandSource::RangeSource(source_range) => {
            match left {
                LeftOperandSource::RangeSource(target_range) => {
                    todo!()
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

pub fn operator_bits_width(buffer: &mut Number, _: LeftOperandSource, right: RightOperandSource) -> OperationResult {
    match right {
        RightOperandSource::DirectSource(number) => {
            buffer.convert(NumberType::Integer, true, number.to_usize());
            Ok((Historical, None))
        }
        _ => return Err("Bit width is necessary argument".to_owned())
    }
}