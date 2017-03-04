use parser::{Pattern, TypeDefinition, Expression};

use super::{OperationsResult, type_definition, expression};
use super::scope::ScopeStack;

pub fn into_operations(
    pattern: Pattern,
    type_def: TypeDefinition,
    expr: Option<Expression>,
    scope: &mut ScopeStack,
) -> OperationsResult {
    let type_id = type_definition::resolve_type_id(type_def, scope)?;

    let name = match pattern {
        Pattern::Identifier(name, ..) => name,
    };

    expr.map_or(Ok(Vec::new()), |e| {
        let mem = scope.declare(name, type_id);
        expression::into_operations(e, type_id, Some(mem), scope)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::{Identifier, Span};
    use operations::item_type::ItemType;

    #[test]
    fn declaration_only() {
        // When only doing a declaration, no operations should be generated
        // since there is no expression to actually evaluate
        let mut scope = ScopeStack::new();
        scope.declare_type(Identifier::from("u8"), ItemType::Primitive(1));

        let ops = into_operations(
            Pattern::Identifier(Identifier::from("foo"), Span {start: 0, end: 0}),
            TypeDefinition::Name {name: Identifier::from("u8"), span: Span {start: 0, end: 0}},
            None,
            &mut scope
        ).unwrap();
        assert_eq!(ops.len(), 0);
    }
}