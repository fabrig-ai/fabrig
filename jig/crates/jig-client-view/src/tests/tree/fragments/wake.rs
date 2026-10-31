use crate::tests::kit::Goldens;
use crate::words::wake_words::WakePhrase;
use crate::{tree::fragments::wake as owner_wake, words::wake_words as owner_wake_words};
use skein_ui::phrases::{
    Form, HoleKind, Language, Phrase, PhraseLimits, Phrasebook, PluralClass, PluralRule, Segment, SpanUnit, WallFormat,
};
use skein_ui::{Builder, Role};

use jig_client::contract::{OwnerWakePolicy, ResultsWake, Subscription, SubscriptionKind, Subscriptions, WakeRule};
use skein_lib::{Duration, List, Wall};

fn builder(nodes: u32, book: &Phrasebook) -> Builder {
    let bytes = owner_wake::phrase_bytes(book).unwrap();
    Builder::new(&skein_ui::Limits {
        nodes: nodes.checked_add(1).unwrap(),
        depth: 4,
        text_per_node: bytes,
        text_bytes: nodes.checked_add(1).unwrap().checked_mul(bytes).unwrap(),
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 0,
        scene_marks: 0,
        scene_bytes: 0,
    })
}

#[test]
fn owner_wake_renderer_keeps_every_rule_result_and_independent_switch() {
    let mut goldens = Goldens::new("tree/fragments/wake/owner_wake_renderer_keeps");
    let book = owner_wake_words::english().unwrap();
    for rule in
        [WakeRule::Never, WakeRule::Immediate, WakeRule::Batch { count: u32::MAX, age: Duration::from_nanos(u64::MAX) }]
    {
        for results in [ResultsWake::Never, ResultsWake::Each, ResultsWake::LastOrFailure] {
            for switches in 0_u8..8 {
                let value = OwnerWakePolicy {
                    words: rule,
                    notices: rule,
                    news: rule,
                    results,
                    questions: switches & 1 != 0,
                    answers: switches & 2 != 0,
                    timers: switches & 4 != 0,
                };
                let mut b = builder(owner_wake::policy_nodes(), &book);
                b.open(Role::Page);
                owner_wake::policy(&mut b, &value, &book, 0);
                b.close();
                let tree = b.finish();
                assert_eq!(tree.len(), owner_wake::policy_nodes() + 1);
                goldens.check(&tree);
            }
        }
    }
}
#[test]
fn owner_subscriptions_renderer_retains_numbers_task_flags_timer_absence_and_topic_pair() {
    let mut goldens = Goldens::new("tree/fragments/wake/owner_subscriptions_renderer");
    let book = owner_wake_words::english().unwrap();
    let mut entries = List::with_capacity(7);
    for kind in [
        SubscriptionKind::Task { target: u64::MAX, held: false, result: false },
        SubscriptionKind::Task { target: 2, held: true, result: false },
        SubscriptionKind::Task { target: 3, held: false, result: true },
        SubscriptionKind::Task { target: 4, held: true, result: true },
        SubscriptionKind::Timer { at: Wall::from_nanos(u64::MAX), period: None },
        SubscriptionKind::Timer { at: Wall::EPOCH, period: Some(Duration::from_nanos(u64::MAX)) },
        SubscriptionKind::Topic { connector: u16::MAX, topic: u64::MAX },
    ] {
        entries.push(Subscription { number: u64::MAX, kind }).unwrap();
    }
    let nodes = owner_wake::subscription_nodes(7).unwrap();
    let mut b = builder(nodes, &book);
    b.open(Role::Page);
    owner_wake::subscriptions(&mut b, &Subscriptions { entries }, &book, 0);
    b.close();
    let tree = b.finish();
    assert!(tree.len() <= nodes + 1);
    goldens.check(&tree);
    assert!(owner_wake::subscription_nodes(u32::MAX).is_none());
    let mut b = builder(owner_wake::subscription_nodes(0).unwrap(), &book);
    b.open(Role::Page);
    owner_wake::subscriptions(&mut b, &Subscriptions { entries: List::with_capacity(0) }, &book, 0);
    b.close();
    assert_eq!(b.finish().len(), 2);
}
#[test]
fn owner_wake_renderer_uses_checked_selected_language_with_reordered_holes() {
    let mut goldens = Goldens::new("tree/fragments/wake/owner_wake_renderer_uses");
    let english = owner_wake_words::language();
    let mut second = owner_wake_words::language();
    for phrase in &mut second.phrases {
        if phrase.id == WakePhrase::Batch.id() {
            phrase.forms = Box::new([Form {
                class: PluralClass::Other,
                segments: Box::new([
                    Segment::Text(b"Age ".as_slice().into()),
                    Segment::Hole { index: 1, kind: HoleKind::Number },
                    Segment::Text(b"; count ".as_slice().into()),
                    Segment::Hole { index: 0, kind: HoleKind::Number },
                ]),
            }]);
        }
    }
    let book = Phrasebook::load(
        owner_wake_words::shapes(),
        Box::new([english, second]),
        &PhraseLimits { languages: 2, phrases: 27, holes: 2, segments: 200, text_bytes: 4000, hole_bytes: 0 },
    )
    .unwrap();
    let value = OwnerWakePolicy {
        words: WakeRule::Never,
        notices: WakeRule::Immediate,
        news: WakeRule::Batch { count: 7, age: Duration::from_nanos(13) },
        results: ResultsWake::Each,
        questions: true,
        answers: false,
        timers: true,
    };
    let mut b = builder(owner_wake::policy_nodes(), &book);
    b.open(Role::Page);
    owner_wake::policy(&mut b, &value, &book, 1);
    b.close();
    let tree = b.finish();
    goldens.check(&tree);
    // Missing phrases fail before they reach the renderer.
    let absent = Language {
        phrases: Box::<[Phrase]>::default(),
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
    };
    assert_eq!(
        Phrasebook::load(
            owner_wake_words::shapes(),
            Box::new([absent]),
            &PhraseLimits { languages: 1, phrases: 27, holes: 2, segments: 100, text_bytes: 2000, hole_bytes: 0 }
        )
        .unwrap_err(),
        skein_ui::phrases::Invalid::Missing { id: WakePhrase::Policy.id() }
    );
}
