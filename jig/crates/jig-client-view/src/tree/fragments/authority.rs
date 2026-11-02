//! Standalone owner authority fragments (client/domain/view.md, 7 and 10).
//! Retains literal boundaries; words are selected checked language data.
use crate::words::Words;
use crate::words::authority_words::AuthorityPhrase;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_client::contract::{Authority, AuthorityDocument, AuthorityExecutor, AuthorityModels, Pattern, PatternEnd};

fn model_phrase(value: Option<&AuthorityModels>) -> AuthorityPhrase {
    match value {
        None => AuthorityPhrase::ModelsDefault,
        Some(AuthorityModels::All) => AuthorityPhrase::ModelsAll,
        Some(AuthorityModels::Named(names)) => {
            if names.is_empty() {
                AuthorityPhrase::ModelsEmpty
            } else {
                AuthorityPhrase::ModelsNamed
            }
        }
    }
}
fn partial(value: &Authority, rows: u32) -> bool {
    let models = match &value.models {
        Some(AuthorityModels::Named(names)) => names.len() > rows,
        Some(AuthorityModels::All) | None => false,
    };
    models || value.grants.len() > rows || value.delegation.kinds.len() > rows || value.note_resources.len() > rows
}
fn text(b: &mut Builder, words: &Words, phrase: AuthorityPhrase, holes: &[Hole]) {
    let saying = words.book().say(words.language(), phrase.id(), holes).expect("checked authority words and facts");
    b.text(Run::Plain, &saying.text);
}
fn number(b: &mut Builder, words: &Words, phrase: AuthorityPhrase, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}
pub(crate) fn pattern(b: &mut Builder, words: &Words, value: &Pattern, text_limit: u32) {
    text(b, words, AuthorityPhrase::Segments, &[]);
    let last = match &value.last {
        PatternEnd::Exact(bytes) | PatternEnd::Open(bytes) => bytes,
    };
    let reserved = u32::try_from(
        words.book().say(words.language(), AuthorityPhrase::Segments.id(), &[]).expect("checked segments").text.len(),
    )
    .expect("bounded phrase")
    .checked_add(crate::words::literal_segment::display_bytes(last).expect("bounded ending"))
    .expect("bounded path words");
    crate::words::literal_segment::path(
        b,
        value.segments.as_slice(),
        words,
        crate::words::literal_segment::PathDisplay { text: text_limit, reserved, keep_last: false },
    );
    match &value.last {
        PatternEnd::Exact(bytes) => {
            text(b, words, AuthorityPhrase::Exact, &[]);
            crate::words::literal_segment::render(b, bytes);
        }
        PatternEnd::Open(bytes) => {
            text(b, words, AuthorityPhrase::Open, &[]);
            crate::words::literal_segment::render(b, bytes);
        }
    }
}
fn body(b: &mut Builder, value: &Authority, words: &Words, text_limit: u32, rows: u32) {
    number(b, words, AuthorityPhrase::Tools, value.tools);
    number(b, words, AuthorityPhrase::Tasks, u64::from(value.delegation.tasks));
    number(b, words, AuthorityPhrase::Depth, u64::from(value.delegation.depth));
    text(b, words, AuthorityPhrase::Spend, &[Hole::Amount(value.spend)]);
    number(b, words, AuthorityPhrase::Notes, u64::from(value.notes));
    if let Some(deadline) = value.deadline {
        text(b, words, AuthorityPhrase::Deadline, &[Hole::Wall(words.adjusted_wall(deadline))]);
    }
    text(b, words, model_phrase(value.models.as_ref()), &[]);
    if let Some(AuthorityModels::Named(names)) = &value.models {
        for name in names.iter().take(skein_lib::length::usize_of(rows)) {
            b.text(Run::Plain, name);
        }
    }
    text(b, words, AuthorityPhrase::Grants, &[]);
    for grant in value.grants.iter().take(skein_lib::length::usize_of(rows)) {
        number(b, words, AuthorityPhrase::Connector, u64::from(grant.connector));
        number(b, words, AuthorityPhrase::Kind, u64::from(grant.kind));
        pattern(b, words, &grant.pattern, text_limit);
    }
    text(b, words, AuthorityPhrase::Delegation, &[]);
    for kind in value.delegation.kinds.iter().take(skein_lib::length::usize_of(rows)) {
        match kind {
            AuthorityExecutor::Job(code) => {
                words.text(b, crate::words::phrases::jig::JigPhrase::Job, &[Hole::Number(u64::from(*code))]);
            }
            AuthorityExecutor::Charter(code) => number(b, words, AuthorityPhrase::Charter, u64::from(*code)),
            AuthorityExecutor::Procedure(code) => number(b, words, AuthorityPhrase::Procedure, u64::from(*code)),
            AuthorityExecutor::Role(code) => number(b, words, AuthorityPhrase::Role, u64::from(*code)),
        }
    }
    text(b, words, AuthorityPhrase::Resources, &[]);
    for resource in value.note_resources.iter().take(skein_lib::length::usize_of(rows)) {
        number(b, words, AuthorityPhrase::Connector, u64::from(resource.connector));
        pattern(b, words, &resource.pattern, text_limit);
    }
    if partial(value, rows) {
        text(b, words, AuthorityPhrase::Cut, &[]);
    }
}
/// Render a whole admitted payload using a separately reserved fragment capacity.
pub fn render_words(b: &mut Builder, value: &Authority, words: &Words, text_limit: u32, rows: u32) {
    b.open(Role::Description);
    body(b, value, words, text_limit, rows);
    b.close();
}
/// Render a document projection and its explicit cut metadata.
pub fn document_words(b: &mut Builder, value: &AuthorityDocument, words: &Words, text_limit: u32, rows: u32) {
    b.open(Role::Description);
    body(b, &value.authority, words, text_limit, rows);
    if value.cut && !partial(&value.authority, rows) {
        text(b, words, AuthorityPhrase::Cut, &[]);
    }
    b.close();
}

/// Maximum nodes for a supplied authority document, including its cut marker.
#[must_use]
pub fn document_nodes(rows: u32, segments: u32) -> Option<u32> {
    let raw = segments.checked_add(1)?.checked_mul(rows.checked_mul(2)?)?;
    raw.checked_add(rows.checked_mul(8)?.checked_add(11)?)?
        .checked_add(rows.min(jig_client::contract::AUTHORITY_MODELS))?
        .checked_add(1)
}
