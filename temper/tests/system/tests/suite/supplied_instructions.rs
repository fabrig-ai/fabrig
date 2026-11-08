//! Archived openings retain delivered FIRST instructions and fitted document briefs.
//! Contract: smith/docs/design/domain/run.md, 3.3; temper/docs/help/reference/ctl-task.md.
use jig_domain_core::CharterPart;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use smith_domain_session::record;
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};
use temper_system_world::{composed::report_script, world};

const BASE: &[u8] = b"@system base";
const FIRST: &[u8] = b"FIRST-selected";
const UNSELECTED: &[u8] = b"QUESTION-only";
const DOCUMENT: &[u8] =
    b"commit 0123456789abcdef0123456789abcdef01234567\nCharter document section supplied to this run.";

#[test]
fn goal_and_producer_openings_keep_their_selected_instructions_and_document_commits() {
    for words in [b"Goal 1 brief".as_slice(), b"Producer 5 brief", b"Producer 6 brief"] {
        let mut root = configured();
        let mut ask = root.chat_ask(1, words);
        let people::Ask::Create { batch, .. } = &mut ask else { panic!("create task") };
        batch[0].flags.conversation = false;
        root.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(41)),
            sign_in: root.session(),
            key: [41; 16],
            ask,
        }));
        root.settle();
        let mut assignment = root.last_assignment();
        assert!(root.task(assignment.task).expect("started task").reasons.contains(tasks::Reasons::FIRST));
        assert!(assignment.run.policy.instructions.starts_with(BASE));
        assert!(skein_lib::bytes::find(&assignment.run.policy.instructions, FIRST).is_some());
        let mut sections = assignment.sections.to_vec();
        sections.push(engine::BriefSection {
            kind: engine::BriefKind::Forge(engine::ForgeBriefKind::Document(b"rules.md, Charter".as_slice().into())),
            body: engine::BriefBody::Text(DOCUMENT.into()),
        });
        assignment.sections = sections.into();
        archived(&mut root, &assignment, words);
    }
}

fn configured() -> Driver {
    let mut configs = [world::config(440), world::config(440)];
    for config in &mut configs {
        let charter = &mut config.core.defaults.charters[0];
        charter.policy.resume = false;
        charter.policy.instructions = BASE.into();
        charter.parts = Box::new([
            CharterPart { on: tasks::Reasons::FIRST, instructions: FIRST.into() },
            CharterPart { on: tasks::Reasons::QUESTION, instructions: UNSELECTED.into() },
        ]);
    }
    let mut root = Driver::configured(Store::new(), configs, &walking::limits());
    root.settle();
    root.send(world::hello(1));
    root.settle();
    root.sign_in();
    root.settle();
    root
}

fn archived(root: &mut Driver, assignment: &engine::Assignment, words: &[u8]) {
    let mut agent = world::scripted_agent_for(assignment, report_script(false));
    world::run_assignment(root, assignment, &mut agent);
    let prompt = agent.prompts().first().expect("one deterministic response");
    let turn = agent.turns().first().expect("archived first turn");
    let record::Kind::First(opener) = &turn.kind else { panic!("first archived turn") };
    let supplied = opener.system.as_deref().expect("retained supplied system text");
    assert_eq!(supplied, prompt.system.as_ref());
    let archived = root
        .store
        .rows
        .values()
        .find_map(|row| match row {
            engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn)))
                if turn.task == assignment.task =>
            {
                Some(turn)
            }
            _ => None,
        })
        .expect("committed archived turn");
    let decoded = smith_transcript::decode_archived(&smith_transcript::CEILINGS, &archived.transcript)
        .expect("archived bytes decode");
    assert_eq!(decoded.system().as_deref(), Some(supplied));
    for expected in [BASE, FIRST, words, DOCUMENT] {
        assert!(skein_lib::bytes::find(supplied, expected).is_some(), "missing supplied section {expected:?}");
    }
    assert!(skein_lib::bytes::find(supplied, UNSELECTED).is_none());
    assert_eq!(agent.prompts().len(), 1);
}
