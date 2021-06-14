use super::names::Names;
use crate::{
    ast::{self, Enum, Field, FieldType, SchemaAst, Top, TopId},
    diagnostics::{DatamodelError, Diagnostics},
};
use std::collections::HashMap;

pub(crate) struct ParserDatabase<'ast> {
    schema: &'ast SchemaAst,
    names: Names<'ast>,
    _type_aliases: HashMap<TopId, FullyResolvedType>,
}

impl<'ast> ParserDatabase<'ast> {
    pub(super) fn new(schema: &'ast SchemaAst, diagnostics: &mut Diagnostics) -> ParserDatabase<'ast> {
        let names = Names::new(schema, diagnostics);

        let type_aliases = resolve_aliases(schema, &names, diagnostics);
        // Todo: check types and populate constraints model per model.

        ParserDatabase {
            schema,
            names,
            _type_aliases: type_aliases,
        }
    }

    pub(super) fn ast(&self) -> &'ast SchemaAst {
        self.schema
    }

    pub(super) fn get_enum(&self, name: &str, schema: &'ast ast::SchemaAst) -> Option<&'ast ast::Enum> {
        self.names.tops.get(name).and_then(|top_id| schema[*top_id].as_enum())
    }

    pub(super) fn iter_enums(&self) -> impl Iterator<Item = (TopId, &'ast Enum)> + '_ {
        self.names
            .tops
            .values()
            .filter_map(move |topid| self.schema[*topid].as_enum().map(|enm| (*topid, enm)))
    }
}

/// The type of a field, with type aliases erased.
#[derive(Debug, Clone, Copy)]
enum FullyResolvedType {
    // Model(TopId),
    Enum(TopId),
    Scalar,
    Unsupported,
    Unknown,
}

const BUILT_IN_SCALARS: &[&str] = &[
    "Int", "BigInt", "Float", "Boolean", "String", "DateTime", "Json", "Bytes", "Decimal",
];

/// Fully resolve type aliases to non-aliased types. Substituting the resolved
/// type from the returned map for the alias will correctly eliminate aliases.
fn resolve_aliases(
    schema: &SchemaAst,
    names: &Names<'_>,
    diagnostics: &mut Diagnostics,
) -> HashMap<TopId, FullyResolvedType> {
    let mut aliases = HashMap::new();
    // The references to other aliases followed from the "root" alias. This
    // is used to render error messages in case a recursive definition is
    // detected.
    let mut traversed_type_aliases: Vec<&str> = Vec::new();

    for (alias_id, type_alias) in schema
        .iter_tops()
        .filter_map(|(id, top)| top.as_type_alias().map(|alias| (id, alias)))
    {
        traversed_type_aliases.clear();
        aliases.insert(
            alias_id,
            resolve_alias(
                (alias_id, type_alias),
                schema,
                names,
                &mut traversed_type_aliases,
                diagnostics,
            ),
        );
    }

    aliases
}

fn resolve_alias<'a>(
    (root_alias_id, root_type_alias): (TopId, &Field),
    schema: &'a SchemaAst,
    names: &Names<'_>,
    traversed_type_aliases: &mut Vec<&'a str>,
    diagnostics: &mut Diagnostics,
) -> FullyResolvedType {
    match &root_type_alias.field_type {
        FieldType::Supported(type_name) => {
            if BUILT_IN_SCALARS.contains(&type_name.name.as_str()) {
                return FullyResolvedType::Scalar;
            }

            match names.tops.get(type_name.name.as_str()).map(|id| (id, &schema[*id])) {
                Some((referenced_alias_id, Top::Type(referenced_alias))) => {
                    if *referenced_alias_id == root_alias_id
                        || traversed_type_aliases.contains(&referenced_alias.name.name.as_str())
                    {
                        // Recursive type.
                        diagnostics.push_error(DatamodelError::new_validation_error(
                            &format!(
                                "Recursive type definitions are not allowed. Recursive path was: {} -> {}.",
                                traversed_type_aliases.join(" -> "),
                                root_type_alias.name.name
                            ),
                            root_type_alias.span,
                        ));
                        return FullyResolvedType::Unknown;
                    }

                    traversed_type_aliases.push(&referenced_alias.name.name);

                    resolve_alias(
                        (root_alias_id, root_type_alias),
                        schema,
                        names,
                        traversed_type_aliases,
                        diagnostics,
                    )
                }
                Some((_, Top::Model(_))) => {
                    diagnostics.push_error(DatamodelError::new_validation_error(
                        "Only scalar types can be used for defining custom types.",
                        root_type_alias.field_type.span(),
                    ));
                    FullyResolvedType::Unknown
                }
                Some((id, Top::Enum(_))) => FullyResolvedType::Enum(*id),
                Some((_, Top::Generator(_))) | Some((_, Top::Source(_))) => unreachable!(),
                None => {
                    diagnostics.push_error(DatamodelError::new_type_not_found_error(
                        &type_name.name,
                        root_type_alias.field_type.span(),
                    ));
                    FullyResolvedType::Unknown
                }
            }
        }
        FieldType::Unsupported(_, _) => FullyResolvedType::Unsupported,
    }
}
