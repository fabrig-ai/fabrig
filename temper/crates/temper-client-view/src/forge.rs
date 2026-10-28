//! The forge's resource names and its fixed unserved-part paragraph.
use crate::words::Words;
use alloc::boxed::Box;
use jig_client::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part};
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::forge::words::{self as shared, ResourceWord as Word};
pub mod bindings;
pub(crate) mod connect;
pub(crate) mod forges;
pub mod page;
pub mod words;
fn path_text(builder: &mut Builder, path: &[Box<[u8]>], words: &Words) {
    let (word, holes) = shared::path(path, words.path_bytes());
    words.resource_text(builder, word, &holes);
}

fn shown(builder: &mut Builder, words: &Words) {
    builder.open(Role::Paragraph);
    words.resource_text(builder, Word::Shown, &[]);
    builder.close();
}
/// Draw a part named by a document, without a reference UI slot.
pub fn drawn(builder: &mut Builder, part: &Part, words: &Words) {
    match part {
        Part::Connector { .. } => words.resource_text(builder, Word::Connector, &[]),
        Part::Resource { path, .. } => path_text(builder, path.as_slice(), words),
        Part::Kind { kind, .. } => match shared::kind(*kind) {
            Some(word) => words.resource_text(builder, word, &[]),
            None => words.resource_text(builder, Word::OtherKind, &[Hole::Number(u64::from(*kind))]),
        },
        Part::Panel { .. }
        | Part::Result { .. }
        | Part::Effect { .. }
        | Part::OwnerEffect { .. }
        | Part::Outbox { .. }
        | Part::Hold { .. }
        | Part::Project { .. }
        | Part::Deployment { .. } => shown(builder, words),
    }
}
pub fn owner_part(builder: &mut Builder, part: &OwnerPart, words: &Words) {
    match part.kind {
        OwnerPartKind::Resource => match &part.key {
            OwnerPartKey::Path(path) => path_text(builder, path.as_slice(), words),
            OwnerPartKey::Number(_) => shown(builder, words),
        },
        OwnerPartKind::Pool
        | OwnerPartKind::Result
        | OwnerPartKind::Procedure
        | OwnerPartKind::Effect
        | OwnerPartKind::Outbox
        | OwnerPartKind::Hold => shown(builder, words),
    }
}
