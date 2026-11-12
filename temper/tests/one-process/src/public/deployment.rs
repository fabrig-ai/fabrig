//! Opening figures and start refusals (jig/docs/design/protocol/limits.md).
use crate::{
    fixtures, stories,
    world::{self, Story},
};
use skein_io::kernel::Fd;

fn start(value: &serde_json::Value) -> Result<temper_service::Service, String> {
    let mut document = value.clone();
    temper_playground::documents::settings::strip_models(&mut document);
    document.as_object_mut().expect("deployment object").remove("world_settings");
    temper_service::startup::with_setup_given(
        &serde_json::to_vec(&document).expect("deployment JSON"),
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        Some(&temper_playground::documents::agent()),
        &fixtures::seeds(),
        None,
        Some(temper_service::WorkerRuntime {
            root: Fd::new(3),
            path: b"/worker".as_slice().into(),
            agent: temper_playground::documents::pinned_agent(),
        }),
        temper_playground::documents::forges(),
        None,
        None,
        None,
        &stories::given(&fixtures::world_document()),
        Box::new([]),
    )
    .map_err(|problems| String::from_utf8_lossy(&temper_service::startup::lines(&problems)).into_owned())
}

pub(super) fn relations() {
    super::ACTIVE.with(|active| active.set(true));
    stories::set(Story::Chat { words: b"Hello" });
    stories::setup_settings(world::Phase::Restart);
    let value = fixtures::world_document();
    eprintln!(
        "public jig kind bounds {:?}",
        jig_client_protocol_wire::public_bounds(&jig_client_protocol_wire::PublicIndexBounds {
            goals: 4,
            ended: 4,
            lines: 8,
            depth: 4,
            projects: 4
        })
    );
    start(&value).unwrap_or_else(|problem| {
        panic!("the specified public deployment must start before testing broken relations: {problem}")
    });
    for (name, broken) in [
        ("clients.public.rate_burst", 7),
        ("clients.public.bytes_burst", super::PUBLIC_BYTES - 1),
        ("clients.public.answer_bytes", super::PUBLIC_BYTES + 1),
        ("clients_http.public_calls", 5),
        ("clients.public.max_age", 61),
        ("clients.public.answer_bytes", super::PUBLIC_ANSWER_BYTES - 1),
        ("core.public.goals", 0),
        ("core.public.ended", 0),
        ("core.public.lines", 0),
        ("core.public.depth", 0),
    ] {
        let mut bad = value.clone();
        bad["engine"]["limits"]["overrides"][name] = broken.into();
        let Err(problem) = start(&bad) else {
            panic!("broken relation {name}={broken} started");
        };
        assert!(problem.contains(name), "broken relation {name}={broken} must name its path: {problem}");
    }
    let mut no_room = value.clone();
    no_room["engine"]["limits"]["overrides"]["clients.public.reads"] = 0.into();
    no_room["engine"]["limits"]["overrides"]["clients_http.public_calls"] = 0.into();
    let problem = start(&no_room).err().expect("positive rate needs room");
    assert!(problem.contains("clients.public.reads"), "{problem}");
    let mut past_index = value.clone();
    for name in ["core.public.goals", "core.public.ended", "core.public.lines"] {
        past_index["engine"]["limits"]["overrides"][name] = u16::MAX.into();
    }
    let problem = start(&past_index).err().expect("index memory exceeds its fit");
    assert!(problem.contains("core.public.lines"), "index memory names its path: {problem}");
    let mut isolated_index = value.clone();
    for (name, count) in [
        ("core.public.goals", 256),
        ("core.public.ended", 256),
        ("core.public.lines", 256),
        ("people.projects", 4096),
        ("clients.public.answer_bytes", u64::from(u32::MAX)),
        ("clients.public.bytes", u64::from(u32::MAX)),
        ("clients.public.bytes_burst", u64::from(u32::MAX)),
    ] {
        isolated_index["engine"]["limits"]["overrides"][name] = count.into();
    }
    let problem = start(&isolated_index).err().expect("only the public index memory relation is broken");
    assert!(problem.contains("core.public.lines"), "isolated index memory names its path: {problem}");
    assert_eq!(problem.lines().count(), 1, "only one broken relation: {problem}");
    // A schema ceiling is a kind whose bound cannot fit the answer ceiling.
    let mut past_kind = value.clone();
    past_kind["engine"]["limits"]["overrides"]["core.public.lines"] = 257.into();
    let problem = start(&past_kind).err().expect("kind exceeds its schema ceiling");
    assert!(problem.contains("clients.public.answer_bytes"), "{problem}");
    super::ACTIVE.with(|active| active.set(false));
}

pub(super) fn provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let call = |name: &[u8], arguments: &[u8]| Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    };
    let grant = |kind| serde_json::json!({"connector":fixtures::FORGE_CONNECTOR,"kind":kind,"segments":["forge","127.0.0.1:3000"],"terminal":"open","last":"acme"});
    let mut delegate = serde_json::json!({"batch":[{
        "executor":{"kind":"procedure","connector":fixtures::FORGE_CONNECTOR,"code":fixtures::PROCEDURE_CODE},
        "tracked":{"priority":1},
        "spec":{"words":"@edit Public change title\nPRIVATE_SPEC_BODY","parameters":[
            {"name":fixtures::PARAMETER_NAMES[0],"kind":"resource","connector":fixtures::FORGE_CONNECTOR,"resource":fixtures::REPOSITORY_TAG},
            {"name":fixtures::PARAMETER_NAMES[1],"kind":"bytes","value":"main"},
            {"name":fixtures::PARAMETER_NAMES[2],"kind":"bytes","value":std::str::from_utf8(fixtures::MERGE_BRANCH).expect("branch")},
            {"name":10,"kind":"bytes","value":"PRIVATE_PARAMETER"}]},
        "contract":{"kind":"change","connector":fixtures::FORGE_CONNECTOR,"change_kind":fixtures::CHANGE_KIND,"words":fixtures::CHANGE_WORDS},
        "authority":{"tools":fixtures::PRODUCER_TOOLS,"grants":([3,4,11].map(grant)),"delegation":{"kinds":[{"kind":"agent","number":fixtures::AGENT_KIND}],"tasks":fixtures::PRODUCER_TASKS,"depth":fixtures::PRODUCER_DEPTH},"budget":{"spend":fixtures::PRODUCER_SPEND},"notes":fixtures::PRODUCER_NOTES}
    }]});
    if !super::TRACKED.with(core::cell::Cell::get) {
        delegate["batch"][0].as_object_mut().expect("delegate").remove("tracked");
    }
    let delegate = serde_json::to_vec(&delegate).expect("delegate JSON");
    let finish = call(b"finish", br#"{"form":"change","text":"PRIVATE_RESULT","fields":{"title":"PRIVATE_PULL_TITLE","body":"PRIVATE_PULL_BODY"}}"#);
    fixtures::scripted_provider(Box::new([
        Script {
            cue: b"@plan".as_slice().into(),
            turns: Box::new([call(b"delegate", &delegate), call(b"wait", b"{}"), call(b"wait", b"{}")]),
        },
        Script { cue: b"@edit".as_slice().into(), turns: Box::new([finish.clone()]) },
        Script {
            cue: b"Resolve the merge conflict".as_slice().into(),
            turns: Box::new([call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#), finish]),
        },
    ]))
}
