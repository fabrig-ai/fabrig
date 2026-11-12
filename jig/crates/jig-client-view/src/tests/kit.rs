//! Fixture trees printed by the text shell (jig/docs/design/client/domain/view.md, 15).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary test fixtures collect values and compare golden files"
)]

use crate::state::objects::Object;
use crate::tests::fixtures::Fixture;
use crate::tests::tree::cards::CLIENT;
use crate::words::Words;
use crate::{JigBindings, tree::cards};
use jig_client::contract::Text;
use skein_lib::Id;
use skein_lib::List;
use skein_ui::{Builder, Role, State, Tree};

pub(crate) fn list<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    List::from_boxed(values.into_iter().collect::<Vec<_>>().into_boxed_slice())
}

/// A fixture may retain room for a later delivered row.
pub(crate) fn list_in<T>(capacity: u32, values: impl IntoIterator<Item = T>) -> List<T> {
    let mut list = List::with_capacity(capacity);
    for value in values {
        list.push(value).ok().unwrap();
    }
    list
}

pub(crate) fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), u32::try_from(bytes.len()).unwrap()).unwrap()
}

pub(crate) fn words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| Words::new(crate::words::phrases::english::english(CLIENT.client.text).unwrap(), 0, 60))
}

/// Literal substitutions preserve the checked phrase shapes and numeric holes.
pub(crate) fn translated_language(marker: &[u8]) -> skein_ui::phrases::Language {
    let mut language = crate::words::phrases::english::language();
    language.wall = skein_ui::phrases::WallFormat::Nanos;
    language.span_unit = skein_ui::phrases::SpanUnit::Nanos;
    for phrase in &mut language.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                if let skein_ui::phrases::Segment::Text(bytes) = segment {
                    *bytes = marker.into();
                }
            }
        }
    }
    language
}

pub(crate) fn limits() -> skein_ui::Limits {
    skein_ui::Limits {
        nodes: 256,
        depth: 10,
        text_per_node: 1024,
        text_bytes: 16384,
        styles_per_node: 2,
        patches: 1536,
        bindings: 64,
        slots: 8,
        markdown_depth: 3,
        ..skein_ui::Limits::EMPTY
    }
}

pub(crate) fn contains(tree: &Tree, bytes: &[u8]) -> bool {
    let text: Vec<_> =
        tree.nodes().iter().flat_map(|node| &node.text).flat_map(|run| run.text.iter().copied()).collect();
    text.windows(bytes.len()).any(|window| window == bytes)
}

/// Fixture states are compared in their declared order, including states that render alike.
pub(crate) struct Goldens {
    name: String,
    next: usize,
}

impl Goldens {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), next: 0 }
    }

    pub(crate) fn case(name: &str, case: u8) -> Self {
        Self::new(format!("{name}/{case}"))
    }

    #[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh rendered goldens")]
    pub(crate) fn check(&mut self, tree: &Tree) {
        let mut bounds = limits();
        bounds.nodes = tree.len();
        let mut print = skein_ui_text::Print::new(&bounds).unwrap();
        skein_ui_text::render(tree, b"jig", Some(crate::regions::JigRegion::Link.region()), &mut print);
        let rendered = print.text.as_slice();
        let path =
            skein_scratch::paths::package().join("golden/rendered").join(&self.name).join(format!("{}.txt", self.next));
        if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, rendered).unwrap();
        }
        let expected = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(
            core::str::from_utf8(rendered).unwrap(),
            core::str::from_utf8(&expected).unwrap(),
            "{}",
            path.display()
        );
        self.next = self.next.checked_add(1).unwrap();
    }
}

pub(crate) fn page(d: &Fixture, kind: u8, words: &Words) -> (Tree, JigBindings) {
    let mut limits = limits();
    limits.nodes = 2000;
    limits.depth = 32;
    limits.bindings = 512;
    limits.text_bytes = 200_000;
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(512, CLIENT.client.text);
    b.open(Role::Page);
    match kind {
        0 | 1 | 3 => {
            crate::tree::pages::main(&mut b, &d.view(), words, &mut bindings);
        }
        _ => {
            crate::tree::pages::not_the_work(&mut b, &d.view(), words, &mut bindings);
        }
    }
    b.close();
    (b.finish(), bindings)
}

pub(crate) fn card(d: &Fixture, id: Id<Object>, words: &Words, bindings: &mut JigBindings) -> Tree {
    bindings.begin();
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    cards::card(&mut b, &d.view(), id, &cards::Place::Elsewhere, words, bindings);
    b.close();
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    assert!(tree.len() <= crate::card_nodes(&CLIENT, &(CLIENT).client).unwrap().checked_add(1).unwrap());
    let facts = d.objects().get(id).unwrap();
    match facts.card {
        crate::state::objects::Card::Deciding { .. } => {
            assert!(tree.nodes().iter().any(|node| node.styles.as_slice().contains(&crate::JigStyle::Card.token())
                && node.states.as_slice().contains(&State::Busy)));
        }
        crate::state::objects::Card::Open
        | crate::state::objects::Card::Refused { .. }
        | crate::state::objects::Card::Unknown { .. }
        | crate::state::objects::Card::NotSent { .. }
        | crate::state::objects::Card::Leaving { .. } => {}
    }
    tree
}
