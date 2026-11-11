#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "step tests use ordinary Rust for assertions and fixture selection"
)]
use super::*;
use crate::{forge, tests::fixtures};
use forge::forges::{Account, Adoption, Installation, Offer, StartsWith};
use skein_lib::Env;

const PROJECT: u32 = 7;
const GITHUB: u16 = 2;
const FORGEJO: u16 = 4;
const INSTALLATION: u64 = 19;
const PAGE: u32 = 100; // Reachable's wire page bound.
const TOTAL: u32 = 213; // The request's large forge, over three pages.

fn forge_row(number: u16, kind: Kind, mode: Mode) -> Row {
    Row {
        number,
        kind,
        mode,
        host: b"git.example".as_slice().into(),
        connection: Connection::Connected { id: 1, login: b"bot".as_slice().into() },
        makings: Box::new([]),
        read_only: false,
        fetched: None,
        offer: Offer::Offered,
        first: None,
        starts_with: StartsWith::Nothing,
        installations: Box::new([]),
        ended: None,
        app: None,
        web: None,
        repositories: Box::new([]),
        owned: None,
        admin: true,
        ended_stop: None,
    }
}
fn rows() -> State {
    let mut github = forge_row(GITHUB, Kind::GitHub, Mode::Connected);
    github.installations = Box::new([Installation {
        id: INSTALLATION,
        account: Some(Account { id: 3, login: b"acme".as_slice().into() }),
        suspended: false,
        stray: false,
    }]);
    State::Rows(Box::new([forge_row(FORGEJO, Kind::Forgejo, Mode::Managed), github]))
}
fn env() -> Env<Limits> {
    fixtures::env(&fixtures::limits()).with(fixtures::limits().forge)
}
fn opened() -> (Side, Queue<Request>) {
    let mut side = Side::new(&env().limits, fixtures::SEED);
    side.forges = rows();
    side.served = super::super::Served::Yes;
    side.party = Some(1);
    let mut out = Queue::uncapped();
    forge::step(
        &mut side,
        &env(),
        forge::Event::Went { address: forge::Address::Adopt { project: PROJECT } },
        &mut out,
    );
    (side, out)
}
fn act(side: &mut Side, act: Act) -> Queue<Request> {
    let mut out = Queue::uncapped();
    super::act(side, env().limits, act, &mut out);
    out
}
fn reached(id: u64, name: &[u8], private: bool) -> w::Reached {
    let owner = w::Account::new(&w::CEILINGS, w::AccountParts { id: 3, login: b"acme".as_slice().into() }).unwrap();
    w::Reached::new(&w::CEILINGS, w::ReachedParts { id, owner, name: name.into(), private }).unwrap()
}
fn page(first: u32, count: u32, more: bool) -> w::Page {
    let mut rows = List::with_capacity(PAGE);
    for id in first..first.checked_add(count).unwrap() {
        let number = skein_lib::Decimal::of(u64::from(id));
        let name = bytes::concat(&[b"repo-", number.as_bytes()]);
        rows.push(reached(u64::from(id), &name, false)).unwrap();
    }
    w::Page::new(&w::CEILINGS, w::PageParts { rows, more }).unwrap()
}
fn reading(side: &Side) -> Token {
    match side.sheet().unwrap().reading {
        Reading::Asked { key, .. } => key,
        Reading::Idle | Reading::Refused(_) => panic!("no read"),
    }
}
fn answer(side: &mut Side, page: w::Page) -> Queue<Request> {
    let key = reading(side);
    let mut out = Queue::uncapped();
    received(side, key, Ok(page), env().limits, &mut out);
    out
}
fn sent(out: &Queue<Request>) -> w::Request {
    for request in out {
        if let Request::Send { ask: jig_client::Ask::Family { record, .. }, .. } = request {
            let mut reader = skein_lib::Reader::new(&record.bytes);
            return w::Requesting::decode(&w::CEILINGS, &mut reader).unwrap().into_parts().request;
        }
    }
    panic!("no send")
}
fn landing(side: &mut Side) {
    super::super::write(
        side.form.as_mut().unwrap(),
        forge::Text::Landing,
        b"main".as_slice().into(),
        Typed::Bytes,
        env().limits.field_bytes,
        env().limits.checks,
    );
}
fn pending(side: &Side) -> Token {
    side.form().unwrap().pending.unwrap().0
}
fn adopted_answer() -> forge::Answer {
    forge::Answer::Adopted(forge::Adopted {
        forge: GITHUB,
        repository: 1,
        owner: b"acme".as_slice().into(),
        name: b"repo-1".as_slice().into(),
        role: forge::Role::Owned,
        home: false,
        kinds: forge::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: forge::Protection::Unknown,
    })
}

#[test]
fn opening_chooses_forges_and_leaving_drops_the_answer() {
    let (mut side, out) = opened();
    let sheet = side.sheet().unwrap();
    assert_eq!((sheet.forge, sheet.installation, sheet.target), (Some(GITHUB), Some(INSTALLATION), Some(FORGEJO)));
    assert!({
        let w::Request::Reachable(read) = sent(&out) else { panic!("reachable") };
        read.forge() == GITHUB && *read.installation() == Some(INSTALLATION) && read.page() == 1
    });
    let key = reading(&side);
    forge::step(&mut side, &env(), forge::Event::Left, &mut Queue::uncapped());
    let mut out = Queue::uncapped();
    received(&mut side, key, Ok(page(1, PAGE, true)), env().limits, &mut out);
    assert!(side.sheet().is_none());
    assert!(out.is_empty());
}

#[test]
fn scrolling_reads_one_page_and_search_finishes_213_rows() {
    let (mut side, _) = opened();
    assert!(act(&mut side, Act::NearEnd).is_empty(), "one read at a time");
    assert!(answer(&mut side, page(1, PAGE, true)).is_empty(), "read only when wanted");
    assert_eq!(count(side.sheet().unwrap(), &side.forges), Count::SoFar(PAGE));
    let out = act(&mut side, Act::NearEnd);
    assert!({
        let w::Request::Reachable(read) = sent(&out) else { panic!("reachable") };
        read.page() == 2
    });
    act(&mut side, Act::Find(b"REPO-213".as_slice().into()));
    let out = answer(&mut side, page(101, PAGE, true));
    assert!({
        let w::Request::Reachable(read) = sent(&out) else { panic!("reachable") };
        read.page() == 3
    });
    assert_eq!(count(side.sheet().unwrap(), &side.forges), Count::Searching(200));
    assert!(answer(&mut side, page(201, 13, false)).is_empty());
    let sheet = side.sheet().unwrap();
    assert_eq!(sheet.rows.len(), TOTAL);
    assert_eq!(count(sheet, &side.forges), Count::Found { found: 1, of: TOTAL });
    assert_eq!(shown(sheet, PROJECT, &side.forges).get(0).unwrap().row.id(), 213);
}

#[test]
fn an_empty_page_ends_even_when_it_claims_more() {
    let (mut side, _) = opened();
    answer(&mut side, page(1, 0, true));
    assert!(!side.sheet().unwrap().more);
    assert_eq!(count(side.sheet().unwrap(), &side.forges), Count::Hidden);
    assert!(!long(side.sheet().unwrap()));
    assert!(act(&mut side, Act::NearEnd).is_empty());
}

#[test]
fn the_picker_stops_at_its_bound_and_clips_overflow() {
    for (read, final_page, expected) in [(1999, 1, 2000), (1999, 2, 2000)] {
        let (mut side, _) = opened();
        for id in 0..read {
            side.sheet.as_mut().unwrap().rows.push(reached(id, b"many", false)).unwrap();
        }
        let out = answer(&mut side, page(1, final_page, true));
        assert!(out.is_empty());
        assert_eq!(side.sheet().unwrap().rows.len(), expected);
        assert_eq!(count(side.sheet().unwrap(), &side.forges), Count::Full);
        assert!(act(&mut side, Act::NearEnd).is_empty());
    }
}

#[test]
fn choices_clear_picks_and_search_and_reject_old_answers() {
    let (mut side, _) = opened();
    answer(&mut side, page(1, 2, true));
    act(&mut side, Act::Pick { id: 1, on: true });
    act(&mut side, Act::Find(b"repo".as_slice().into()));
    let previous = reading(&side);
    act(&mut side, Act::Installation(20));
    let sheet = side.sheet().unwrap();
    assert!(sheet.rows.is_empty() && sheet.picks.is_empty() && sheet.find.text.is_empty());
    received(&mut side, previous, Ok(page(3, 1, false)), env().limits, &mut Queue::uncapped());
    assert!(side.sheet().unwrap().rows.is_empty());
    let previous = reading(&side);
    act(&mut side, Act::Forge(FORGEJO));
    received(&mut side, previous, Ok(page(3, 1, false)), env().limits, &mut Queue::uncapped());
    assert_eq!(side.sheet().unwrap().forge, Some(FORGEJO));
    assert!(side.sheet().unwrap().rows.is_empty());
}

#[test]
fn explicit_choices_survive_forge_refreshes() {
    let (mut side, _) = opened();
    if let State::Rows(rows) = &mut side.forges {
        let mut rows = rows.to_vec();
        rows.push(forge_row(9, Kind::Forgejo, Mode::Owned));
        side.forges = State::Rows(rows.into_boxed_slice());
    }
    act(&mut side, Act::Forge(FORGEJO));
    act(&mut side, Act::Target(9));
    let pending = reading(&side);
    progress(&mut side, env().limits, &mut Queue::uncapped());
    assert_eq!(side.sheet().unwrap().forge, Some(FORGEJO));
    assert_eq!(side.sheet().unwrap().target, Some(9));
    assert_eq!(reading(&side), pending);
    answer(&mut side, page(1, 1, false));
    assert_eq!(side.sheet().unwrap().rows.len(), 1);
}

#[test]
fn a_new_forge_drops_a_copy_adoption_intent() {
    let (mut side, _) = opened();
    act(&mut side, Act::Source(Source::Copy));
    answer(&mut side, page(1, 1, false));
    act(&mut side, Act::Addresses(b"acme/repo-1".as_slice().into()));
    act(&mut side, Act::AdoptIt { org: b"acme".as_slice().into(), name: b"repo-1".as_slice().into() });
    act(&mut side, Act::Forge(GITHUB));
    answer(&mut side, page(1, 1, false));
    assert!(side.sheet().unwrap().picks.is_empty());
}

#[test]
fn a_search_after_the_last_page_fills_not_listed() {
    let (mut side, _) = opened();
    answer(&mut side, page(1, 9, false));
    assert!(act(&mut side, Act::Find(b"acme/missing".as_slice().into())).is_empty());
    let sheet = side.sheet().unwrap();
    assert_eq!(sheet.not_listed.0.text.as_ref(), b"acme");
    assert_eq!(sheet.not_listed.1.text.as_ref(), b"missing");
    assert!(sheet.not_listed.0.written > 0 && sheet.not_listed.1.written > 0);
    act(&mut side, Act::Find(b"repo-1".as_slice().into()));
    assert!(side.sheet().unwrap().not_listed.1.text.is_empty());
    act(&mut side, Act::NotListed { owner: Some(b"manual".as_slice().into()), name: None });
    act(&mut side, Act::Find(b"acme/missing".as_slice().into()));
    assert_eq!(side.sheet().unwrap().not_listed.0.text.as_ref(), b"manual");
}

#[test]
fn search_preserves_hidden_picks_and_filters_owner_and_name() {
    let (mut side, _) = opened();
    answer(&mut side, page(1, 9, false));
    act(&mut side, Act::Pick { id: 1, on: true });
    act(&mut side, Act::Find(b" ACME/REPO-2 ".as_slice().into()));
    let sheet = side.sheet().unwrap();
    assert_eq!(sheet.picks.len(), 1);
    assert_eq!(shown(sheet, PROJECT, &side.forges).len(), 1);
    assert_eq!(count(sheet, &side.forges), Count::Found { found: 1, of: 9 });
    act(&mut side, Act::Find(b"missing".as_slice().into()));
    assert_eq!(count(side.sheet().unwrap(), &side.forges), Count::Hidden);
    act(&mut side, Act::Find(Box::new([])));
    assert!(shown(side.sheet().unwrap(), PROJECT, &side.forges).get(0).unwrap().picked);
}

#[test]
fn marks_disable_the_projects_and_copying_repositories() {
    let (mut side, _) = opened();
    if let State::Rows(rows) = &mut side.forges {
        let row = rows.iter_mut().find(|row| row.number == GITHUB).unwrap();
        row.repositories = Box::new([Adoption {
            project: PROJECT,
            owner: b"acme".as_slice().into(),
            name: b"repo-1".as_slice().into(),
        }]);
        row.makings = Box::new([forge::making::Making {
            org: b"acme".as_slice().into(),
            name: b"repo-2".as_slice().into(),
            from: w::MakeFrom::Empty,
            into: None,
            state: w::MakingState::Making,
        }]);
    }
    answer(&mut side, page(1, 2, false));
    side.sheet.as_mut().unwrap().rows.push(reached(3, b"private", true)).unwrap();
    let shown = shown(side.sheet().unwrap(), PROJECT, &side.forges);
    assert_eq!(
        shown.as_slice().iter().map(|shown| shown.mark).collect::<Vec<_>>(),
        [Mark::InProject, Mark::Copying, Mark::Private]
    );
    act(&mut side, Act::Pick { id: 1, on: true });
    act(&mut side, Act::Pick { id: 2, on: true });
    act(&mut side, Act::Pick { id: 3, on: true });
    assert_eq!(side.sheet().unwrap().picks.as_slice().iter().map(|pick| pick.id).collect::<Vec<_>>(), [3]);
}

#[test]
fn not_listed_remedies_follow_the_forge() {
    let mut sheet = Sheet::new(PROJECT, &rows());
    sheet.not_listed = (field(b"acme"), field(b"missing"));
    let state = rows();
    let github = row(&state, Some(GITHUB)).unwrap();
    let grant = Remedy::Grant {
        owner: b"acme".as_slice().into(),
        name: b"missing".as_slice().into(),
        page: b"https://github.com/apps/bot/installations/new".as_slice().into(),
    };
    assert_eq!(remedy(&sheet, github), Some(grant));
    sheet.not_listed.0 = field(b"other");
    assert_eq!(remedy(&sheet, github), Some(Remedy::NotInstalled { owner: b"other".as_slice().into() }));
    let connected = forge_row(FORGEJO, Kind::Forgejo, Mode::Connected);
    assert_eq!(
        remedy(&sheet, &connected),
        Some(Remedy::Collaborator {
            login: b"bot".as_slice().into(),
            owner: b"other".as_slice().into(),
            name: b"missing".as_slice().into()
        })
    );
    for mode in [Mode::Managed, Mode::Owned] {
        assert_eq!(remedy(&sheet, &forge_row(FORGEJO, Kind::Forgejo, mode)), None);
    }
}

#[test]
fn an_installed_account_gets_the_same_remedy_without_a_known_slug() {
    let mut state = rows();
    let State::Rows(rows) = &mut state else { panic!("forge rows") };
    let github = rows.iter_mut().find(|row| row.number == GITHUB).unwrap();
    github.connection = Connection::NotConnected;
    let mut sheet = Sheet::new(PROJECT, &State::Unknown);
    sheet.not_listed = (field(b"ACME"), field(b"missing"));
    let grant = remedy(&sheet, github).unwrap();
    assert_eq!(
        grant,
        Remedy::Grant { owner: b"ACME".as_slice().into(), name: b"missing".as_slice().into(), page: Box::new([]) }
    );
    assert_eq!(
        forge::words::sheet_remedy(&grant).as_ref(),
        b"Check ACME/missing exists, and grant it to temper's App on GitHub."
    );
}

#[test]
fn an_unlisted_installed_repository_waits_for_a_grant_unless_context() {
    let (mut side, _) = opened();
    answer(&mut side, page(1, 0, false));
    act(&mut side, Act::NotListed { owner: Some(b"acme".as_slice().into()), name: Some(b"missing".as_slice().into()) });
    assert!(!button(side.sheet().unwrap(), side.form().unwrap(), &side.forges).1);
    side.form.as_mut().unwrap().role = forge::Role::Context;
    assert!(button(side.sheet().unwrap(), side.form().unwrap(), &side.forges).1);
}

#[test]
fn use_it_repairs_only_its_line_and_adopt_it_picks_after_read() {
    let (mut side, _) = opened();
    act(&mut side, Act::Source(Source::Copy));
    answer(&mut side, page(1, 1, false));
    act(&mut side, Act::Addresses(b"github/acme/widget\nacme/repo-1".as_slice().into()));
    act(&mut side, Act::UseIt { line: b"github.com/acme/widget".as_slice().into() });
    assert_eq!(side.sheet().unwrap().addresses.text.as_ref(), b"github.com/acme/widget\nacme/repo-1");
    act(&mut side, Act::AdoptIt { org: b"acme".as_slice().into(), name: b"repo-1".as_slice().into() });
    let sheet = side.sheet().unwrap();
    assert_eq!(sheet.source, Source::Forge);
    assert_eq!(sheet.forge, Some(FORGEJO));
    assert!(sheet.picks.is_empty());
    answer(&mut side, page(1, 1, false));
    assert_eq!(side.sheet().unwrap().picks.get(0).unwrap().id, 1);
    assert_eq!(side.sheet().unwrap().addresses.text.as_ref(), b"github.com/acme/widget");
}

#[test]
fn plural_adoptions_send_one_at_a_time_without_home() {
    let (mut side, _) = opened();
    answer(&mut side, page(1, 2, false));
    landing(&mut side);
    act(&mut side, Act::Pick { id: 1, on: true });
    act(&mut side, Act::Pick { id: 2, on: true });
    let out = act(&mut side, Act::Send);
    assert!({
        let w::Request::Adopt(adopt) = sent(&out) else { panic!("adopt") };
        !adopt.home() && adopt.name() == b"repo-1"
    });
    assert!(act(&mut side, Act::Send).is_empty());
    let mut out = Queue::uncapped();
    let request = pending(&side);
    adopted(&mut side, request, &adopted_answer(), env().limits, &mut out);
    assert!({
        let w::Request::Adopt(adopt) = sent(&out) else { panic!("adopt") };
        !adopt.home() && adopt.name() == b"repo-2"
    });
    assert_eq!(side.sheet().unwrap().picks.len(), 1);
    let request = pending(&side);
    let mut out = Queue::uncapped();
    adopted(&mut side, request, &adopted_answer(), env().limits, &mut out);
    assert!(out.iter().any(|request| if let Request::Repositories { project } = request {
        *project == PROJECT
    } else {
        false
    }));
}

#[test]
fn an_adoption_refusal_keeps_only_unfinished_picks() {
    let (mut side, _) = opened();
    answer(&mut side, page(1, 2, false));
    landing(&mut side);
    for id in [1, 2] {
        act(&mut side, Act::Pick { id, on: true });
    }
    act(&mut side, Act::Send);
    let request = pending(&side);
    adopted(&mut side, request, &adopted_answer(), env().limits, &mut Queue::uncapped());
    let request = pending(&side);
    adopted(
        &mut side,
        request,
        &forge::Answer::ForgeRefused(forge::Reason::NoAccess),
        env().limits,
        &mut Queue::uncapped(),
    );
    assert_eq!(side.sheet().unwrap().picks.get(0).unwrap().id, 2);
    assert_eq!(side.sheet().unwrap().sent, Sent::Refused(b"temper cannot access this repository.".as_slice().into()));
    assert!(side.form().unwrap().pending.is_none());
}

#[test]
fn copies_send_in_project_scope_with_home_only_first() {
    let (mut side, _) = opened();
    act(&mut side, Act::Source(Source::Copy));
    answer(&mut side, page(1, 0, false));
    act(&mut side, Act::Addresses(b"acme/one\nacme/two".as_slice().into()));
    let out = act(&mut side, Act::Send);
    assert!(out.iter().any(|request| {
        if let Request::Send {
            ask: jig_client::Ask::Family { scope: jig_client::contract::Scope::Project(project), .. },
            ..
        } = request
        {
            *project == PROJECT
        } else {
            false
        }
    }));
    assert!({
        let w::Request::MakeRepository(make) = sent(&out) else { panic!("make") };
        w::MakeRepository::into(&make).as_ref().unwrap().home() && make.name() == b"one"
    });
    let request = pending(&side);
    let mut out = Queue::uncapped();
    made(&mut side, request, Ok(()), env().limits, &mut out);
    assert_eq!(side.sheet().unwrap().addresses.text.as_ref(), b"acme/two");
    assert!({
        let w::Request::MakeRepository(make) = sent(&out) else { panic!("make") };
        !w::MakeRepository::into(&make).as_ref().unwrap().home() && make.name() == b"two"
    });
    let request = pending(&side);
    let mut out = Queue::uncapped();
    made(&mut side, request, Ok(()), env().limits, &mut out);
    assert!(out.iter().any(|request| if let Request::Repositories { project } = request {
        *project == PROJECT
    } else {
        false
    }));
}

#[test]
fn making_refusal_keeps_its_words_and_remaining_addresses() {
    let (mut side, _) = opened();
    act(&mut side, Act::Source(Source::Copy));
    answer(&mut side, page(1, 0, false));
    act(&mut side, Act::Addresses(b"acme/one\nacme/two".as_slice().into()));
    act(&mut side, Act::Send);
    let request = pending(&side);
    made(&mut side, request, Ok(()), env().limits, &mut Queue::uncapped());
    let request = pending(&side);
    let words = b"The project already has a home.";
    made(&mut side, request, Err(words.as_slice().into()), env().limits, &mut Queue::uncapped());
    assert_eq!(side.sheet().unwrap().addresses.text.as_ref(), b"acme/two");
    assert_eq!(side.sheet().unwrap().sent, Sent::Refused(words.as_slice().into()));
}

#[test]
fn empty_making_has_the_name_and_existing_projects_get_no_home() {
    let (mut side, _) = opened();
    if let State::Rows(rows) = &mut side.forges {
        rows[0].repositories = Box::new([Adoption {
            project: PROJECT,
            owner: b"acme".as_slice().into(),
            name: b"home".as_slice().into(),
        }]);
    }
    progress(&mut side, env().limits, &mut Queue::uncapped());
    act(&mut side, Act::Source(Source::Empty));
    act(&mut side, Act::Name(b"new".as_slice().into()));
    let out = act(&mut side, Act::Send);
    assert!({
        let w::Request::MakeRepository(make) = sent(&out) else { panic!("make") };
        make.org() == b"work"
            && make.name() == b"new"
            && !w::MakeRepository::into(&make).as_ref().unwrap().home()
            && make.from() == &w::MakeFrom::Empty
    });
}

#[test]
fn making_buttons_hold_back_an_edit_past_the_fields_bound() {
    let (mut side, _) = opened();
    act(&mut side, Act::Source(Source::Empty));
    act(&mut side, Act::Name(b"new".as_slice().into()));
    assert!(button(side.sheet().unwrap(), side.form().unwrap(), &side.forges).1);
    let too_long = b"x".repeat(usize::try_from(env().limits.field_bytes).unwrap() + 1);
    for edit in [Act::Org(too_long.clone().into()), Act::Name(too_long.clone().into())] {
        act(&mut side, edit);
        assert!(!button(side.sheet().unwrap(), side.form().unwrap(), &side.forges).1);
        assert!(act(&mut side, Act::Send).is_empty());
        act(&mut side, Act::Org(b"work".as_slice().into()));
        act(&mut side, Act::Name(b"new".as_slice().into()));
    }
    act(&mut side, Act::Source(Source::Copy));
    answer(&mut side, page(1, 0, false));
    act(&mut side, Act::Addresses(b"acme/one".as_slice().into()));
    assert!(button(side.sheet().unwrap(), side.form().unwrap(), &side.forges).1);
    act(&mut side, Act::Addresses(too_long.into()));
    assert_eq!(side.sheet().unwrap().addresses.text.as_ref(), b"acme/one");
    assert!(!button(side.sheet().unwrap(), side.form().unwrap(), &side.forges).1);
    assert!(act(&mut side, Act::Send).is_empty());
    act(&mut side, Act::Addresses(b"github/acme/one".as_slice().into()));
    assert_eq!(
        button(side.sheet().unwrap(), side.form().unwrap(), &side.forges),
        (Button::Copy { count: 1, first: b"github/acme/one".as_slice().into() }, false)
    );
}

fn failed_making(from: Option<&[u8]>, reason: w::MakeFailure) -> super::super::making::Making {
    super::super::making::Making {
        org: b"acme".as_slice().into(),
        name: b"again".as_slice().into(),
        from: match from {
            Some(address) => {
                w::MakeFrom::Copy(w::MakeCopy::new(&w::CEILINGS, w::MakeCopyParts { address: address.into() }).unwrap())
            }
            None => w::MakeFrom::Empty,
        },
        into: Some(w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: PROJECT, home: false }).unwrap()),
        state: w::MakingState::Failed(
            w::MakingFailed::new(
                &w::CEILINGS,
                w::MakingFailedParts {
                    step: w::MakeStep::Repository,
                    reason,
                    words: b"the forge refused".as_slice().into(),
                },
            )
            .unwrap(),
        ),
    }
}
fn with_failed(from: Option<&[u8]>, reason: w::MakeFailure) -> Side {
    let (mut side, _) = opened();
    let mut row = forge_row(FORGEJO, Kind::Forgejo, Mode::Managed);
    row.makings = Box::new([failed_making(from, reason)]);
    side.forges = State::Rows(Box::new([row]));
    forge::step(&mut side, &env(), forge::Event::Left, &mut Queue::uncapped());
    side
}
fn reopen(side: &mut Side, again: Act) {
    let out = act(side, again);
    assert!(out.iter().any(|request| match request {
        Request::Show { address } => *address == forge::Address::Adopt { project: PROJECT },
        Request::Repositories { .. }
        | Request::PlaygroundReset
        | Request::PanelEnded { .. }
        | Request::ConnectForge { .. }
        | Request::Leave { .. }
        | Request::Client { .. }
        | Request::Send { .. } => false,
    }));
    forge::step(
        side,
        &env(),
        forge::Event::Went { address: forge::Address::Adopt { project: PROJECT } },
        &mut Queue::uncapped(),
    );
}
#[test]
fn copy_again_opens_its_forge_and_fills_the_address() {
    let mut side = with_failed(Some(b"https://source.example/acme/again"), w::MakeFailure::Other);
    reopen(
        &mut side,
        Act::CopyAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let sheet = side.sheet().unwrap();
    assert_eq!((sheet.source, sheet.target), (Source::Copy, Some(FORGEJO)));
    assert_eq!(sheet.addresses.text.as_ref(), b"https://source.example/acme/again");
    assert!(button(sheet, side.form().unwrap(), &side.forges).1, "a failed making does not occupy its retry name");
}
#[test]
fn copy_again_reuses_the_failed_copys_listed_name() {
    let mut side = with_failed(Some(b"https://source.example/acme/again"), w::MakeFailure::Other);
    reopen(
        &mut side,
        Act::CopyAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let mut rows = List::with_capacity(1);
    rows.push(reached(11, b"again", false)).unwrap();
    answer(&mut side, w::Page::new(&w::CEILINGS, w::PageParts { rows, more: false }).unwrap());
    let sheet = side.sheet().unwrap();
    assert_eq!(
        lines(sheet, &side.forges).as_slice(),
        &[forge::making::Line::To {
            address: b"https://source.example/acme/again".as_slice().into(),
            org: b"acme".as_slice().into(),
            name: b"again".as_slice().into(),
        }]
    );
    assert!(button(sheet, side.form().unwrap(), &side.forges).1);
}
#[test]
fn copy_again_does_not_free_a_taken_name() {
    let mut side = with_failed(Some(b"https://source.example/acme/again"), w::MakeFailure::Taken);
    reopen(
        &mut side,
        Act::CopyAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let mut rows = List::with_capacity(1);
    rows.push(reached(11, b"again", false)).unwrap();
    answer(&mut side, w::Page::new(&w::CEILINGS, w::PageParts { rows, more: false }).unwrap());
    let sheet = side.sheet().unwrap();
    assert_eq!(
        lines(sheet, &side.forges).as_slice(),
        &[forge::making::Line::Taken {
            raw: b"https://source.example/acme/again".as_slice().into(),
            org: b"acme".as_slice().into(),
            name: b"again".as_slice().into(),
        }]
    );
    assert!(!button(sheet, side.form().unwrap(), &side.forges).1);
}
#[test]
fn copy_again_does_not_free_another_sources_name() {
    let mut side = with_failed(Some(b"https://source.example/acme/again"), w::MakeFailure::Other);
    reopen(
        &mut side,
        Act::CopyAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let mut rows = List::with_capacity(1);
    rows.push(reached(11, b"again", false)).unwrap();
    answer(&mut side, w::Page::new(&w::CEILINGS, w::PageParts { rows, more: false }).unwrap());
    act(&mut side, Act::Addresses(b"https://other.example/acme/again".as_slice().into()));
    let sheet = side.sheet().unwrap();
    assert_eq!(
        lines(sheet, &side.forges).as_slice(),
        &[forge::making::Line::Taken {
            raw: b"https://other.example/acme/again".as_slice().into(),
            org: b"acme".as_slice().into(),
            name: b"again".as_slice().into(),
        }]
    );
    assert!(!button(sheet, side.form().unwrap(), &side.forges).1);
}
#[test]
fn copy_again_compares_normalised_sources() {
    let mut side = with_failed(Some(b"source.example/acme/again.git"), w::MakeFailure::Other);
    reopen(
        &mut side,
        Act::CopyAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let mut rows = List::with_capacity(1);
    rows.push(reached(11, b"again", false)).unwrap();
    answer(&mut side, w::Page::new(&w::CEILINGS, w::PageParts { rows, more: false }).unwrap());
    act(&mut side, Act::Addresses(b"https://source.example/acme/again".as_slice().into()));
    let sheet = side.sheet().unwrap();
    assert_eq!(
        lines(sheet, &side.forges).as_slice(),
        &[forge::making::Line::To {
            address: b"https://source.example/acme/again".as_slice().into(),
            org: b"acme".as_slice().into(),
            name: b"again".as_slice().into(),
        }]
    );
    assert!(button(sheet, side.form().unwrap(), &side.forges).1);
}
#[test]
fn copy_again_does_not_free_an_adopted_name() {
    let mut side = with_failed(Some(b"https://source.example/acme/again"), w::MakeFailure::Other);
    if let State::Rows(rows) = &mut side.forges {
        rows[0].repositories = Box::new([Adoption {
            project: PROJECT,
            owner: b"acme".as_slice().into(),
            name: b"again".as_slice().into(),
        }]);
    }
    reopen(
        &mut side,
        Act::CopyAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let sheet = side.sheet().unwrap();
    assert_eq!(
        lines(sheet, &side.forges).as_slice(),
        &[forge::making::Line::Taken {
            raw: b"https://source.example/acme/again".as_slice().into(),
            org: b"acme".as_slice().into(),
            name: b"again".as_slice().into(),
        }]
    );
    assert!(!button(sheet, side.form().unwrap(), &side.forges).1);
}
#[test]
fn copy_again_does_not_free_an_in_flight_name() {
    let made =
        w::MakingMade::new(&w::CEILINGS, w::MakingMadeParts { id: 11, branch: b"main".as_slice().into() }).unwrap();
    for state in [
        w::MakingState::Waiting,
        w::MakingState::Making,
        w::MakingState::Made(made.clone()),
        w::MakingState::Adopting(made),
    ] {
        let mut side = with_failed(Some(b"https://source.example/acme/again"), w::MakeFailure::Other);
        reopen(
            &mut side,
            Act::CopyAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
        );
        if let State::Rows(rows) = &mut side.forges {
            rows[0].makings[0].state = state;
        }
        let sheet = side.sheet().unwrap();
        assert_eq!(
            lines(sheet, &side.forges).as_slice(),
            &[forge::making::Line::Taken {
                raw: b"https://source.example/acme/again".as_slice().into(),
                org: b"acme".as_slice().into(),
                name: b"again".as_slice().into(),
            }]
        );
        assert!(!button(sheet, side.form().unwrap(), &side.forges).1);
    }
}
#[test]
fn make_again_opens_its_forge_and_fills_both_names() {
    let mut side = with_failed(None, w::MakeFailure::Other);
    reopen(
        &mut side,
        Act::MakeAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let sheet = side.sheet().unwrap();
    assert_eq!((sheet.source, sheet.target), (Source::Empty, Some(FORGEJO)));
    assert_eq!((sheet.org.text.as_ref(), sheet.name.text.as_ref()), (b"acme".as_slice(), b"again".as_slice()));
}
#[test]
fn adopt_again_picks_the_taken_name_after_reading() {
    let mut side = with_failed(Some(b"https://source.example/acme/again"), w::MakeFailure::Taken);
    reopen(
        &mut side,
        Act::AdoptAgain { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() },
    );
    let mut rows = List::with_capacity(1);
    rows.push(reached(11, b"again", false)).unwrap();
    answer(&mut side, w::Page::new(&w::CEILINGS, w::PageParts { rows, more: false }).unwrap());
    let sheet = side.sheet().unwrap();
    assert_eq!((sheet.source, sheet.forge), (Source::Forge, Some(FORGEJO)));
    assert_eq!(
        sheet.picks.as_slice(),
        &[Pick { id: 11, owner_id: 3, owner: b"acme".as_slice().into(), name: b"again".as_slice().into() }]
    );
    assert_eq!(shown(sheet, PROJECT, &side.forges).get(0).unwrap().mark, Mark::None);
    act(&mut side, Act::Pick { id: 11, on: false });
    assert!(side.sheet().unwrap().picks.is_empty());
}
#[test]
fn dismiss_uses_the_rows_project_and_waits_for_the_watch() {
    let mut side = with_failed(None, w::MakeFailure::Other);
    let dismiss = Act::Dismiss { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() };
    let out = act(&mut side, dismiss.clone());
    let w::Request::DismissMaking(request) = sent(&out) else { panic!("dismiss making") };
    assert_eq!((request.forge(), request.org(), request.name()), (FORGEJO, b"acme".as_slice(), b"again".as_slice()));
    assert_eq!(request.into_parts().into, Some(PROJECT));
    let (key, scope) = out
        .iter()
        .find_map(|request| match request {
            Request::Send { request, ask: jig_client::Ask::Family { scope, .. }, .. } => Some((*request, *scope)),
            Request::Repositories { .. }
            | Request::PlaygroundReset
            | Request::PanelEnded { .. }
            | Request::Show { .. }
            | Request::ConnectForge { .. }
            | Request::Leave { .. }
            | Request::Client { .. }
            | Request::Send { .. } => None,
        })
        .unwrap();
    assert_eq!(scope, jig_client::contract::Scope::Project(PROJECT));
    assert!(act(&mut side, dismiss).is_empty(), "one dismissal in flight per row");
    dismissal_answer(&mut side, key, w::Outcome::MakeKept);
    assert!(making(&side, FORGEJO, b"acme", b"again").is_some(), "only the watch removes a row");
    assert_eq!(dismissal(&side, FORGEJO, b"acme", b"again").unwrap().pending, None);
    side.forges = State::Rows(Box::new([]));
    progress(&mut side, env().limits, &mut Queue::uncapped());
    assert!(side.sheet_dismissals.is_empty());
}
#[test]
fn a_dismissal_refusal_stays_with_its_row_after_leaving() {
    let mut side = with_failed(None, w::MakeFailure::Other);
    act(&mut side, Act::Dismiss { forge: FORGEJO, org: b"acme".as_slice().into(), name: b"again".as_slice().into() });
    let token = dismissal(&side, FORGEJO, b"acme", b"again").unwrap().pending.unwrap();
    let refused = w::MakeRefused::new(&w::CEILINGS, w::MakeRefusedParts { why: w::MakeRefusal::NotOwner }).unwrap();
    dismissal_answer(&mut side, token, w::Outcome::MakeRefused(refused));
    assert_eq!(
        dismissal(&side, FORGEJO, b"acme", b"again").unwrap().words.as_deref(),
        Some(b"Only the deployment's owner makes repositories.".as_slice())
    );
}

fn dismissal_answer(side: &mut Side, token: Token, outcome: w::Outcome) {
    let outcome = jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
    }));
    let client = jig_client::Client::new(&fixtures::limits().client, fixtures::SEED).unwrap();
    told(
        side,
        &client,
        &jig_client::Told::Answered {
            key: jig_client::Key { asker: jig_client::Asker(1), number: token.raw() },
            outcome,
        },
        env().limits,
        &mut Queue::uncapped(),
    );
}
