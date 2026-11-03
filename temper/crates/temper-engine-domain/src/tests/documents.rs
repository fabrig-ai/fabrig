//! Charter documents bind before gathering and retain their titles through completion
//! (temper/docs/design/protocol/configuration.md, 13.7).
use super::resume_fixture;
use crate::assemble::{DOCUMENT_KINDS, brief_outputs, finish_brief_plan};
use crate::boundary::Work;
use crate::{BriefBody, BriefKind, Decision, Domain, ForgeBriefKind, Limits};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core::{Document, PlannedDocument};
use skein_lib::{Env, Id, List, Queue, Token};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;

fn root() -> (Domain, Env<Limits>) {
    let mut limits = resume_fixture::limits();
    limits.brief.sections = 8;
    limits.forge.brief_sections =
        limits.brief.briefs.checked_mul(limits.brief.sections).unwrap().checked_mul(2).unwrap();
    resume_fixture::root_with_config_and_limits(resume_fixture::config(7), &limits)
}

fn document(token: u64, heading: Option<Box<[u8]>>) -> PlannedDocument {
    PlannedDocument {
        token: Token::new(token),
        document: Document {
            connector: 0,
            name: jig_domain_authority::Name {
                segments: Box::new([
                    Box::from(b"forge".as_slice()),
                    Box::from(b"forge.example".as_slice()),
                    Box::from(b"org".as_slice()),
                    Box::from(b"rules".as_slice()),
                    Box::from(b"branch".as_slice()),
                    Box::from(b"guides".as_slice()),
                    Box::from(b"main".as_slice()),
                ]),
            },
            path: Box::from(b"docs/rules.md".as_slice()),
            heading,
        },
    }
}

fn plan(documents: &[PlannedDocument], capacity: u32) -> List<brief::Planned> {
    let mut wanted = List::with_capacity(capacity);
    for planned in documents {
        wanted
            .push(brief::Planned::Connector {
                connector: planned.document.connector,
                kind: jig_domain_core::DOCUMENT_KIND,
                token: planned.token,
                size: 0,
                limit: 1,
                priority: jig_domain_core::DOCUMENT_PRIORITY,
                required: false,
            })
            .unwrap();
    }
    wanted
}

fn take_plan(domain: &mut Domain) -> Box<[brief::Planned]> {
    while let Some(work) = domain.work.pop() {
        if let Work::Brief(brief::GatherEvent::Plan { sections, .. }) = work {
            return sections;
        }
    }
    panic!("bound brief plan")
}

fn outputs(domain: &mut Domain, env: &Env<Limits>, requests: &mut Queue<brief::GatherRequest>) {
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    brief_outputs(domain, env, &mut decision, requests);
}

#[test]
fn two_documents_are_planned_on_the_forge_and_rewritten_with_their_own_limits() {
    let (mut domain, env) = root();
    let documents = Box::new([document(91, None), document(92, Some(Box::from(b"Safety".as_slice())))]);
    let wanted = plan(&documents[..], 8);
    finish_brief_plan(&mut domain, &env, 8, None, wanted, documents);
    let mut tokens = List::with_capacity(2);
    for heading in [None, Some(Box::from(b"Safety".as_slice()))] {
        let Some(Work::Forge(forge::Event::PlanDocument { section, document })) = domain.work.pop() else {
            panic!("documents registered before the plan")
        };
        assert_eq!(
            document.as_ref(),
            &forge::BriefDocument {
                host: Box::from(b"forge.example".as_slice()),
                owner: Box::from(b"org".as_slice()),
                name: Box::from(b"rules".as_slice()),
                branch: Box::from(b"guides/main".as_slice()),
                path: Box::from(b"docs/rules.md".as_slice()),
                heading,
            }
        );
        tokens.push(section).unwrap();
    }
    let Some(Work::Brief(brief::GatherEvent::Plan { sections, .. })) = domain.work.pop() else { panic!("bound plan") };
    assert!(domain.work.is_empty());
    assert_eq!(tokens.len(), 2);
    for (index, section) in sections.iter().enumerate() {
        let brief::Planned::Connector { connector: 0, token, kind, limit: 4096, .. } = section else {
            panic!("document section")
        };
        assert_eq!(*kind, DOCUMENT_KINDS + u16::try_from(index).unwrap());
        assert_eq!(*token, *tokens.get(u32::try_from(index).unwrap()).unwrap());
        assert_ne!(*token, Token::new(91 + u64::try_from(index).unwrap()));
        let row = domain.brief_connectors.get(Id::from_token(*token)).unwrap();
        let expected: &[u8] =
            if index == 0 { b"org/rules docs/rules.md" } else { b"org/rules docs/rules.md, \"Safety\"" };
        assert_eq!(row.kind, ForgeBriefKind::Document(Box::from(expected)));
    }
    assert_eq!(domain.brief_document_titles.get(&8).unwrap().len(), 2);
}

#[test]
fn a_missing_document_section_is_titled_with_its_document() {
    let (mut domain, env) = root();
    let documents = Box::new([document(91, None), document(92, Some(Box::from(b"Safety".as_slice())))]);
    let wanted = plan(&documents[..], 8);
    finish_brief_plan(&mut domain, &env, 8, None, wanted, documents);
    let sections = take_plan(&mut domain);
    let mut requests = Queue::uncapped();
    for section in sections {
        let brief::Planned::Connector { token, .. } = section else { panic!("document") };
        requests.push(brief::GatherRequest::Drop { connector: 0, section: token });
    }
    requests.push(brief::GatherRequest::Complete {
        brief: Token::new(8),
        order: Box::new([
            brief::GatherPlaced::Missing { connector: 0, kind: 16, why: brief::GatherMissing::Failed },
            brief::GatherPlaced::Missing { connector: 0, kind: 17, why: brief::GatherMissing::Late },
        ]),
    });
    outputs(&mut domain, &env, &mut requests);
    let completed = crate::link::take_sections(&mut domain, 8).unwrap();
    for (index, section) in completed.iter().enumerate() {
        let title: &[u8] = if index == 0 { b"org/rules docs/rules.md" } else { b"org/rules docs/rules.md, \"Safety\"" };
        assert_eq!(section.kind, BriefKind::Forge(ForgeBriefKind::Document(Box::from(title))));
        assert_eq!(
            section.body,
            BriefBody::Missing(if index == 0 { brief::GatherMissing::Failed } else { brief::GatherMissing::Late })
        );
    }
    assert_eq!(crate::budget::section_bytes(&completed), 23 + 33);
    assert!(domain.brief_document_titles.is_empty());
    domain.reclaim();
    assert!(domain.brief_connectors.iter().next().is_none());
}

#[test]
fn a_name_that_is_no_forge_branch_is_missing_without_asking_the_forge() {
    let (mut domain, env) = root();
    let mut document = document(91, Some(Box::from(b"unused heading".as_slice())));
    document.document.name.segments = Box::new([Box::from(b"other".as_slice())]);
    let documents = Box::new([document]);
    let wanted = plan(&documents[..], 8);
    finish_brief_plan(&mut domain, &env, 8, None, wanted, documents);
    let Some(Work::Brief(brief::GatherEvent::Plan { sections, .. })) = domain.work.pop() else {
        panic!("no forge request for an invalid document")
    };
    assert!(domain.work.is_empty());
    let brief::Planned::Connector { token, .. } = sections[0] else { panic!("document") };
    assert_eq!(
        domain.brief_connectors.get(Id::from_token(token)).unwrap().kind,
        ForgeBriefKind::Document(Box::from(b"docs/rules.md".as_slice()))
    );
    let mut requests = Queue::uncapped();
    requests.push(brief::GatherRequest::Gather { connector: 0, section: token, budget: 4096 });
    outputs(&mut domain, &env, &mut requests);
    let Some(Work::Brief(brief::GatherEvent::Missing { brief, section })) = domain.work.pop() else {
        panic!("missing document")
    };
    assert_eq!((brief, section), (Token::new(8), token));
    assert!(domain.work.is_empty());
}

#[test]
fn abandonment_drops_every_document_row_and_its_titles() {
    let (mut domain, env) = root();
    let documents = Box::new([document(91, None), document(92, None)]);
    let wanted = plan(&documents[..], 8);
    finish_brief_plan(&mut domain, &env, 8, None, wanted, documents);
    let sections = take_plan(&mut domain);
    let mut gathering = Queue::uncapped();
    brief::gather_step(
        &mut domain.core.brief,
        &Env { now: env.now, wall: env.wall, limits: crate::limits::brief_limits(&env.limits.brief) },
        brief::GatherEvent::Plan {
            brief: Token::new(8),
            budget: env.limits.brief.brief_bytes,
            deadline: env.now.saturating_add(env.limits.brief.gather),
            sections,
        },
        &mut gathering,
    );
    assert_eq!(gathering.len(), 2);
    let requests = jig_domain_core::step(
        &mut domain.core,
        &crate::limits::environment_core(&env),
        jig_domain_core::Event::Brief(brief::GatherEvent::Abandon { brief: Token::new(8) }),
    );
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
    assert!(domain.brief_document_titles.is_empty());
    domain.reclaim();
    assert!(domain.brief_connectors.iter().next().is_none());
    assert_eq!(domain.work.len(), 2);
    while let Some(work) = domain.work.pop() {
        let Work::Forge(forge::Event::DropBrief { .. }) = work else { panic!("drop registered document") };
    }
}

fn repair() -> forge::ChangeRow {
    forge::ChangeRow {
        task: 7,
        repository: skein_forge::api::Repository { forge: 1, repository: 1 },
        branch: Box::from(b"temper/7".as_slice()),
        base: Box::from(b"main".as_slice()),
        title: Box::from(b"Change".as_slice()),
        body: Box::new([]),
        priority: 0,
        ci: None,
        change: change::Change {
            early: change::Early::None,
            way: change::Way::Queue,
            landing: None,
            task: 7,
            state: change::State::Producing { requested: false },
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released: None,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: skein_lib::Wall::EPOCH,
            ready_since: None,
            owns_turn: false,
            last_head: Some([1; 32]),
        },
        pull: Some(7),
        pending: None,
        effect: change::EffectResult::None,
        delegate: Some((8, change::Delegate::Repair(change::Repair::Ci))),
        delegate_status: change::Status::Unknown,
        verdicts: Box::new([]),
        result: forge::ChangeResult::empty(),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 2,
        review_charter: 4,
        effect_account: None,
        producer_failure: None,
        report: Box::new([]),
        catch_up: None,
        merge_result: None,
    }
}

#[test]
fn a_repair_replaces_optional_core_sections_and_keeps_both_documents() {
    let (mut domain, env) = root();
    domain.forge.restore_begin(env.wall);
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Change(repair()) },
        &mut Queue::uncapped(),
    );
    let documents = Box::new([document(91, None), document(92, None)]);
    let mut wanted = plan(&documents[..], 8);
    for (kind, required) in [
        (brief::Core::Task, true),
        (brief::Core::Results, true),
        (brief::Core::TranscriptTail, true),
        (brief::Core::Plan, false),
        (brief::Core::Attempts, false),
        (brief::Core::NotesIndex, false),
    ] {
        wanted.push(brief::Planned::Core { kind, text: Box::new([]), limit: 1, priority: 0, required }).unwrap();
    }
    finish_brief_plan(&mut domain, &env, 8, Some(7), wanted, documents);
    let sections = take_plan(&mut domain);
    assert_eq!(sections.len(), 8);
    for (index, section) in sections.iter().take(2).enumerate() {
        let brief::Planned::Connector { kind, .. } = section else { panic!("document preserved") };
        assert_eq!(*kind, 16 + u16::try_from(index).unwrap());
    }
    let brief::Planned::Core { kind: brief::Core::Plan, .. } = sections[5] else {
        panic!("first optional core section preserved")
    };
    let brief::Planned::Connector { kind: 3, required: true, .. } = sections[6] else { panic!("required Pull") };
    let brief::Planned::Connector { kind: 1, required: true, .. } = sections[7] else { panic!("required reason") };
}

#[test]
fn required_forge_context_without_optional_core_room_abandons_the_documents() {
    let (mut domain, env) = root();
    domain.forge.restore_begin(env.wall);
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Change(repair()) },
        &mut Queue::uncapped(),
    );
    let documents = Box::new([document(91, None), document(92, None)]);
    let wanted = plan(&documents[..], 2);
    finish_brief_plan(&mut domain, &env, 8, Some(7), wanted, documents);
    let mut failed = false;
    while let Some(work) = domain.work.pop() {
        if let Work::Brief(brief::GatherEvent::Plan { .. }) = work {
            panic!("no room for a ready plan");
        }
        if let Work::Core(jig_domain_core::Event::BriefAssembled { task: 8, ready: false }) = work {
            failed = true;
        }
    }
    assert!(failed);
    assert!(domain.brief_document_titles.is_empty());
    domain.reclaim();
    assert!(domain.brief_connectors.iter().next().is_none());
}

#[test]
fn a_gathered_document_keeps_its_kind_and_frees_the_binding_at_completion() {
    let (mut domain, env) = root();
    let documents = Box::new([document(91, Some(Box::from(b"Safety".as_slice())))]);
    let wanted = plan(&documents[..], 8);
    finish_brief_plan(&mut domain, &env, 8, None, wanted, documents);
    let Some(Work::Forge(event)) = domain.work.pop() else { panic!("document registration") };
    let mut out = Queue::uncapped();
    forge::step(&mut domain.forge, &crate::translate::environment_forge(&env), event, &mut out);
    assert!(out.is_empty());
    let sections = take_plan(&mut domain);
    let brief::Planned::Connector { token, .. } = sections[0] else { panic!("document") };
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let mut requests = Queue::uncapped();
    requests.push(brief::GatherRequest::Gather { connector: 0, section: token, budget: 4096 });
    brief_outputs(&mut domain, &env, &mut decision, &mut requests);
    let Some(Work::Forge(event)) = domain.work.pop() else { panic!("forge gather") };
    forge::step(&mut domain.forge, &crate::translate::environment_forge(&env), event, &mut out);
    let Some(forge::Request::BriefSized { section, size: Some(size) }) = out.pop() else { panic!("gathered words") };
    assert_eq!(section, token);
    assert!(out.is_empty());
    requests.push(brief::GatherRequest::Complete {
        brief: Token::new(8),
        order: Box::new([brief::GatherPlaced::Connector { connector: 0, token, kind: 16, size }]),
    });
    brief_outputs(&mut domain, &env, &mut decision, &mut requests);
    let sections = crate::link::take_sections(&mut domain, 8).unwrap();
    assert_eq!(
        sections[0].kind,
        BriefKind::Forge(ForgeBriefKind::Document(Box::from(b"org/rules docs/rules.md, \"Safety\"".as_slice())))
    );
    assert_eq!(
        sections[0].body,
        BriefBody::Text(Box::from(b"Repository org/rules on forge.example is not adopted.".as_slice()))
    );
    assert_eq!(crate::budget::section_bytes(&sections), u64::from(size) + 33);
    assert!(domain.brief_document_titles.is_empty());
    domain.reclaim();
    assert!(domain.brief_connectors.is_empty());
    assert!(domain.forge.briefs_idle());
}
