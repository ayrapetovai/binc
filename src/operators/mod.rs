use crate::syntax::OperandSource;
use crate::number::{Number, BitsIndexRange, BitsIndex};
use log::trace;

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
        DirectSource(mut other_number) => {
            other_number.signed_extend_to(buffer.max_size());
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    let bits = other_number.get_bits(BitsIndexRange(BitsIndex::HighestBit, BitsIndex::LowestBit));
                    trace!("operator_assign: get bits: {:?}", bits);
                    buffer.set_bits(target_range, bits);
                },
                NamedAccessSource(_) => {}
            }
        },
        RangeSource(source_range) => {
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range).to_owned();
                    buffer.set_bits(target_range, &bits[..]);
                },
                NamedAccessSource(_) => {}
            }
        }
        NamedAccessSource(_) => {}
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
                NamedAccessSource(_) => {}
            }
        },
        RangeSource(source_range) => {
            match left {
                DirectSource(_) => panic!("The left side of an expression cannot be represented by an immediate value"), // will never be here
                RangeSource(target_range) => {
                    let bits = buffer.get_bits(source_range).to_owned();
                    buffer.set_bits(target_range, &bits[..]);
                },
                NamedAccessSource(_) => {}
            }
        }
        NamedAccessSource(_) => {}
    }
    Historical
}

pub fn operator_unsigned_shift_left(buffer: &mut Number, left: OperandSource, right: OperandSource) -> HandlerResult {
    todo!()
}