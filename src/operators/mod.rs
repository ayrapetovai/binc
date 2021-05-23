use crate::syntax::OperandSource;
use crate::number::{Number, BitsIndexRange, BitsIndex};
use log::trace;

pub type OperationResult = Result<(HandlerResult, Option<String>), String>;

#[derive(PartialEq)]
pub enum HandlerResult {
    Historical,
    Nonhistorical
}

use HandlerResult::Nonhistorical;
use HandlerResult::Historical;

use OperandSource::DirectSource;
use OperandSource::RangeSource;
use OperandSource::NamedAccessSource;
use OperandSource::Empty;

pub type Operator = fn(buffer: &mut Number, left: OperandSource, right: OperandSource) -> OperationResult;

// TODO colored output
pub fn operator_show_help(_: &mut Number, _: OperandSource, _: OperandSource) -> OperationResult {
    Ok((
        Nonhistorical,
        Some(
"X operator Y: >> << >>> <<< + - * / pow sqrt > < s<< s>> s>>> s<<< ^ & | <> == = count
operator X: ! random shuffle reverse ~
X and Y can be: [] [i] [:] [i:] [:j] [i:j] c; e f
only Y can be: 1 3.14 -0; -inf +inf NaN eps
commands: intX floatX fixedX printf signed unsigned history undo redo about ?".to_owned()
        )
    ))
}

pub fn operator_assign(buffer: &mut Number, left: OperandSource, right: OperandSource) -> OperationResult {
    match right {
        DirectSource(mut other_number) => {
            other_number.signed_extend_to(buffer.max_size());
            match left {
                DirectSource(_) => return Err("The left side of an expression cannot be represented by an immediate value".to_owned()), // will never be here
                RangeSource(target_range) => {
                    let bits = other_number.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_assign: get bits: {:?}", bits);
                    buffer.set_bits(target_range, bits);
                },
                NamedAccessSource(_) => {},
                Empty => return Err("no second operand!".to_owned())
            }
        },
        RangeSource(source_range) => {
            match left {
                DirectSource(_) => return Err("The left side of an expression cannot be represented by an immediate value".to_owned()), // will never be here
                RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range).to_owned();
                    buffer.set_bits(target_range, &bits[..]);
                },
                NamedAccessSource(_) => {},
                Empty => return Err("no second operand!".to_owned())
            }
        }
        NamedAccessSource(_) => {},
        Empty => return Err("no first operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_sum(buffer: &mut Number, left: OperandSource, right: OperandSource) -> OperationResult {
    match right {
        DirectSource(mut second_sum_operand) => {
            second_sum_operand.signed_extend_to(buffer.max_size());
            match left {
                DirectSource(_) => return Err("The left side of an expression cannot be represented by an immediate value".to_owned()), // will never be here
                RangeSource(target_range) => {
                    let bits = second_sum_operand.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_sum: get bits: {:?}", bits);
                    buffer.add_bits(bits);
                },
                NamedAccessSource(_) => {},
                Empty => return Err("no second operand!".to_owned())
            }
        }
        RangeSource(source_range) => {
            match left {
                DirectSource(_) => return Err("The left side of an expression cannot be represented by an immediate value".to_owned()), // will never be here
                RangeSource(target_range) => {
                    todo!()
                },
                NamedAccessSource(_) => {},
                Empty => return Err("no second operand!".to_owned())
            }
        }
        NamedAccessSource(_) => {},
        Empty => return Err("no first operand!".to_owned())
    }
    Ok((Historical, None))
}

pub fn operator_unsigned_shift_left(buffer: &mut Number, left: OperandSource, right: OperandSource) -> OperationResult {
    Ok((Historical, Some("not implemented".to_owned())))
}