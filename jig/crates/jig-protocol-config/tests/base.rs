//! Documents read over an application's base (jig/docs/design/protocol/limits.md, the stock).
#![cfg(test)]
use jig_protocol_config::{self as config, Defaults, Directories, DocumentLimits, Limits, Path, What};
use skein_json::Document;

fn document(limits: &[u8], rest: &[u8]) -> Document {
    let bytes = skein_lib::bytes::concat(&[br#"{"limits":"#, limits, rest]);
    config::document(&bytes, &DocumentLimits { bytes: 2048, tokens: 512, depth: 8 }).expect("fixture JSON")
}
fn engine(limits: &[u8]) -> Document {
    engine_connectors(limits, b"[]")
}
fn engine_connectors(limits: &[u8], connectors: &[u8]) -> Document {
    document(
        limits,
        &skein_lib::bytes::concat(&[
            br#",
        "llm":{"endpoints":[],"providers":[]},
        "hosts":{"workers":[],"listener":[]},
        "clients":{"heartbeat_ms":10000,"link":false},
        "deployment":{"directory":"state"},
        "providers":[{"number":1,"kind":"deployment"}],
        "owners":[{"provider":1,"subject":"owner"}],
        "projects":[],"connectors":"#,
            connectors,
            br#", "memory_bytes":34359738368
    }"#,
        ]),
    )
}
fn worker(limits: &[u8]) -> Document {
    document(
        limits,
        br#",
        "engine":{"in_process":"workers"},"name":"worker-a",
        "secret":"worker.secret","slots":2,
        "programs":{"check_interval_ms":60000},
        "graces":{"grace_ms":30000,"agent_stop_ms":10000,"delivery_ms":20000,"save_ms":5000},
        "agent":{"program":"jig-test","arguments":["agent"],"configuration":"agent.json"}
    }"#,
    )
}
fn base() -> Limits {
    let mut base = config::stock();
    base.core.tasks.tasks = 999;
    // Keep the base consistent even when the document lowers tasks.tasks to five.
    base.core.people.inbox_tasks = 1998;
    base.core.people.operations = 5;
    base.core.tasks.delegates = 4;
    base.core.tasks.project_goals = 5;
    base.core.tasks.exemptions = 5;
    base
}
const CASES: [(&[u8], u32); 3] =
    [(br#"{"overrides":{}}"#, 999), (br#"{"profile":"standard"}"#, 999), (br#"{"overrides":{"tasks.tasks":5}}"#, 5)];

#[test]
fn engine_limits_read_over_the_base() {
    let base = base();
    for (limits, expected) in CASES {
        let engine = config::engine_over(&engine(limits), 0, &Directories::working(), Defaults::Passed, &base)
            .expect("engine limits");
        assert_eq!(engine.limits.core.tasks.tasks, expected);
    }
}
#[test]
fn worker_limits_read_over_the_base() {
    let base = base();
    for (limits, expected) in CASES {
        let worker = config::worker_over(&worker(limits), 0, &Directories::working(), &base).expect("worker limits");
        assert_eq!(worker.limits.core.tasks.tasks, expected);
    }
}
#[test]
fn unknown_profiles_are_refused_at_limits_profile() {
    for limits in [br#"{"profile":"small"}"#.as_slice(), br#"{"profile":"large"}"#] {
        let base = base();
        let engine = config::engine_over(&engine(limits), 0, &Directories::working(), Defaults::Passed, &base)
            .expect_err("unknown engine profile");
        let worker = config::worker_over(&worker(limits), 0, &Directories::working(), &base)
            .expect_err("unknown worker profile");
        for problems in [engine, worker] {
            assert_eq!(
                problems.problems.as_slice(),
                &[config::Problem { path: Path::root().member(b"limits").member(b"profile"), what: What::Override },]
            );
            assert_eq!(problems.past, 0);
        }
    }
}
#[test]
fn refused_profiles_still_check_the_document_against_the_base() {
    let mut base = config::stock();
    base.core.connectors = 1;
    base.link.name = 1;
    for limits in [br#"{"profile":"small"}"#.as_slice(), br#"{"profile":"large"}"#] {
        let engine = config::engine_over(
            &engine_connectors(limits, b"[1,2]"),
            0,
            &Directories::working(),
            Defaults::Passed,
            &base,
        )
        .expect_err("profile and connector count");
        let worker = config::worker_over(&worker(limits), 0, &Directories::working(), &base)
            .expect_err("profile and worker name");
        for (problems, member) in [(engine, b"connectors".as_slice()), (worker, b"name")] {
            assert_eq!(
                problems.problems.as_slice(),
                &[
                    config::Problem { path: Path::root().member(b"limits").member(b"profile"), what: What::Override },
                    config::Problem { path: Path::root().member(member), what: What::Bound },
                ]
            );
            assert_eq!(problems.past, 0);
        }
    }
}
#[test]
fn default_readers_use_jigs_stock() {
    let stock = config::stock();
    let engine = engine(b"{}");
    let directories = Directories::working();
    let default = config::engine(&engine, 0, &directories, Defaults::Passed).expect("stock engine");
    let over = config::engine_over(&engine, 0, &directories, Defaults::Passed, &stock).expect("engine over stock");
    assert_eq!(default.limits, over.limits);
    assert_eq!(default.limits.core.tasks.tasks, stock.core.tasks.tasks);
    let worker = worker(b"{}");
    let default = config::worker(&worker, 0, &directories).expect("stock worker");
    let over = config::worker_over(&worker, 0, &directories, &stock).expect("worker over stock");
    assert_eq!(default.limits, over.limits);
    assert_eq!(default.limits.core.tasks.tasks, stock.core.tasks.tasks);
}
