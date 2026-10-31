mod binding;
pub(crate) mod fixtures;
mod owner;
mod phrases;
mod records;
mod regions;

pub(crate) mod cards;
mod cards_objects;

mod cards_access;

mod pages;

mod proposed_task;
mod recurring;
mod work_pages;

mod turns;

mod page_capacity;

mod owner_wake;
mod refusal_facts;

mod owner_authority;
mod owner_budget;

mod frame_facts;
mod owner_outcome;
mod owner_result;
mod policy_edit;
mod runtime_facts;

mod owner_chip;

mod chip_facts;
mod owner_activity;
mod owner_amendment;
mod resource_facts;

mod owner_history;
mod page_loss;

mod history_pages;
mod owner_milestone;

mod owner_inbox;
mod owner_tree;
mod question_sender;

mod owner_person_task;
mod owner_proposal;

mod conversation_work;
mod conversations;
mod historical_pages;
mod offered;
mod owner_deployment;
mod owner_project;

mod historical_objects;

mod task_proposals;

mod turn_read;

mod limits;

fn text_bytes(node: &skein_ui::Node) -> u32 {
    let mut bytes = 0_u32;
    if let Some(name) = &node.name {
        bytes = u32::try_from(name.len()).unwrap();
    }
    if let Some(value) = &node.value {
        bytes = bytes.checked_add(u32::try_from(value.text.len()).unwrap()).unwrap();
    }
    if let Some(external) = &node.external {
        bytes = bytes.checked_add(u32::try_from(external.len()).unwrap()).unwrap();
    }
    for run in &node.text {
        bytes = bytes.checked_add(u32::try_from(run.text.len()).unwrap()).unwrap();
    }
    bytes
}

mod domain;

mod bridge;

mod handles;

mod shapes;

mod address;

// Match production English's checked segment sum, once per loaded language.
// These selected-language fixtures change literal bytes, retaining the segment shapes.
fn english_phrase_segments(languages: u32) -> u32 {
    let language = crate::phrases::english::language();
    let mut segments = 0_u32;
    for phrase in &language.phrases {
        for form in &phrase.forms {
            segments = segments
                .checked_add(u32::try_from(form.segments.len()).expect("fixture segments fit u32"))
                .expect("whole vocabulary segments fit u32");
        }
    }
    segments.checked_mul(languages).expect("loaded fixture languages fit u32")
}

fn checked_words(
    languages: Box<[skein_ui::phrases::Language]>,
    language: u16,
    offset_minutes: i32,
) -> crate::words::Words {
    let limits = skein_ui::phrases::PhraseLimits {
        languages: u16::try_from(languages.len()).unwrap(),
        phrases: crate::owner_words::phrase_count(),
        holes: 4,
        segments: 20_000,
        text_bytes: 200_000,
        hole_bytes: 4096,
    };
    let book = skein_ui::phrases::Phrasebook::load(crate::phrases::jig::shapes(4096), languages, &limits).unwrap();
    crate::words::Words::new(book, language, offset_minutes)
}
fn english_words() -> crate::words::Words {
    let mut english = crate::phrases::english::language();
    english.wall = skein_ui::phrases::WallFormat::Nanos;
    english.span_unit = skein_ui::phrases::SpanUnit::Nanos;
    checked_words(Box::new([english]), 0, 0)
}

// These rendering fixtures reserve their storage, as an application's world does.
fn render_limits(client: &crate::Limits, words: &crate::words::Words) -> Option<skein_ui::Limits> {
    let mut limits =
        crate::limits::envelope(client, words.jig_capacity()?, &crate::page::Arrangement::none(), 0, 0, 0)?;
    limits.nodes = 16_384;
    limits.text_bytes = 16_777_216;
    limits.bindings = 16_384;
    limits.slots = 16_384;
    Some(limits)
}
