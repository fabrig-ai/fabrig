//! The forge's resource names and its fixed unserved-part paragraph.
use crate::words::Words;
use alloc::boxed::Box;
use jig_client::contract::{ApplicationPart, Part, PartKey, PartKind};
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
pub fn drawn(builder: &mut Builder, part: &ApplicationPart, words: &Words) {
    match part {
        ApplicationPart::Connector { .. } => words.resource_text(builder, Word::Connector, &[]),
        ApplicationPart::Resource { path, .. } => path_text(builder, path.as_slice(), words),
        ApplicationPart::Kind { kind, .. } => match shared::kind(*kind) {
            Some(word) => words.resource_text(builder, word, &[]),
            None => words.resource_text(builder, Word::OtherKind, &[Hole::Number(u64::from(*kind))]),
        },
        ApplicationPart::Panel { .. }
        | ApplicationPart::Result { .. }
        | ApplicationPart::Effect { .. }
        | ApplicationPart::OwnerEffect { .. }
        | ApplicationPart::Outbox { .. }
        | ApplicationPart::Hold { .. }
        | ApplicationPart::Project { .. }
        | ApplicationPart::Deployment { .. } => shown(builder, words),
    }
}
pub fn owner_part(builder: &mut Builder, part: &Part, words: &Words) {
    match part.kind {
        PartKind::Resource => match &part.key {
            PartKey::Path(path) => path_text(builder, path.as_slice(), words),
            PartKey::Number(_) => shown(builder, words),
        },
        PartKind::Pool
        | PartKind::Result
        | PartKind::Procedure
        | PartKind::Effect
        | PartKind::Outbox
        | PartKind::Hold => shown(builder, words),
    }
}
