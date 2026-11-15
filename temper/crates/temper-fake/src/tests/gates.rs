//! Gates and merges declared by the process document (testing/fakes.md, 2).
use super::{forge, forge_document};
use serde_json::{Value, json};
use skein_fake_forge_domain::{self as domain, api as a};
use skein_fake_peers::forge::Forge;
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall, bytes::copy_of};

fn decode(value: &Value) -> forge::Document {
    forge::Document::decode(&serde_json::to_vec(value).unwrap()).unwrap()
}
fn workflow(cue: Value) -> Value {
    json!({"file":"gate.yml", "jobs":["gate"], "cue":cue})
}
fn call(forge: &mut Forge, env: &mut Env<domain::Config>, name: &[u8], op: a::Op) -> a::Answer {
    let mut out = Queue::uncapped();
    domain::step(
        &mut forge.domain,
        env,
        domain::Event::Call {
            reply_to: ReplyTo::new(Token::new(1)),
            user: forge::OWNER,
            repository: copy_of(name),
            op,
        },
        &mut out,
    );
    for _ in 0..256 {
        while let Some(request) = out.pop() {
            if let domain::Request::Reply { result, .. } = request {
                forge.domain.reclaim();
                return result.unwrap();
            }
        }
        env.now = forge.domain.next_deadline().expect("call timer");
        domain::fire(&mut forge.domain, env, &mut out);
    }
    panic!("call did not answer");
}
fn settle(forge: &mut Forge, env: &mut Env<domain::Config>) {
    let mut out = Queue::uncapped();
    for _ in 0..256 {
        let Some(at) = forge.domain.next_deadline() else { return };
        env.now = env.now.max(at);
        domain::fire(&mut forge.domain, env, &mut out);
        out.clear();
        forge.domain.reclaim();
    }
    panic!("forge did not settle");
}
fn jobs(forge: &Forge, name: &[u8], commit: u64) -> (a::Run, Box<[a::Job]>) {
    let a::Answer::Runs(runs) = forge
        .domain
        .inspect(
            &forge.config,
            name,
            &a::Read::Runs {
                workflow: Some(copy_of(b"gate.yml")),
                commit: Some(commit),
                number: None,
                page: 1,
                limit: 1,
            },
        )
        .unwrap()
    else {
        panic!("runs")
    };
    let [run] = runs.as_ref() else { panic!("one dispatch run") };
    let a::Answer::Jobs(jobs) = forge.domain.inspect(&forge.config, name, &a::Read::Jobs { run: run.id }).unwrap()
    else {
        panic!("jobs")
    };
    (run.clone(), jobs)
}
fn dispatch(forge: &mut Forge, env: &mut Env<domain::Config>, name: &[u8], inputs: Box<[a::WorkflowInput]>) {
    assert_eq!(
        call(
            forge,
            env,
            name,
            a::Op::Write(a::Write::Dispatch { workflow: copy_of(b"gate.yml"), branch: copy_of(b"main"), inputs })
        ),
        a::Answer::Done
    );
}

#[test]
fn document_gates_judge_the_dispatched_tree_at_the_ci_pace() {
    for (cue, missing, present) in [
        (json!({"declared":".fake-forge/gate"}), a::Conclusion::Success, a::Conclusion::Failure),
        (Value::Null, a::Conclusion::Success, a::Conclusion::Success),
        (json!({"path":".fake-forge/gate", "green":"fail"}), a::Conclusion::Failure, a::Conclusion::Success),
    ] {
        let mut value = forge_document();
        value["workflows"] = json!([workflow(cue)]);
        value["pace"] = json!({"ci_ms":1500});
        value["sources"] = json!([{"address":"https://code.invalid/source/kit", "repository":{
            "owner":"source", "name":"kit", "landing":"main", "tree":[]}}]);
        let (mut forge, _) = decode(&value).seed(23);
        assert_eq!((forge.config.limits.workflows, forge.config.limits.workflow_runs), (8, 32));
        let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge.config.clone() };
        domain::create_repository(&mut forge.domain, &env.limits, 3, b"fake/made", b"made", forge::OWNER).unwrap();
        for name in [b"fake/widget".as_slice(), b"fake/made", b"source/kit"] {
            domain::grant(&mut forge.domain, name, forge::OWNER, a::Permission::Admin);
            for (content, expected) in [(None, missing), (Some(b"fail".as_slice()), present)] {
                if let Some(content) = content {
                    domain::advance(&mut forge.domain, &env, name, b"main", b".fake-forge/gate", content, forge::OWNER)
                        .unwrap();
                }
                settle(&mut forge, &mut env);
                let head = forge.domain.branch(name, b"main").unwrap();
                dispatch(&mut forge, &mut env, name, Box::new([]));
                let (run, pending) = jobs(&forge, name, head);
                assert_eq!(run.commit, head);
                assert_eq!(run.event, a::RunEvent::Dispatch);
                assert_eq!(pending.len(), 1);
                assert_eq!(pending[0].name.as_ref(), b"gate");
                assert_eq!(pending[0].conclusion, None);
                assert_eq!(forge.domain.next_deadline(), Some(env.now.saturating_add(Duration::from_millis(1500))));
                settle(&mut forge, &mut env);
                let (run, completed) = jobs(&forge, name, head);
                assert_eq!(run.state, a::RunState::Completed);
                assert_eq!(run.conclusion, Some(expected));
                assert_eq!(completed[0].conclusion, Some(expected));
            }
        }
    }
}

#[test]
fn document_gates_keep_probe_inputs_without_declaring_them_and_reach_the_same_verdict() {
    let mut value = forge_document();
    value["workflows"] = json!([workflow(json!({"declared":".fake-forge/gate"}))]);
    let (mut forge, _) = decode(&value).seed(23);
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge.config.clone() };
    let name = b"fake/widget";
    domain::grant(&mut forge.domain, name, forge::OWNER, a::Permission::Admin);
    let head = forge.domain.branch(name, b"main").unwrap();
    let mut runs = Vec::new();
    for probe in [true, false] {
        let mut inputs = vec![
            (copy_of(b"temper_commit"), head.to_string().into_bytes().into_boxed_slice()),
            (copy_of(b"temper_key"), copy_of(b"key")),
        ];
        if probe {
            inputs.push((copy_of(b"temper_probe"), copy_of(b"yes")));
        }
        let inputs = inputs.into_boxed_slice();
        dispatch(&mut forge, &mut env, name, inputs.clone());
        settle(&mut forge, &mut env);
        let (run, completed) = jobs(&forge, name, head);
        assert_eq!(run.commit, head);
        assert_eq!(run.event, a::RunEvent::Dispatch);
        assert_eq!(run.inputs, inputs);
        assert_eq!(run.state, a::RunState::Completed);
        assert_eq!(completed[0].conclusion, run.conclusion);
        runs.push(run);
    }
    assert_ne!(runs[0].id, runs[1].id);
    assert_eq!(runs[0].conclusion, Some(a::Conclusion::Success));
    assert_eq!(runs[0].conclusion, runs[1].conclusion);
}

fn fast_forward(forge: &mut Forge, env: &mut Env<domain::Config>, name: &[u8], enabled: bool) {
    let parent = forge.domain.branch(name, b"main").unwrap();
    domain::grant(&mut forge.domain, name, forge::OWNER, a::Permission::Admin);
    call(forge, env, name, a::Op::Write(a::Write::CreateBranch { branch: copy_of(b"work"), commit: parent }));
    let head = domain::advance(&mut forge.domain, env, name, b"work", b"change", b"member", forge::OWNER).unwrap();
    let a::Answer::Created(number) = call(
        forge,
        env,
        name,
        a::Op::Write(a::Write::OpenPull {
            title: copy_of(b"member"),
            body: copy_of(b""),
            head: copy_of(b"work"),
            base: copy_of(b"main"),
            opens_empty: false,
        }),
    ) else {
        panic!("pull")
    };
    call(forge, env, name, a::Op::Git(a::Git::Push { branch: copy_of(b"main"), commit: head, expected: Some(parent) }));
    let a::Answer::Pull(pull) = forge.domain.inspect(&forge.config, name, &a::Read::Pull { number }).unwrap() else {
        panic!("pull")
    };
    assert_eq!(pull.merged, if enabled { Some(head) } else { None });
    assert_eq!(pull.state, if enabled { a::State::Closed } else { a::State::Open });
}

#[test]
fn document_manual_merges_detect_fast_forwards_only_when_enabled() {
    for settings in [None, Some(json!({})), Some(json!({"allow":true})), Some(json!({"autodetect":true}))] {
        let mut value = forge_document();
        if let Some(settings) = &settings {
            value["manual_merges"] = settings.clone();
        }
        let (mut forge, _) = decode(&value).seed(23);
        assert_eq!((forge.config.limits.workflows, forge.config.limits.workflow_runs), (0, 0));
        let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge.config.clone() };
        domain::create_repository(&mut forge.domain, &env.limits, 2, b"fake/made", b"made", forge::OWNER).unwrap();
        let enabled = settings.as_ref().is_some_and(|s| s["autodetect"] == true);
        for name in [b"fake/widget".as_slice(), b"fake/made"] {
            fast_forward(&mut forge, &mut env, name, enabled);
        }
    }
}

#[test]
fn document_gate_members_and_bounds_are_refused_by_name() {
    let good = workflow(Value::Null);
    for (pointer, replacement, expected) in [
        ("/workflows/0/file", json!(""), "workflows[0].file: must be non-empty\n"),
        ("/workflows/0/file", json!("dir/gate.yml"), "workflows[0].file: must be a file name without /\n"),
        ("/workflows/0/file", json!("f".repeat(65)), "workflows[0].file: past its bound\n"),
        ("/workflows/0", json!({"file":"gate.yml", "cue":null}), "workflows[0].jobs: missing\n"),
        ("/workflows/0/jobs", json!([]), "workflows[0].jobs: must be non-empty\n"),
        (
            "/workflows/0/jobs",
            json!(["a", "b", "c", "d", "e", "f", "g", "h", "i"]),
            "workflows[0].jobs: past its bound\n",
        ),
        ("/workflows/0/jobs", json!(["not a word"]), "workflows[0].jobs[0]: expected a word\n"),
        ("/workflows/0/jobs", json!([""]), "workflows[0].jobs[0]: expected a word\n"),
        ("/workflows/0/jobs", json!(["gate", "gate"]), "workflows[0].jobs[1]: must be unique\n"),
        ("/workflows", json!([good, good]), "workflows[1].file: must be unique\n"),
        (
            "/workflows",
            json!(
                (0..9).map(|i| json!({"file":format!("gate-{i}.yml"),"jobs":["gate"],"cue":null})).collect::<Vec<_>>()
            ),
            "workflows: past its bound\n",
        ),
        (
            "/workflows/0",
            json!({"file":"gate.yml","jobs":["gate"],"cue":null,"extra":true}),
            "workflows[0].extra: unknown member\n",
        ),
        ("/manual_merges", json!({"extra":true}), "manual_merges.extra: unknown member\n"),
        ("/manual_merges/allow", Value::Null, "manual_merges.allow: wrong value kind\n"),
        ("/manual_merges/autodetect", json!(1), "manual_merges.autodetect: wrong value kind\n"),
        (
            "/workflows/0/cue",
            json!({"declared":"../bad"}),
            "workflows[0].cue.declared: expected a relative file path with no empty, . or .. component\n",
        ),
    ] {
        let mut value = forge_document();
        value["workflows"] = json!([good]);
        value["manual_merges"] = json!({"allow":false,"autodetect":false});
        *value.pointer_mut(pointer).unwrap() = replacement;
        assert_eq!(
            forge::Document::decode(&serde_json::to_vec(&value).unwrap()).unwrap_err().line().as_ref(),
            expected.as_bytes(),
            "{pointer}"
        );
    }
    let mut value = forge_document();
    value["workflows"] = json!((0..8).map(|i| json!({"file":format!("{i}{}", "f".repeat(63)),"jobs":["a","b","c","d","e","f","g","job_8-x"],"cue":null})).collect::<Vec<_>>());
    let (forge, _) = decode(&value).seed(23);
    assert_eq!(forge.config.made_workflows.len(), 8);
}
