use crate::syntax::OperandSource;
use crate::number::Number;

pub enum HandlerResult {
    Error(String),
    Historical,
    Nonhistorical
}

use HandlerResult::Error;
use HandlerResult::Nonhistorical;
use HandlerResult::Historical;

use OperandSource::DirectSource;
use OperandSource::RangeSource;
use OperandSource::NamedAccessSource;

pub type Operator = fn(buffer: &mut Number, left: OperandSource, right: OperandSource) -> HandlerResult;

pub fn operator_assign(buffer: &mut Number, left: OperandSource, right: OperandSource) -> HandlerResult {
    match right {
        DirectSource(other_number) => {
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    buffer.assign_value(&other_number);
                    // let bits = other_number.get_bits((usize::MAX, 0));
                    // buffer.set_bits(target_range, bits);
                },
                NamedAccessSource(_) => {}
            }
        },
        RangeSource(source_range) => {
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    // let bits = buffer.get_bits(source_range);
                    // buffer.set_bits(target_range, bits);
                },
                NamedAccessSource(_) => {}
            }
        }
        NamedAccessSource(_) => {}
    }
    Historical
}