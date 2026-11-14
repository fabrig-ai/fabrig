use hearth_domain_render_vocabulary::errors::TypeName;

pub(super) fn type_name_article(item: &TypeName) -> &'static [u8] {
    match item {
        TypeName::Int | TypeName::Enum | TypeName::Object => b"an ",
        TypeName::Bool
        | TypeName::String
        | TypeName::List
        | TypeName::Map
        | TypeName::Path
        | TypeName::Mode
        | TypeName::Duration
        | TypeName::Secret
        | TypeName::Target
        | TypeName::File => b"a ",
    }
}
pub(super) fn type_name_name(item: &TypeName) -> &'static [u8] {
    match item {
        TypeName::Bool => b"bool",
        TypeName::Int => b"int",
        TypeName::String => b"string",
        TypeName::Enum => b"enum",
        TypeName::List => b"list",
        TypeName::Map => b"map",
        TypeName::Object => b"object",
        TypeName::Path => b"path",
        TypeName::Mode => b"mode",
        TypeName::Duration => b"duration",
        TypeName::Secret => b"secret",
        TypeName::Target => b"target",
        TypeName::File => b"file",
    }
}
