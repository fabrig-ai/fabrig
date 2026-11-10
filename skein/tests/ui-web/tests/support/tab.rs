//! A tab through the page's press and inbox (skein/docs/design/ui/web/testing.md, 3).
use skein_fake_person::{Doing, Face, Find, Found, Next, Step, Why};
use skein_fake_person_browser::{BrowserFace, Outcome, Script, harness};
use skein_lib::Time;
use skein_ui::{Act, NodeId, Role};
use skein_ui_web_tests::SECOND;
use std::collections::{BTreeMap, VecDeque};

enum Part {
    Script(Box<Script>),
    Find(&'static [u8]),
    Press(&'static [u8]),
}

// The fixture keeps its four button nodes while a tab opens beside it.
struct TabStory {
    parts: VecDeque<Part>,
    buttons: BTreeMap<&'static [u8], NodeId>,
    at: u32,
}

impl TabStory {
    fn new() -> Self {
        Self { parts: VecDeque::new(), buttons: BTreeMap::new(), at: 0 }
    }

    fn steps(&mut self, steps: Vec<harness::Step>) {
        self.parts.push_back(Part::Script(Box::new(Script::new(steps.into()))));
    }

    fn press(&mut self, name: &'static [u8]) {
        self.parts.push_back(Part::Press(name));
    }
}

impl harness::Play for TabStory {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        let Some(part) = self.parts.front_mut() else { return Next::Done };
        let next = match part {
            Part::Script(script) => match script.next(face, now) {
                Next::Done => Next::Wait { until: None },
                next => return next,
            },
            Part::Find(name) => {
                let find = Find::exactly_named(Role::Button, name);
                match face.find(&find) {
                    Found::Pending => return Next::Wait { until: None },
                    Found::Absent => return Next::Failed { step: self.at, why: Why::Missing { find } },
                    Found::Node(node) => {
                        self.buttons.insert(name, node);
                    }
                }
                Next::Wait { until: None }
            }
            Part::Press(name) => {
                Next::Do(Doing::Act { node: *self.buttons.get(name).expect("button found"), act: Act::Press })
            }
        };
        self.parts.pop_front();
        self.at += 1;
        next
    }

    fn at(&self) -> u32 {
        self.at
    }

    fn check_deadline(&mut self, face: &mut BrowserFace<'_>, now: Time) {
        if let Some(Part::Script(script)) = self.parts.front_mut() {
            script.check_deadline(face, now);
        }
    }
}

fn status(words: &[u8]) -> harness::Step {
    harness::Step::Person(Step::See { find: Find::exactly_named(Role::Status, words), within: SECOND })
}

fn tabs(tabs: Vec<harness::TabSeen>) -> harness::Step {
    harness::Step::Tabs { tabs: tabs.into(), within: SECOND }
}

fn opened() -> harness::TabSeen {
    harness::TabSeen { title: Some("Opening the page…".as_bytes().into()), address: None, opener: Some(true) }
}

fn landed() -> harness::TabSeen {
    harness::TabSeen { title: None, address: Some(b"/test/tab/landed".as_slice().into()), opener: Some(false) }
}

#[test]
fn a_tab_opens_only_from_a_press() {
    let mut person = TabStory::new();
    person.steps(vec![harness::Step::Person(Step::Go { address: b"/test/tab".as_slice().into() }), status(b"Ready")]);
    for name in [b"Open".as_slice(), b"Point", b"Point outside", b"Close"] {
        person.parts.push_back(Part::Find(name));
    }
    person.press(b"Open");
    person.steps(vec![
        harness::Step::Keep {
            selector: b"[role=status]".as_slice().into(),
            attribute: Some(b"aria-label".as_slice().into()),
            name: b"Opened".as_slice().into(),
        },
        tabs(vec![opened()]),
    ]);
    person.press(b"Point");
    person.steps(vec![tabs(vec![landed()])]);
    for closing in [b"Point outside".as_slice(), b"Close"] {
        person.press(b"Open");
        // Chromium lists the two targets in either order; both must exist before closing one.
        person.steps(vec![tabs(vec![
            harness::TabSeen { title: None, address: None, opener: None },
            harness::TabSeen { title: None, address: None, opener: None },
        ])]);
        person.press(closing);
        person.steps(vec![tabs(vec![landed()])]);
    }
    let mut without_press = Script::new(Box::new([
        harness::Step::Person(Step::Go { address: b"/test/tab?at-start".as_slice().into() }),
        status(b"Refused"),
        tabs(vec![]),
    ]));
    let mut harness = harness::Harness::new(super::setup());
    assert_eq!(harness.run(&mut [&mut person, &mut without_press]), Outcome::Done);
    assert!(!harness.kept().is_empty(), "the opening's status was read");
    for kept in harness.kept() {
        assert_eq!(kept.value, kept.name, "the opening's status");
    }
}
