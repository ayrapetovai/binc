use crate::syntax::OperandSource;
use crate::number::{Number, BitsIndexRange, BitsIndex};
use log::trace;

// TODO remove Message, introduce Historical(String) and Nonhistorical(String), Result<HandlerResult, String>
pub enum HandlerResult {
    Message(String),
    Historical,
    Nonhistorical
}

use HandlerResult::Message;
use HandlerResult::Nonhistorical;
use HandlerResult::Historical;

use OperandSource::DirectSource;
use OperandSource::RangeSource;
use OperandSource::NamedAccessSource;

pub type Operator = fn(buffer: &mut Number, left: OperandSource, right: OperandSource) -> HandlerResult;

pub fn operator_show_help(_: &mut Number, _: OperandSource, _: OperandSource) -> HandlerResult {
    Message(
"X operator (Y|X): >> << >>> <<< + - * / pow sqrt > < s<< s>> s>>> s<<< ^ & | <> == = count
operator X: ! random shuffle reverse ~
X, Y: [] [i] [:] [i:] [:j] [i:j] c; e f
Y: 1 3.14 -0; -inf +inf NaN eps
commands: intX floatX fixedX printf signed unsigned history undo redo about ?".to_owned()
    )
}

pub fn operator_assign(buffer: &mut Number, left: OperandSource, right: OperandSource) -> HandlerResult {
    match right {
        DirectSource(mut other_number) => {
            other_number.signed_extend_to(buffer.max_size());
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    let bits = other_number.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_assign: get bits: {:?}", bits);
                    buffer.set_bits(target_range, bits);
                },
                NamedAccessSource(_) => {},
                Empty => panic!("no second operand!")
            }
        },
        RangeSource(source_range) => {
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range).to_owned();
                    buffer.set_bits(target_range, &bits[..]);
                },
                NamedAccessSource(_) => {},
                Empty => panic!("no second operand!")
            }
        }
        NamedAccessSource(_) => {},
        Empty => panic!("no first operand!")
    }
    Historical
}

pub fn operator_sum(buffer: &mut Number, left: OperandSource, right: OperandSource) -> HandlerResult {
    match right {
        DirectSource(mut other_number) => {
            other_number.signed_extend_to(buffer.max_size());
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    let bits = other_number.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_sum: get bits: {:?}", bits);
                    buffer.add_bools(bits);
                },
                NamedAccessSource(_) => {},
                Empty => panic!("no second operand!")
            }
        },
        RangeSource(source_range) => {
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    todo!()
                },
                NamedAccessSource(_) => {},
                Empty => panic!("no second operand!")
            }
        }
        NamedAccessSource(_) => {},
        Empty => panic!("no first operand!")
    }
    Historical
}

pub fn operator_unsigned_shift_left(buffer: &mut Number, left: OperandSource, right: OperandSource) -> HandlerResult {
    Message("not implemented".to_owned())
}