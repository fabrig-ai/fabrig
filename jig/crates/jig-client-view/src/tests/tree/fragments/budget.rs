use crate::tests::kit::Goldens;
use crate::tree::fragments::budget as view;
use crate::words::Words;
use skein_ui::phrases::{PhraseLimits, Phrasebook, Segment, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::{Funder, OwnerBudget, OwnerNumbers, PoolLedger};
use skein_lib::{Wall, bytes::copy_of};

fn numbers() -> OwnerNumbers {
    OwnerNumbers { budget: 100, spent: 4, spent_below: 7, reserved: 11 }
}
fn budget() -> OwnerBudget {
    OwnerBudget {
        numbers: numbers(),
        left: 17,
        funder: Funder::Task(u64::MAX),
        deadline: Some(Wall::from_nanos(120_000_000_000)),
    }
}
fn ledger() -> PoolLedger {
    PoolLedger {
        source: Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX - 1 },
        parent: Some(Funder::Recurring { project: u32::MAX - 1, task: u64::MAX - 2, period: u64::MAX - 3 }),
        numbers: numbers(),
        left: 17,
        closed: true,
        began: Some(Wall::from_nanos(120_000_000_000)),
        next: Some(Wall::from_nanos(180_000_000_000)),
    }
}
fn words() -> Words {
    crate::tests::english_words()
}

fn budget_tree(value: &OwnerBudget, words: &Words) -> Tree {
    let required =
        crate::tests::render_limits(&crate::tests::tree::cards::CLIENT, words).expect("whole checked capacity");
    let mut builder = Builder::new(&required);
    view::render(&mut builder, value, words);
    builder.finish()
}
fn ledger_tree(value: &PoolLedger, words: &Words) -> Tree {
    let required =
        crate::tests::render_limits(&crate::tests::tree::cards::CLIENT, words).expect("whole checked capacity");
    let mut builder = Builder::new(&required);
    view::ledger(&mut builder, value, words);
    builder.finish()
}

#[test]
fn budget_says_all_owner_numbers_exact_task_funder_and_only_the_supplied_deadline() {
    let mut goldens = Goldens::new("tree/fragments/budget/budget_says_all");
    let value = budget();
    let words = words();
    let tree = budget_tree(&value, &words);
    assert_eq!(tree.len(), 9);
    goldens.check(&tree);
    let mut without_deadline = value;
    without_deadline.deadline = None;
    let tree = budget_tree(&without_deadline, &words);
    assert_eq!(tree.len(), 8);
    goldens.check(&tree);
}
#[test]
fn ledger_keeps_distinct_parent_periods_full_source_widths_and_explicit_optional_facts() {
    let mut goldens = Goldens::new("tree/fragments/budget/ledger_keeps_distinct");
    let value = ledger();
    let words = words();
    let tree = ledger_tree(&value, &words);
    assert_eq!(tree.len(), 13);
    goldens.check(&tree);
    let mut absent = value;
    absent.parent = None;
    absent.began = None;
    absent.next = None;
    absent.closed = false;
    let tree = ledger_tree(&absent, &words);
    assert_eq!(tree.len(), 9);
    goldens.check(&tree);
}
#[test]
fn budget_and_shared_funder_sayings_borrow_selected_normal_words_and_person_offset() {
    let mut goldens = Goldens::new("tree/fragments/budget/budget_and_shared");
    let mut translated = crate::words::phrases::english::language();
    translated.wall = WallFormat::Clock;
    for phrase in &mut translated.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                match segment {
                    Segment::Text(bytes) => *bytes = copy_of(b"selected "),
                    Segment::Hole { .. } => {}
                }
            }
        }
    }
    let languages: Box<[_]> = Box::new([crate::words::phrases::english::language(), translated]);
    let mut segments = 0_u32;
    let mut text_bytes = 0_u32;
    for language in &languages {
        for phrase in &language.phrases {
            for form in &phrase.forms {
                segments = segments
                    .checked_add(u32::try_from(form.segments.len()).expect("form segment count fits"))
                    .expect("both complete languages' segment count fits");
                for segment in &form.segments {
                    match segment {
                        Segment::Text(bytes) => {
                            text_bytes = text_bytes
                                .checked_add(u32::try_from(bytes.len()).expect("literal byte count fits"))
                                .expect("both complete languages' literal byte count fits");
                        }
                        Segment::Hole { .. } => {}
                    }
                }
            }
        }
    }
    let limits = PhraseLimits {
        languages: 2,
        phrases: crate::words::owner_words::phrase_count(),
        holes: 4,
        segments,
        text_bytes,
        hole_bytes: 256,
    };
    let book = Phrasebook::load(crate::words::phrases::jig::shapes(256), languages, &limits)
        .expect("normal composed vocabulary checks both supplied languages whole");
    let words = Words::new(book, 1, -1);
    let tree = budget_tree(&budget(), &words);
    goldens.check(&tree);
    let tree = ledger_tree(&ledger(), &words);
    goldens.check(&tree);

    let mut value = budget();
    value.deadline = Some(Wall::EPOCH);
    goldens.check(&budget_tree(&value, &words));
}
