use jig_domain_people::{
    Act, ActKey, Ask, Domain, Entry, EntryKind, Event, Holding, Key, Outcome, Refusal, Reply, ReplyRef, Request,
    RequestKey, ResultRef, Role, Stored, Update, Whom, step,
};
use jig_people_world::{LIMITS, identity};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};

struct People {
    domain: Domain,
    env: Env<jig_domain_people::Limits>,
    out: Queue<Request>,
}

impl People {
    fn loading() -> Self {
        let env = Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH };
        let mut people =
            Self { domain: Domain::new(&env.limits, Box::new([]), 1, Box::new([])), env, out: Queue::uncapped() };
        for party in [1, 2] {
            people.restore(Stored::Person { number: party, identity: identity(party) });
            people.restore(Stored::SignIn {
                number: party,
                person: party,
                expires: Wall::from_nanos(Duration::from_secs(3600).as_nanos()),
            });
        }
        people
    }

    fn restore(&mut self, record: Stored) {
        self.send(Event::Restore { record });
        assert!(self.out.is_empty(), "valid retained row");
    }

    fn send(&mut self, event: Event) {
        step(&mut self.domain, &self.env, event, &mut self.out);
    }

    fn roster(&mut self, project: u32) {
        self.restore(Stored::Roles {
            project,
            holdings: Box::new([Holding { person: 1, role: Role::Member }, Holding { person: 2, role: Role::Member }]),
        });
    }

    fn open(&mut self) {
        self.send(Event::Restored);
        assert!(self.domain.ready());
        assert!(self.out.is_empty());
    }

    fn ask(&mut self, key: u8, ask: Ask, expected: Outcome) {
        let to = Token::new(1);
        self.send(Event::Ask { reply_to: ReplyTo::new(to), sign_in: 1, key: [key; 16], ask, models: Box::new([]) });
        assert_eq!(self.out.pop(), Some(Request::Reply { to: ReplyTo::new(to), reply: Reply::Outcome(expected) }));
        assert!(self.out.is_empty(), "replay neither routes nor changes stored answers");
    }

    fn act(&mut self, key: u8, act: Act, expected: Outcome) {
        let to = Token::new(1);
        self.send(Event::Act { reply_to: ReplyTo::new(to), connector: 1, key: [key; 16], act: Box::new(act) });
        assert_eq!(self.out.pop(), Some(Request::Acted { to: ReplyTo::new(to), outcome: expected }));
        assert!(self.out.is_empty(), "replay neither routes nor changes stored answers");
    }

    fn answer(&mut self, key: u8, ask: Ask, outcome: Outcome) {
        self.restore(Stored::Answer {
            key: RequestKey { person: 1, key: [key; 16] },
            ask: Box::new(ask),
            models: Box::new([]),
            outcome,
            at: Wall::EPOCH,
        });
    }

    fn acted(&mut self, key: u8, act: Act, outcome: Outcome) {
        self.restore(Stored::Acted { key: ActKey { connector: 1, key: [key; 16] }, act, outcome, at: Wall::EPOCH });
    }
}

fn entry(task: u64, project: u32, whom: Whom, kind: EntryKind) -> Entry {
    Entry { task, project, whom, kind, at: Wall::from_nanos(task) }
}

fn result(task: u64, project: u32) -> ResultRef {
    ResultRef { task, project, position: task, at: Wall::from_nanos(task), forgettable: None }
}

fn update_task(update: Update) -> u64 {
    match update {
        Update::Result(row) => row.task,
        Update::Reply(row) => row.task,
        Update::ChatReply(reply) => reply.entry.task,
    }
}

#[test]
fn forgetting_a_task_drops_every_partys_read_and_its_chat() {
    let mut people = People::loading();
    people.roster(3);
    for task in [9, 10] {
        for party in [1, 2] {
            people.restore(Stored::ReadConversation { party, task, through: 1 });
        }
    }
    people.open();
    for task in [9, 10] {
        let chat = entry(task, 3, Whom::Person(1), EntryKind::ChatReply { message: 3 });
        people.send(Event::ChatState {
            party: 1,
            project: 3,
            task,
            last_reply: 3,
            live: true,
            seen: None,
            replies: Box::new([ReplyRef { entry: chat, activity: task }]),
        });
        people.send(Event::Waiting {
            task,
            entries: Box::new([
                entry(task, 3, Whom::Role { project: 3, role: Role::Member.number() }, EntryKind::PersonTask),
                chat,
                entry(task, 3, Whom::Person(2), EntryKind::Reply { message: task }),
                entry(task, 3, Whom::Person(2), EntryKind::Question { message: task }),
            ]),
        });
        for party in [1, 2] {
            people.domain.remember_result(&people.env.limits, party, result(task, 3));
        }
    }
    assert!(people.out.is_empty());
    assert_eq!(people.domain.chat_owner(9), Some(1));
    let unchanged_chat = people.domain.chat_unread(&people.env.limits, 1, 10);
    for party in [1, 2] {
        assert!(people.domain.task_needs(party, 9));
        assert!(people.domain.cached_results(party).expect("party").contains(&result(9, 3)));
        assert!(
            people
                .domain
                .cached_updates(&people.env.limits, party)
                .expect("party")
                .iter()
                .any(|u| update_task(*u) == 9)
        );
    }
    people.send(Event::ForgetTask { task: 9 });
    let erased: Vec<_> = std::iter::from_fn(|| people.out.pop()).collect();
    assert_eq!(erased, [1, 2].map(|party| Request::Erase { key: Key::ReadConversation { party, task: 9 } }));
    assert_eq!(people.domain.chat_owner(9), None);
    assert_eq!(people.domain.chat_unread(&people.env.limits, 1, 9).expect("party").unread, 0);
    assert_eq!(people.domain.chat_owner(10), Some(1));
    assert_eq!(people.domain.chat_unread(&people.env.limits, 1, 10), unchanged_chat);
    for party in [1, 2] {
        assert!(!people.domain.read_conversations().contains_key(&(party, 9)));
        assert_eq!(people.domain.conversation_seen(party, 10), Some(1));
        assert!(!people.domain.task_needs(party, 9));
        assert!(people.domain.task_needs(party, 10));
        assert_eq!(people.domain.cached_results(party), Some([result(10, 3)].as_slice()));
        assert!(people.domain.cached_waiting(&people.env.limits, party).expect("party").iter().all(|e| e.task == 10));
        assert!(
            people
                .domain
                .cached_updates(&people.env.limits, party)
                .expect("party")
                .iter()
                .all(|u| update_task(*u) == 10)
        );
        let counts = people.domain.counts(&people.env.limits, party).expect("party");
        assert_eq!(counts.unread, if party == 1 { 3 } else { 2 }, "chat/notice plus result census");
    }
    // A task without a roster still loses every party's read position.
    let mut unrostered = People::loading();
    for party in [1, 2] {
        unrostered.restore(Stored::ReadConversation { party, task: 9, through: 1 });
    }
    unrostered.open();
    unrostered.send(Event::ForgetTask { task: 9 });
    for party in [1, 2] {
        assert_eq!(unrostered.out.pop(), Some(Request::Erase { key: Key::ReadConversation { party, task: 9 } }));
    }
    assert!(unrostered.out.is_empty());
}

#[test]
fn dropping_a_project_drops_its_roster() {
    let mut people = People::loading();
    for project in [3, 4] {
        people.roster(project);
    }
    people.open();
    for (project, task) in [(3, 9), (4, 10)] {
        people.send(Event::Waiting {
            task,
            entries: Box::new([
                entry(task, project, Whom::Role { project, role: Role::Member.number() }, EntryKind::PersonTask),
                entry(task, project, Whom::Person(1), EntryKind::Reply { message: task }),
            ]),
        });
        people.domain.remember_result(&people.env.limits, 1, result(task, project));
        people.send(Event::ChatState {
            party: 1,
            project,
            task,
            last_reply: 3,
            live: true,
            seen: None,
            replies: Box::new([]),
        });
    }
    let roster = people.domain.project_members(4).expect("live project").to_vec();
    people.send(Event::DropProject { project: 3 });
    assert_eq!(people.out.pop(), Some(Request::Erase { key: Key::Roles(3) }));
    assert!(people.out.is_empty());
    assert!(!people.domain.has_project(3));
    assert_eq!(people.domain.project_members(4), Some(roster.as_slice()));
    assert_eq!(people.domain.chat_owner(9), None);
    assert_eq!(people.domain.chat_owner(10), Some(1));
    assert_eq!(people.domain.cached_results(1), Some([result(10, 4)].as_slice()));
    let counts = people.domain.counts(&people.env.limits, 1).expect("party");
    assert_eq!(counts.unread, people.env.limits.inbox_entries);
    assert!(counts.more_unread);
    for party in [1, 2] {
        assert_eq!(people.domain.needs(party), Some(1));
        assert!(people.domain.cached_waiting(&people.env.limits, party).expect("party").iter().all(|e| e.project == 4));
        assert!(
            people
                .domain
                .cached_updates(&people.env.limits, party)
                .expect("party")
                .iter()
                .all(|u| update_task(*u) == 10)
        );
    }
}

#[test]
fn a_retained_answer_naming_a_gone_project_answers_unknown() {
    let mut people = People::loading();
    for project in [3, 4] {
        people.roster(project);
        let key = u8::try_from(project).expect("small project");
        people.answer(key, Ask::Rename { project, name: "renamed".into() }, Outcome::Renamed { project });
        people.acted(
            key,
            Act::MakeProject { maker: 1, name: "made".into(), policy: 1 },
            Outcome::ProjectMade { project },
        );
    }
    people.open();
    people.send(Event::DropProject { project: 3 });
    assert_eq!(people.out.pop(), Some(Request::Erase { key: Key::Roles(3) }));
    for _ in 0..2 {
        for project in [3, 4] {
            let key = u8::try_from(project).expect("small project");
            people.ask(
                key,
                Ask::Rename { project, name: "renamed".into() },
                if project == 3 { Outcome::Refused(Refusal::Unknown) } else { Outcome::Renamed { project } },
            );
            people.act(
                key,
                Act::MakeProject { maker: 1, name: "made".into(), policy: 1 },
                if project == 3 { Outcome::Refused(Refusal::Unknown) } else { Outcome::ProjectMade { project } },
            );
        }
    }
    // The retained answers still own their keys after an unknown replay.
    people.send(Event::Ask {
        reply_to: ReplyTo::new(Token::new(1)),
        sign_in: 1,
        key: [3; 16],
        ask: Ask::Rename { project: 3, name: "different".into() },
        models: Box::new([]),
    });
    assert_eq!(
        people.out.pop(),
        Some(Request::Reply { to: ReplyTo::new(Token::new(1)), reply: Reply::Refused(Refusal::KeyConflict) })
    );
    assert!(people.out.is_empty());
    people.act(3, Act::RemoveProject { project: 3 }, Outcome::Refused(Refusal::KeyConflict));
}

#[test]
fn a_retained_project_removed_replays_as_itself() {
    let mut people = People::loading();
    people.roster(3);
    people.answer(3, Ask::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
    people.acted(3, Act::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
    people.open();
    people.send(Event::DropProject { project: 3 });
    assert_eq!(people.out.pop(), Some(Request::Erase { key: Key::Roles(3) }));
    people.ask(3, Ask::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
    people.act(3, Act::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
    let mut restarted = People::loading();
    restarted.answer(3, Ask::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
    restarted.acted(3, Act::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
    restarted.open();
    restarted.ask(3, Ask::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
    restarted.act(3, Act::RemoveProject { project: 3 }, Outcome::ProjectRemoved { project: 3 });
}

#[test]
fn a_start_with_an_answer_naming_a_gone_roster_opens() {
    for explicit in [false, true] {
        let env = Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH };
        let mut people =
            People { domain: Domain::new(&env.limits, Box::new([]), 1, Box::new([])), env, out: Queue::uncapped() };
        if explicit {
            people.domain.start_restore(people.env.wall);
        }
        people.restore(Stored::Person { number: 1, identity: identity(1) });
        people.restore(Stored::SignIn {
            number: 1,
            person: 1,
            expires: Wall::from_nanos(Duration::from_secs(3600).as_nanos()),
        });
        // The historical member also needs no link once the roster is gone.
        let ask = Ask::SetRoles { project: 3, holdings: Box::new([Holding { person: 99, role: Role::Member }]) };
        people.answer(3, ask.clone(), Outcome::RolesSet { project: 3 });
        if explicit {
            assert!(people.domain.start_link(&people.env));
            people.domain.start_sweep();
            assert!(people.domain.start_complete(&people.env));
        } else {
            people.open();
        }
        assert!(people.domain.ready());
        assert!(!people.domain.has_project(3));
        people.ask(3, ask, Outcome::Refused(Refusal::Unknown));
    }
}
