//! Normal jig phrasebooks compose every exact owner fragment (view.md, 7).
//! Composition happens while languages load, never while a page renders.
use alloc::boxed::Box;
use skein_lib::List;
use skein_ui::phrases::{Language, PhraseShape, Phrasebook};

#[must_use]
pub fn phrase_count() -> u32 {
    jig_client::words::PHRASES
        .checked_add(u32::try_from(crate::words::phrases::jig::ALL.len()).expect("view count fits"))
        .expect("composed count fits")
        .checked_add(u32::try_from(crate::words::authority_words::ALL.len()).expect("authority phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::wake_words::ALL.len()).expect("wake phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::proposed_task_words::ALL.len()).expect("batch phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::result_words::ALL.len()).expect("result phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::budget_words::ALL.len()).expect("budget phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::outcome_words::ALL.len()).expect("outcome phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::chip_words::ALL.len()).expect("chip phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::live_call_words::ALL.len()).expect("live call phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::amendment_words::ALL.len()).expect("amendment phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::activity_words::ALL.len()).expect("activity phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::milestone_words::ALL.len()).expect("milestone phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::history_words::ALL.len()).expect("history phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::tree_words::ALL.len()).expect("tree phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::proposal_words::ALL.len()).expect("proposal phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::project_words::ALL.len()).expect("project phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::inbox_words::ALL.len()).expect("Inbox phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::person_task_words::ALL.len()).expect("person phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(
            u32::try_from(crate::words::historical_pages_words::ALL.len()).expect("historical phrase count fits"),
        )
        .expect("composed phrase count fits")
        .checked_add(u32::try_from(crate::words::deployment_words::ALL.len()).expect("deployment phrase count fits"))
        .expect("composed phrase count fits")
        .checked_add(
            u32::try_from(crate::words::historical_words::ALL.len()).expect("historical object phrase count fits"),
        )
        .expect("composed phrase count fits")
}

#[must_use]
pub fn shapes(normal: Box<[PhraseShape]>, text: u32) -> Box<[PhraseShape]> {
    let mut result = List::with_capacity(phrase_count());
    for shape in jig_client::words::phrase_shapes(text) {
        result.push(shape).expect("public word shapes fit");
    }
    for shape in normal {
        result.push(shape).expect("normal shapes fit");
    }
    for shape in crate::words::authority_words::shapes() {
        result.push(shape).expect("authority shapes fit");
    }
    for shape in crate::words::wake_words::shapes() {
        result.push(shape).expect("wake shapes fit");
    }
    for shape in crate::words::proposed_task_words::shapes() {
        result.push(shape).expect("batch shapes fit");
    }
    for shape in crate::words::result_words::shapes() {
        result.push(shape).expect("result shapes fit");
    }
    for shape in crate::words::budget_words::shapes() {
        result.push(shape).expect("budget shapes fit");
    }
    for shape in crate::words::outcome_words::shapes() {
        result.push(shape).expect("outcome shapes fit");
    }
    for shape in crate::words::chip_words::shapes() {
        result.push(shape).expect("chip shapes fit");
    }
    for shape in crate::words::live_call_words::shapes() {
        result.push(shape).expect("live call shapes fit");
    }
    for shape in crate::words::amendment_words::shapes() {
        result.push(shape).expect("amendment shapes fit");
    }
    for shape in crate::words::activity_words::shapes() {
        result.push(shape).expect("activity shapes fit");
    }
    for shape in crate::words::history_words::shapes() {
        result.push(shape).expect("history shapes fit");
    }
    for shape in crate::words::milestone_words::shapes() {
        result.push(shape).expect("milestone shapes fit");
    }
    for shape in crate::words::inbox_words::shapes() {
        result.push(shape).expect("Inbox shapes fit");
    }
    for shape in crate::words::tree_words::shapes() {
        result.push(shape).expect("tree shapes fit");
    }
    for shape in crate::words::proposal_words::shapes() {
        result.push(shape).expect("proposal shapes fit");
    }
    for shape in crate::words::person_task_words::shapes() {
        result.push(shape).expect("person shapes fit");
    }
    for shape in crate::words::historical_pages_words::shapes() {
        result.push(shape).expect("historical shapes fit");
    }
    for shape in crate::words::project_words::shapes() {
        result.push(shape).expect("project shapes fit");
    }
    for shape in crate::words::deployment_words::shapes() {
        result.push(shape).expect("deployment shapes fit");
    }
    for value in crate::words::historical_words::shapes() {
        result.push(value).expect("historical object words fit");
    }
    result.into_boxed()
}

#[must_use]
pub fn language(normal: Language) -> Language {
    let mut phrases = List::with_capacity(phrase_count());
    for phrase in jig_client::words::english().phrases {
        phrases.push(phrase).expect("public words fit");
    }
    for phrase in normal.phrases {
        phrases.push(phrase).expect("normal words fit");
    }
    for phrase in crate::words::authority_words::english().phrases {
        phrases.push(phrase).expect("authority words fit");
    }
    for phrase in crate::words::wake_words::language().phrases {
        phrases.push(phrase).expect("wake words fit");
    }
    for phrase in crate::words::proposed_task_words::language().phrases {
        phrases.push(phrase).expect("batch words fit");
    }
    for phrase in crate::words::result_words::language().phrases {
        phrases.push(phrase).expect("result words fit");
    }
    for phrase in crate::words::budget_words::language().phrases {
        phrases.push(phrase).expect("budget words fit");
    }
    for phrase in crate::words::outcome_words::english().phrases {
        phrases.push(phrase).expect("outcome words fit");
    }
    for phrase in crate::words::chip_words::language().phrases {
        phrases.push(phrase).expect("chip words fit");
    }
    for phrase in crate::words::live_call_words::language().phrases {
        phrases.push(phrase).expect("live call words fit");
    }
    for phrase in crate::words::amendment_words::language().phrases {
        phrases.push(phrase).expect("amendment words fit");
    }
    for phrase in crate::words::activity_words::language().phrases {
        phrases.push(phrase).expect("activity words fit");
    }
    for phrase in crate::words::history_words::language().phrases {
        phrases.push(phrase).expect("history words fit");
    }
    for phrase in crate::words::milestone_words::language().phrases {
        phrases.push(phrase).expect("milestone words fit");
    }
    for phrase in crate::words::inbox_words::language().phrases {
        phrases.push(phrase).expect("Inbox words fit");
    }
    for phrase in crate::words::tree_words::language().phrases {
        phrases.push(phrase).expect("tree words fit");
    }
    for phrase in crate::words::proposal_words::language().phrases {
        phrases.push(phrase).expect("tree words fit");
    }
    for phrase in crate::words::person_task_words::language().phrases {
        phrases.push(phrase).expect("person words fit");
    }
    for phrase in crate::words::historical_pages_words::language().phrases {
        phrases.push(phrase).expect("historical words fit");
    }
    for phrase in crate::words::project_words::language().phrases {
        phrases.push(phrase).expect("project words fit");
    }
    for phrase in crate::words::deployment_words::language().phrases {
        phrases.push(phrase).expect("deployment words fit");
    }
    for value in crate::words::historical_words::language().phrases {
        phrases.push(value).expect("historical object words fit");
    }
    Language { plural: normal.plural, span_unit: normal.span_unit, wall: normal.wall, phrases: phrases.into_boxed() }
}

/// All loaded languages contribute to the saying envelope.
#[must_use]
pub fn phrase_bytes(book: &Phrasebook) -> Option<u32> {
    let mut largest = 0;
    for word in jig_client::words::ALL {
        largest = largest.max(book.capacity(word.id())?);
    }
    for phrase in crate::words::authority_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::wake_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::proposed_task_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::result_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::budget_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::outcome_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::chip_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::live_call_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::amendment_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::activity_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::history_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::milestone_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::inbox_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::tree_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::proposal_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::person_task_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::historical_pages_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::project_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::deployment_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    for phrase in crate::words::historical_words::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    Some(largest)
}
