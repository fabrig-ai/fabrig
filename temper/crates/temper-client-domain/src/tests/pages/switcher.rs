use super::fixtures;
use crate::{
    Address, ChatsFilter,
    pages::{
        self, Act as PageAct, Event, Pages, Request,
        switcher::{Act, Form},
    },
};
use alloc::boxed::Box;
use jig_client::{Key, Outcome, Told, contract as c};
use skein_lib::{List, Queue, Token};

const MEMBER: u32 = 2;
const MADE: u32 = 3;
const NAME_BYTES: usize = 64;
struct Fixture {
    root: crate::Domain,
    pages: Pages,
    watch: Token,
    out: Queue<Request>,
}
fn me() -> c::Me {
    let mut me = fixtures::chats::me();
    let mut member = me.projects.get(0).unwrap().clone();
    member.project = MEMBER;
    member.role = c::HeldRole::Policy { role: 1 };
    member.name = c::DocumentText::complete(fixtures::acts::text(b"Member"));
    let owner = me.projects.get(0).unwrap().clone();
    me.projects = List::with_capacity(3);
    me.projects.push(owner).unwrap();
    me.projects.push(member).unwrap();
    me
}
fn fixture() -> Fixture {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let watch = fixtures::start(&mut root, List::with_capacity(0));
    let mut f = Fixture { root, pages: Pages::new(), watch, out: Queue::uncapped() };
    update_me(&mut f, me());
    event(&mut f, Event::Went { address: chats() });
    while f.out.pop().is_some() {}
    f
}
fn chats() -> Address {
    Address::Chats { filter: ChatsFilter::All }
}
fn event(f: &mut Fixture, event: Event) {
    pages::step(&mut f.pages, &fixtures::env(&fixtures::limits()), f.root.client(), event, &mut f.out);
}
fn act(f: &mut Fixture, act: Act) {
    event(f, Event::Act { act: PageAct::Switcher(act) });
}
fn update_me(f: &mut Fixture, me: c::Me) {
    crate::step(
        &mut f.root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: f.watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut Queue::uncapped(),
    );
    event(f, Event::Told { told: Told::Changed { handle: pages::me_handle() } });
}
fn sent(f: &mut Fixture) -> (Key, c::Ask) {
    let Some(Request::Client(jig_client::Event::Send { key, name: None, request: jig_client::Ask::Jig(ask) })) =
        f.out.pop()
    else {
        panic!("one keyed jig ask")
    };
    assert_eq!(key.asker, crate::owner::Owner::Pages.asker());
    assert!(f.out.is_empty());
    (key, *ask)
}
fn answer(f: &mut Fixture, key: Key, answer: c::Answer) {
    event(
        f,
        Event::Told { told: Told::Answered { key, outcome: Outcome::Answered(jig_client::Answered::Jig(answer)) } },
    );
}
fn done(outcome: c::DocumentOutcome) -> c::Answer {
    c::Answer::Done(c::Outcome::Document(outcome))
}
fn refused() -> c::Answer {
    c::Answer::Refused { refusal: c::Refusal::NameTaken, at: None }
}
fn rename(f: &mut Fixture) -> Key {
    act(f, Act::Rename);
    assert_eq!(f.pages.switcher().rename_words(), b"Project 1");
    act(f, Act::RenameWords { words: b"Renamed".as_slice().into() });
    act(f, Act::Save);
    let (key, ask) = sent(f);
    assert_eq!(ask, c::Ask::Rename { project: fixtures::PROJECT, name: fixtures::acts::text(b"Renamed") });
    key
}
fn create(f: &mut Fixture) -> Key {
    act(f, Act::NewWords { words: b"New".as_slice().into() });
    act(f, Act::Create);
    let (key, ask) = sent(f);
    assert_eq!(ask, c::Ask::MakeProject { name: fixtures::acts::text(b"New"), policy: 1 });
    key
}
#[test]
fn rename_sends_jigs_rename_with_the_new_name() {
    let mut f = fixture();
    rename(&mut f);
    assert_eq!(f.pages.switcher().pending(), Some(Form::Rename));
}
#[test]
fn rename_is_not_opened_on_a_project_the_person_does_not_own() {
    let mut f = fixture();
    event(&mut f, Event::Act { act: PageAct::Project { project: MEMBER } });
    assert!(fixtures::acts::sent(&mut f.out).is_empty());
    act(&mut f, Act::Rename);
    assert_eq!(f.pages.switcher().renaming(), None);
    assert!(f.out.is_empty());
}
#[test]
fn an_empty_or_unchanged_name_closes_without_sending() {
    for words in [b"".as_slice(), b"   ", b"Project 1", b"  Project 1  "] {
        let mut f = fixture();
        act(&mut f, Act::Rename);
        act(&mut f, Act::RenameWords { words: words.into() });
        act(&mut f, Act::Save);
        assert_eq!(f.pages.switcher().renaming(), None);
        assert_eq!(f.pages.switcher().pending(), None);
        assert!(f.out.is_empty());
    }
}
#[test]
fn rename_sends_words_without_spaces_at_the_ends() {
    let mut f = fixture();
    act(&mut f, Act::Rename);
    act(&mut f, Act::RenameWords { words: b"  New name  ".as_slice().into() });
    act(&mut f, Act::Save);
    assert_eq!(sent(&mut f).1, c::Ask::Rename { project: fixtures::PROJECT, name: fixtures::acts::text(b"New name") });
}
#[test]
fn cancel_closes_the_form_and_sends_nothing() {
    let mut f = fixture();
    act(&mut f, Act::Rename);
    act(&mut f, Act::Cancel);
    act(&mut f, Act::Save);
    assert_eq!(f.pages.switcher().renaming(), None);
    assert!(f.out.is_empty());
}
#[test]
fn a_renamed_answer_closes_the_form() {
    let mut f = fixture();
    let key = rename(&mut f);
    answer(&mut f, key, done(c::DocumentOutcome::Renamed { project: fixtures::PROJECT }));
    assert_eq!(f.pages.switcher().renaming(), None);
    assert_eq!(f.pages.switcher().pending(), None);
    assert_eq!(f.pages.switcher().refusal(), None);
    assert!(f.out.is_empty());
}
#[test]
fn a_refused_rename_keeps_the_form_and_its_refusal() {
    let mut f = fixture();
    let key = rename(&mut f);
    answer(&mut f, key, refused());
    assert_eq!(f.pages.switcher().renaming(), Some(fixtures::PROJECT));
    assert_eq!(f.pages.switcher().rename_words(), b"Renamed");
    assert_eq!(f.pages.switcher().pending(), None);
    assert_eq!(
        f.pages.switcher().refusal(),
        Some((Form::Rename, &Outcome::Answered(jig_client::Answered::Jig(refused()))))
    );
    act(&mut f, Act::Save);
    let (retry, _) = sent(&mut f);
    assert_ne!(retry, key);
    assert_eq!(f.pages.switcher().refusal(), None);
}
#[test]
fn a_new_project_sends_make_project_on_policy_one() {
    let mut f = fixture();
    create(&mut f);
    assert_eq!(f.pages.switcher().pending(), Some(Form::New));
}
#[test]
fn an_empty_new_name_sends_nothing() {
    let mut f = fixture();
    act(&mut f, Act::Create);
    assert!(f.pages.switcher().new_words().is_empty());
    assert!(f.out.is_empty());
}
#[test]
fn a_refused_new_project_keeps_its_words_and_refusal() {
    let mut f = fixture();
    let key = create(&mut f);
    answer(&mut f, key, refused());
    assert_eq!(f.pages.switcher().new_words(), b"New");
    assert_eq!(f.pages.switcher().pending(), None);
    assert_eq!(
        f.pages.switcher().refusal(),
        Some((Form::New, &Outcome::Answered(jig_client::Answered::Jig(refused()))))
    );
    act(&mut f, Act::NewWords { words: b"Other".as_slice().into() });
    assert_eq!(f.pages.switcher().refusal(), None);
    assert!(f.out.is_empty());
}
fn with_made_project() -> c::Me {
    let mut me = me();
    let mut made = me.projects.get(0).unwrap().clone();
    made.project = MADE;
    made.name = c::DocumentText::complete(fixtures::acts::text(b"New"));
    me.projects.push(made).unwrap();
    me
}
#[test]
fn a_made_project_is_opened_on_its_home_page() {
    for listed_first in [false, true] {
        let mut f = fixture();
        let key = create(&mut f);
        if listed_first {
            update_me(&mut f, with_made_project());
            assert_eq!(f.pages.project(), Some(fixtures::PROJECT));
            assert!(f.out.is_empty());
        }
        answer(&mut f, key, done(c::DocumentOutcome::ProjectMade { project: MADE }));
        assert!(f.pages.switcher().new_words().is_empty());
        assert_eq!(f.pages.switcher().pending(), None);
        if !listed_first {
            assert_eq!(f.pages.project(), Some(fixtures::PROJECT));
            assert!(f.out.is_empty());
            update_me(&mut f, with_made_project());
        }
        assert_eq!(f.pages.project(), Some(MADE));
        let home = Address::Home { project: MADE };
        assert_eq!(f.out.pop(), Some(Request::Show { address: home.clone() }));
        while let Some(request) = f.out.pop() {
            match request {
                Request::Show { .. } => panic!("the made project opens once"),
                Request::Push(_) | Request::Screen { .. }
                | Request::KeepLook { .. }
                | Request::Client(_)
                | Request::Replace { .. }
                | Request::OpenTab { .. } => {}
            }
        }
        event(&mut f, Event::Went { address: home.clone() });
        assert_eq!(f.pages.address(), &home);
        while f.out.pop().is_some() {}
        update_me(&mut f, with_made_project());
        assert!(f.out.is_empty(), "membership updates do not reopen the project");
    }
}
#[test]
fn one_request_at_a_time() {
    for form in [Form::Rename, Form::New] {
        let mut f = fixture();
        act(&mut f, Act::Rename);
        act(&mut f, Act::RenameWords { words: b"Renamed".as_slice().into() });
        act(&mut f, Act::NewWords { words: b"New".as_slice().into() });
        act(&mut f, if form == Form::Rename { Act::Save } else { Act::Create });
        let (key, _) = sent(&mut f);
        event(&mut f, Event::Told { told: Told::Pending { key } });
        for _ in 0_u32..3 {
            act(&mut f, Act::Save);
            act(&mut f, Act::Create);
        }
        assert_eq!(f.pages.switcher().pending(), Some(form));
        assert!(f.out.is_empty());
    }
}
#[test]
fn choosing_another_project_closes_the_form() {
    let mut f = fixture();
    act(&mut f, Act::Rename);
    event(&mut f, Event::Act { act: PageAct::Project { project: fixtures::PROJECT } });
    assert_eq!(f.pages.switcher().renaming(), Some(fixtures::PROJECT));
    event(&mut f, Event::Act { act: PageAct::Project { project: MEMBER } });
    assert_eq!(f.pages.switcher().renaming(), None);
    assert!(fixtures::acts::sent(&mut f.out).is_empty());
}
#[test]
fn leaving_for_a_page_of_no_project_closes_the_form() {
    for address in [Address::Accounts, Address::Forges, Address::Inbox { filter: crate::address::Filter::ALL }] {
        let mut f = fixture();
        act(&mut f, Act::Rename);
        event(&mut f, Event::Went { address });
        assert_eq!(f.pages.switcher().renaming(), None);
    }
}
#[test]
fn form_words_keep_jigs_name_bound() {
    let mut f = fixture();
    act(&mut f, Act::Rename);
    let valid: Box<[u8]> = Box::from([b'a'; NAME_BYTES]);
    let too_long: Box<[u8]> = Box::from([b'b'; NAME_BYTES + 1]);
    act(&mut f, Act::RenameWords { words: valid.clone() });
    act(&mut f, Act::NewWords { words: valid.clone() });
    act(&mut f, Act::RenameWords { words: too_long.clone() });
    act(&mut f, Act::NewWords { words: too_long });
    assert_eq!(f.pages.switcher().rename_words(), valid.as_ref());
    assert_eq!(f.pages.switcher().new_words(), valid.as_ref());
    act(&mut f, Act::Create);
    assert_eq!(
        sent(&mut f).1,
        c::Ask::MakeProject { name: c::Text::new(valid, u32::try_from(NAME_BYTES).unwrap()).unwrap(), policy: 1 }
    );
}
#[test]
fn an_unrelated_answer_does_not_finish_the_request() {
    let mut f = fixture();
    let key = rename(&mut f);
    answer(
        &mut f,
        Key { number: key.number + 1, ..key },
        done(c::DocumentOutcome::Renamed { project: fixtures::PROJECT }),
    );
    assert_eq!(f.pages.switcher().pending(), Some(Form::Rename));
    assert_eq!(f.pages.switcher().renaming(), Some(fixtures::PROJECT));
}
