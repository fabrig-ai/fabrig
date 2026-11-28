//! The set enters whole and is kept by the project and its live agents
//! (jig/docs/design/domain/engine/project-charters.md, 2–4 and 6).
use super::*;

pub(super) mod binding;

const PROJECT: u32 = 1;
pub(super) const NAME: &[u8] = b"implementer";

pub(super) fn set(revision: u64) -> ProjectCharters {
    ProjectCharters {
        project: PROJECT,
        revision,
        source: Box::from(b"application source".as_slice()),
        charters: Box::new([ProjectCharter {
            name: Box::from(NAME),
            settings: Box::from(b"not-yet-offered".as_slice()),
            instructions: Box::from(b"Be careful.".as_slice()),
            parts: Box::new([part(tasks::Reasons::FIRST, b"First words")]),
        }]),
    }
}

fn part(on: tasks::Reasons, words: &[u8]) -> CharterPart {
    CharterPart { on, instructions: Box::from(words) }
}

pub(super) fn kept_key(revision: u64) -> Key {
    Key::Core(CoreKey::Settings(crate::settings::Key::ProjectCharters { project: PROJECT, revision }))
}

fn send(core: &mut Core, limits: &Limits, set: ProjectCharters) -> Queue<Request> {
    step(core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }, Event::ProjectCharters(set))
}

fn saved(out: &Queue<Request>, set: &ProjectCharters) -> bool {
    for request in out {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(
            crate::settings::Record::ProjectCharters(found),
        )))) = request
            && found == set
        {
            return true;
        }
    }
    false
}

pub(super) fn erased(out: &Queue<Request>, revision: u64) -> bool {
    for request in out {
        if let Request::Write(Write::Erase(key)) = request
            && *key == kept_key(revision)
        {
            return true;
        }
    }
    false
}

#[test]
fn project_charter_sets_are_taken_or_refused_whole() {
    use ProjectChartersFault as Fault;
    let (mut core, mut limits) = core();
    limits.tasks.charters = 1;
    limits.tasks.charter_parts = 1;
    assert_eq!(core.project_charters(PROJECT), 0);
    assert!(saved(&send(&mut core, &limits, set(1)), &set(1)));
    assert_eq!(core.project_charters(PROJECT), 1);
    let mut unknown = set(2);
    unknown.project = 99;
    let mut source = set(2);
    source.source = skein_lib::bytes::zeroed(257);
    let mut count = set(2);
    count.charters = Box::new([count.charters[0].clone(), count.charters[0].clone()]);
    let mut parts = set(2);
    parts.charters[0].parts = Box::new([part(tasks::Reasons::FIRST, b"1"), part(tasks::Reasons::FIRST, b"2")]);
    let mut reasons = set(2);
    reasons.charters[0].parts = Box::new([part(tasks::Reasons(0), b"no reason")]);
    let mut bytes = set(2);
    bytes.charters[0].instructions = skein_lib::bytes::zeroed(skein_lib::usize_of(limits.run_bytes));
    for (input, fault) in [
        (set(0), Fault::Stale),
        (set(1), Fault::Stale),
        (unknown, Fault::Unknown),
        (source, Fault::Source { bound: 256 }),
        (count, Fault::Count { charter: NAME.into(), bound: 1 }),
        (parts, Fault::Parts { charter: NAME.into(), bound: 1 }),
        (reasons, Fault::Reasons { charter: NAME.into() }),
        (bytes, Fault::Bytes { charter: NAME.into(), bound: limits.run_bytes }),
    ] {
        let expected = (input.project, input.revision, fault);
        let mut out = send(&mut core, &limits, input);
        let Some(Request::Now(now)) = out.pop() else { panic!("expected refusal for {expected:?}: {out:?}") };
        let Now::ProjectChartersRefused { project, revision, fault } = *now else { panic!("refusal") };
        assert_eq!((project, revision, fault), expected);
        assert!(out.is_empty(), "a refusal writes nothing");
        assert_eq!(core.project_charters(PROJECT), 1);
        assert_eq!(core.project_charter_set(PROJECT, 1), Some(&set(1)));
    }
}

#[test]
fn only_the_last_live_agent_releases_a_superseded_set() {
    let (mut core, limits) = tree_core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    assert!(saved(&send(&mut core, &limits, set(1)), &set(1)));
    for number in [10, 11] {
        make_task(&mut core, &limits, number);
        assert_eq!(core.tasks.task(number).expect("live agent").charters, 1);
    }
    let replaced = send(&mut core, &limits, set(2));
    assert!(saved(&replaced, &set(2)));
    assert!(!erased(&replaced, 1));
    assert_eq!(core.project_charter_set(PROJECT, 1), Some(&set(1)));
    for number in [10, 11] {
        let cancelled = step(
            &mut core,
            &env,
            Event::Tasks(tasks::Event::Cancel {
                reply_to: ReplyTo::new(Token::new(number)),
                by: tasks::Party::Person(1),
                operation: tasks::Cancel { task: number, reason: Box::new([]), grace: None, message: None },
            }),
        );
        let settled = step(&mut core, &env, Event::Tasks(tasks::Event::EffectsSettled { task: number }));
        let ended = step(&mut core, &env, Event::Tasks(tasks::Event::Settled { task: number }));
        assert!(core.tasks.task(number).is_none(), "task ended");
        let erased = erased(&cancelled, 1) || erased(&settled, 1) || erased(&ended, 1);
        assert_eq!(erased, number == 11);
        if number == 10 {
            assert_eq!(core.project_charter_set(PROJECT, 1), Some(&set(1)));
        }
    }
    assert_eq!(core.project_charter_set(PROJECT, 1), None);
    let replaced = send(&mut core, &limits, set(3));
    assert!(saved(&replaced, &set(3)));
    assert!(erased(&replaced, 2), "an unheld newest is replaced in the same decision");
    assert_eq!(core.project_charter_set(PROJECT, 2), None);
}
