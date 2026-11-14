/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn the_engine_declares_the_merge_kind_where_its_tasks_child_holds_it() {
    let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let bound = limits::merge_bound();
    assert_eq!(limits::merge_jobs(&jig).as_ref(), [bound]);
    jig.core.tasks.result_bytes = 32;
    assert!(limits::merge_jobs(&jig).is_empty());
    jig.core.tasks.result_bytes = bound.result;
    jig.core.tasks.spec_bytes = bound.spec - 1;
    assert!(limits::merge_jobs(&jig).is_empty());
    jig.core.tasks.spec_bytes = bound.spec;
    assert_eq!(limits::merge_jobs(&jig).as_ref(), [bound]);
}

use crate::limits::{self, Checkout, GitComponent, Peer, Profile, Refused, Shared};
use skein_lib::Duration;

/// Both production profiles bound preparing, runs' and jobs' alike.
#[test]
fn profiles_prepare_one_workspace_at_a_time() {
    for profile in [Profile::Small, Profile::Standard] {
        assert_eq!(limits::profile(profile).checkout.preparations, 1);
    }
}

#[test]
fn preparation_overrides_refuse_zero_naming_the_value_and_minimum() {
    use crate::startup::{
        configured_limits,
        problems::{SharedProblems, render_problem},
    };
    use jig_protocol_config::{self as config, Path};
    use skein_lib::Writer;
    for (input, value) in [
        (br#"{"profile":"small","overrides":{"checkout.preparations":0}}"#.as_slice(), 0),
        (br#"{"profile":"small","overrides":{"checkout.preparations":1}}"#, 1),
        (br#"{"profile":"standard","overrides":{"checkout.preparations":7}}"#, 7),
    ] {
        let document = config::document(input, &config::DocumentLimits { bytes: 1024, tokens: 128, depth: 8 }).unwrap();
        let mut problems = SharedProblems::empty();
        let path = Path::root().member(b"application").member(b"limits");
        let limits = configured_limits(&document, 0, &path, &mut problems).unwrap();
        assert_eq!(limits.checkout.preparations, value);
        if value == 0 {
            assert_eq!(problems.problems.len(), 1);
            let expected = b"application.limits.overrides.checkout.preparations: 0; requires at least 1\n";
            let mut out = Writer::new(expected.len());
            render_problem(problems.problems.get(0).unwrap(), &mut out, false);
            assert_eq!(out.finish().as_ref(), expected);
        } else {
            assert!(problems.is_empty(), "a nonzero override starts");
        }
    }
}

/// Each dotted name changes its field alone, with checked representations.
#[test]
fn shared_overrides_apply_only_to_the_named_field() {
    let small = limits::profile(Profile::Small);
    for (name, expected) in [
        (b"checkout.workspaces".as_slice(), Shared { checkout: Checkout { workspaces: 7, ..small.checkout }, ..small }),
        (b"checkout.preparations", Shared { checkout: Checkout { preparations: 7, ..small.checkout }, ..small }),
        (b"checkout.repositories", Shared { checkout: Checkout { repositories: 7, ..small.checkout }, ..small }),
        (b"checkout.name_bytes", Shared { checkout: Checkout { name_bytes: 7, ..small.checkout }, ..small }),
        (b"checkout.message_bytes", Shared { checkout: Checkout { message_bytes: 7, ..small.checkout }, ..small }),
        (b"checkout.conflicts", Shared { checkout: Checkout { conflicts: 7, ..small.checkout }, ..small }),
        (b"checkout.path_bytes", Shared { checkout: Checkout { path_bytes: 7, ..small.checkout }, ..small }),
        (
            b"checkout.remote_timeout_ms",
            Shared { checkout: Checkout { remote_timeout: Duration::from_millis(7), ..small.checkout }, ..small },
        ),
        (
            b"checkout.local_timeout_ms",
            Shared { checkout: Checkout { local_timeout: Duration::from_millis(7), ..small.checkout }, ..small },
        ),
        (b"checkout.facts", Shared { checkout: Checkout { facts: 7, ..small.checkout }, ..small }),
        (
            b"git_component.operations",
            Shared { git_component: GitComponent { operations: 7, ..small.git_component }, ..small },
        ),
        (
            b"git_component.stdout_bytes",
            Shared { git_component: GitComponent { stdout_bytes: 7, ..small.git_component }, ..small },
        ),
        (
            b"git_component.stderr_bytes",
            Shared { git_component: GitComponent { stderr_bytes: 7, ..small.git_component }, ..small },
        ),
        (
            b"git_component.read_bytes",
            Shared { git_component: GitComponent { read_bytes: 7, ..small.git_component }, ..small },
        ),
        (
            b"git_component.sweep_entries",
            Shared { git_component: GitComponent { sweep_entries: 7, ..small.git_component }, ..small },
        ),
        (
            b"git_component.sweep_bytes",
            Shared { git_component: GitComponent { sweep_bytes: 7, ..small.git_component }, ..small },
        ),
        (b"forge.forges", Shared { peer: Peer { forges: 7, ..small.peer }, ..small }),
        (b"forges.installations", Shared { peer: Peer { installations: 7, ..small.peer }, ..small }),
        (b"forge_component.token_bytes", Shared { peer: Peer { token_bytes: 7, ..small.peer }, ..small }),
    ] {
        let mut actual = small;
        assert_eq!(limits::apply(&mut actual, name, 7), Ok(()), "{name:?}");
        assert_eq!(actual, expected, "{name:?}");
        assert_eq!(limits::apply(&mut actual, name, u64::MAX), Err(Refused::Range), "{name:?}");
        assert_eq!(actual, expected, "a refused override leaves the field intact");
    }
    let mut actual = small;
    assert_eq!(limits::apply(&mut actual, b"checkout.unknown", 7), Err(Refused::Unknown));
    assert_eq!(actual, small);
}

/// The common grammar refuses unknown names and malformed integers, not engine ranges.
#[test]
fn shared_limits_grammar_checks_names_and_integer_representations() {
    use crate::startup::{
        configured_limits,
        problems::{SharedProblems, SharedWhat as What},
    };
    use jig_protocol_config::{self as config, Path};
    for (input, expected) in [
        (br#"{"profile":"small","overrides":{"checkout.conflicts":7,"forge.change_policy.tasks":0,"forge.changes":18446744073709551615}}"#.as_slice(), None),
        (br#"{"profile":"small","overrides":{"unknown":7}}"#, Some((b"unknown".as_slice(), What::Override(Refused::Unknown)))),
        (br#"{"profile":"small","overrides":{"checkout.conflicts":4294967296}}"#, Some((b"checkout.conflicts", What::Override(Refused::Range)))),
        (br#"{"profile":"small","overrides":{"forge.changes":18446744073709551616}}"#, Some((b"forge.changes", What::Override(Refused::Range)))),
        (br#"{"profile":"small","overrides":{"forge.changes":"7"}}"#, Some((b"forge.changes", What::Jig(config::What::Kind)))),
    ] {
        let document = config::document(input, &config::DocumentLimits { bytes: 1024, tokens: 128, depth: 8 }).unwrap();
        let mut problems = SharedProblems::empty();
        let limits = configured_limits(&document, 0, &Path::root(), &mut problems).unwrap();
        if let Some((name, what)) = expected {
            assert_eq!(problems.problems.len(), 1);
            let problem = problems.problems.get(0).unwrap();
            assert_eq!((&problem.path, problem.what), (&Path::root().member(b"overrides").member(name), what));
        } else {
            assert!(problems.is_empty());
            assert_eq!(limits.checkout.conflicts, 7);
        }
    }
}

#[test]
fn reds_wait_is_stock_in_both_profiles_and_outside_its_range_names_the_override() {
    use crate::startup::{
        configured_limits,
        problems::{SharedProblems, render_problem},
    };
    use jig_protocol_config::{self as config, Path};
    use skein_lib::Writer;
    for profile in [Profile::Small, Profile::Standard] {
        assert_eq!(limits::profile(profile).checkout.reds_read_ms, 5_000);
    }
    for value in [100, 60_000] {
        let mut shared = limits::profile(Profile::Small);
        limits::apply(&mut shared, b"checkout.reds_read_ms", value).unwrap();
        assert_eq!(u64::from(shared.checkout.reds_read_ms), value);
    }
    for input in [
        br#"{"profile":"small","overrides":{"checkout.reds_read_ms":99}}"#.as_slice(),
        br#"{"profile":"small","overrides":{"checkout.reds_read_ms":60001}}"#,
    ] {
        let document = config::document(input, &config::DocumentLimits { bytes: 1024, tokens: 128, depth: 8 }).unwrap();
        let mut problems = SharedProblems::empty();
        let path = Path::root().member(b"limits");
        let _limits = configured_limits(&document, 0, &path, &mut problems);
        assert!(!problems.is_empty());
        let problem = problems.problems.get(0).unwrap();
        let size = render_problem(problem, &mut Writer::new(0), true);
        let mut out = Writer::new(size);
        render_problem(problem, &mut out, false);
        assert!(skein_lib::bytes::find(&out.finish(), b"checkout.reds_read_ms").is_some());
    }
}

#[test]
fn rerun_processes_is_stock_and_accepts_its_endpoints_but_names_seventeen() {
    use crate::startup::{
        configured_limits,
        problems::{SharedProblems, render_problem},
    };
    use jig_protocol_config::{self as config, Path};
    use skein_lib::Writer;
    for profile in [Profile::Small, Profile::Standard] {
        assert_eq!(limits::profile(profile).checkout.rerun_processes, 4);
        for value in [0, 16] {
            let mut shared = limits::profile(profile);
            limits::apply(&mut shared, b"checkout.rerun_processes", value).unwrap();
            assert_eq!(u64::from(shared.checkout.rerun_processes), value);
        }
    }
    let document = config::document(
        br#"{"profile":"small","overrides":{"checkout.rerun_processes":17}}"#,
        &config::DocumentLimits { bytes: 1024, tokens: 128, depth: 8 },
    )
    .unwrap();
    let mut problems = SharedProblems::empty();
    let _limits = configured_limits(&document, 0, &Path::root().member(b"limits"), &mut problems);
    assert!(!problems.is_empty());
    let problem = problems.problems.get(0).unwrap();
    let size = render_problem(problem, &mut Writer::new(0), true);
    let mut out = Writer::new(size);
    render_problem(problem, &mut out, false);
    assert!(skein_lib::bytes::find(&out.finish(), b"checkout.rerun_processes").is_some());
}
