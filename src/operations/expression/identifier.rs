use parser::Identifier;
use memory::MemoryBlock;

use operations::{Error, Operation, OperationsResult};
use operations::item_type::ItemType;
use operations::scope::{TypeId, ScopeStack, ScopeItem, ArraySize};

use super::number::store_number;
use super::byte_literal::store_byte_literal;

pub fn store_identifier(
    scope: &mut ScopeStack,
    name: Identifier,
    target_type: TypeId,
    target: MemoryBlock,
) -> OperationsResult {
    scope.lookup(&name).first().ok_or_else(|| {
        Error::UnresolvedName(name.clone())
    }).map(|item| (**item).clone()).and_then(|item| match item {
        // There is a non-lexical lifetimes issue here which was introduced by calling store_number() below
        // The clone() above is completely unnecssary and is a hack to work around this problem
        // in the Rust compiler
        // http://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/#problem-case-2-conditional-control-flow
        ScopeItem::Constant {type_id, ref bytes} => {
            if target_type == type_id {
                Ok(Operation::increment_to_value(target, bytes))
            }
            else {
                mismatched_types(scope, target_type, type_id)
            }
        },

        ScopeItem::NumericLiteral(number) => store_number(scope, number, target_type, target),

        ScopeItem::ByteLiteral(bytes) => Err(Error::MismatchedTypes {
            expected: scope.get_type(target_type).clone(),
            found: ItemType::Array {
                item: Some(scope.primitives().u8()),
                size: Some(bytes.len()),
            },
        }),

        ScopeItem::TypedBlock {type_id, memory} => {
            if target_type == type_id {
                // Need to check this invariant or else this can lead to
                // many very subtle bugs
                debug_assert!(memory.size() == target.size());

                Ok(vec![Operation::Copy {
                    source: memory.position(),
                    target: target.position(),
                    size: memory.size(),
                }])
            }
            else {
                mismatched_types(scope, target_type, type_id)
            }
        },

        // Arrays don't have their own type_ids, so this is clearly a type error
        ScopeItem::Array {item, size, ..} => Err(Error::MismatchedTypes {
            expected: scope.get_type(target_type).clone(),
            found: ItemType::Array {
                item: Some(item),
                size: Some(size),
            },
        }),

        ScopeItem::BuiltInFunction { .. } => {
            // This is not supported for now
            unreachable!();
        },
    })
}

pub fn store_identifier_array(
    scope: &mut ScopeStack,
    name: Identifier,
    item_type: TypeId,
    size: ArraySize,
    target: MemoryBlock,
) -> OperationsResult {
    scope.lookup(&name).first().ok_or_else(|| {
        Error::UnresolvedName(name.clone())
    }).map(|item| (**item).clone()).and_then(|item| match item {
        // There is a non-lexical lifetimes issue here which was introduced by calling store_byte_literal() below
        // The clone() above is completely unnecssary and is a hack to work around this problem
        // in the Rust compiler
        // http://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/#problem-case-2-conditional-control-flow
        ScopeItem::ByteLiteral(bytes) => store_byte_literal(scope, bytes, item_type, size, target),
        ScopeItem::Array {item, size: asize, memory} if item == item_type && size == asize => {
            // Need to check this invariant or else this can lead to
            // many very subtle bugs
            debug_assert!(memory.size() == target.size());

            Ok(vec![Operation::Copy {
                source: memory.position(),
                target: target.position(),
                size: memory.size(),
            }])
        },
        item => Err(Error::MismatchedTypes {
            expected: ItemType::Array {
                item: Some(item_type),
                size: Some(size),
            },
            found: match item {
                //TODO: Update this when more numeric types are added
                ScopeItem::NumericLiteral(..) => scope.get_type(scope.primitives().u8()).clone(),
                ScopeItem::ByteLiteral(..) => unreachable!(),
                ScopeItem::Array {item, size, ..} => ItemType::Array {item: Some(item), size: Some(size)},
                ref arg => scope.get_type(arg.type_id()).clone(),
            },
        }),
    })
}

fn mismatched_types(scope: &ScopeStack, expected: TypeId, found: TypeId) -> OperationsResult {
    Err(Error::MismatchedTypes {
        expected: scope.get_type(expected).clone(),
        found: scope.get_type(found).clone(),
    })
}
