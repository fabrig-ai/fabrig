use crate::render;
use skein_lib::bytes::copy_of;

pub const EXPORTED_CAUSES: &[render::errors::Cause] = &[
    render::errors::Cause::Decode(render::errors::Decode::Unexpected),
    render::errors::Cause::Repository(render::errors::Repository::Stray),
    render::errors::Cause::Composition(render::errors::Composition::Null),
    render::errors::Cause::Schema(render::errors::Schema::Main),
    render::errors::Cause::Evaluation(render::evaluate::FaultKind::Type),
];
#[must_use]
pub fn exported_error(cause: render::errors::Cause) -> render::errors::Error {
    render::errors::Error {
        side: None,
        kind: render::errors::Kind::Exported(render::errors::Exported {
            exporter: copy_of(b"west-prod"),
            task: copy_of(b"vpn"),
            cause,
        }),
        class: render::vocabulary::Class::Configuration,
        at: None,
        also: None,
        target: Some(copy_of(b"carbon-prod")),
        task: Some(copy_of(b"reader")),
    }
}
