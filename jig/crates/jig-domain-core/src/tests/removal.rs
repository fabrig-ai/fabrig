//! Removal's writes share the decision with its held answer (draft 5).
use crate::*;
use skein_lib::{Env, ReplyTo, Time, Token, Wall};

const PROJECT: u32 = 1;
const OWNER: u64 = 1;
#[test]
fn a_removal_saves_its_row_and_closes_the_project_in_one_decision() {
    let limits = tests::limits();
    let mut core = Core::new(tests::config(41), &limits).expect("bounded core");
    let _restart = core.restart_begin(false);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let _seed = step(&mut core, &env, Event::People(people::Event::Restored));
    let _roles = step(
        &mut core,
        &env,
        Event::People(people::Event::Roles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
        }),
    );
    let _sign_in = step(
        &mut core,
        &env,
        Event::SignIn {
            reply_to: ReplyTo::new(Token::new(10)),
            held: None,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new(7_u64.to_be_bytes()) },
                login: b"owner".as_slice().into(),
                name: b"Owner".as_slice().into(),
            },
        },
    );
    let Requests::Out(mut out) = step(
        &mut core,
        &env,
        Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(11)),
            sign_in: 1,
            key: [1; 16],
            ask: people::Ask::RemoveProject { project: PROJECT },
            models: Box::new([]),
        }),
    );
    let mut row = false;
    let mut policy = false;
    let mut roles = false;
    let mut answer = false;
    while let Some(next) = out.pop() {
        match next {
            Request::Write(Write::Save(Record::Core(CoreRecord::Removal(removal)))) => {
                assert_eq!(removal.project, PROJECT);
                assert_eq!(removal.stage, if row { RemovalStage::Erasing } else { RemovalStage::Ending });
                row = true;
            }
            Request::Write(Write::Erase(Key::People(people::Key::Policy(PROJECT)))) => policy = true,
            Request::Write(Write::Erase(Key::People(people::Key::Roles(PROJECT)))) => roles = true,
            Request::Held(held) => {
                if let Held::PeopleReply {
                    reply: people::Reply::Outcome(people::Outcome::ProjectRemoved { project: PROJECT }),
                    ..
                } = *held
                {
                    answer = true;
                }
            }
            Request::ProjectMade { .. }
            | Request::Write(_)
            | Request::Ask { .. }
            | Request::Now(_)
            | Request::Decided => {}
        }
    }
    assert!(row && policy && roles && answer);
    assert!(!core.projects.as_slice().contains(&PROJECT));
    assert!(!core.names.contains_key(&PROJECT));
    assert!(core.authority.policy(PROJECT).is_none());
    assert!(!core.people.has_project(PROJECT));
}
