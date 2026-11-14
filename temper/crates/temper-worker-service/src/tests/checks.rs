#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect diagnostic text and edit document fixtures"
)]
use super::fixtures;
use crate::startup::{Problems, What, lines};
use jig_protocol_config as config;
use jig_protocol_config::Path;
#[test]
fn a_worker_part_names_the_agents_program() {
    let part =
        crate::startup::worker_part(&fixtures::worker_alone_document(), &config::Directories::working()).unwrap();
    assert_eq!(part.named.program.as_ref(), fixtures::TEMPER);
}

#[test]
fn worker_checkout_capacity_names_stdout_and_its_room_without_an_engine() {
    let document = fixtures::with(
        &fixtures::worker_alone_document(),
        b"application.limits.overrides",
        br#"{"checkout.conflicts":1024,"checkout.path_bytes":4096,"git_component.stdout_bytes":262144}"#,
    );
    let mut agent = fixtures::startup_agent();
    agent.host_limits.path_bytes = 4096;
    let part = crate::startup::worker_part(&document, &config::Directories::working()).unwrap();
    let found = crate::startup::worker_alone(part, &fixtures::files(), &agent, fixtures::worker_runtime(), 7)
        .err()
        .expect("checkout capacity is refused");
    assert_eq!(found.problems.len(), 1);
    assert_eq!(
        String::from_utf8_lossy(&lines(&found)),
        "checkout.conflicts 1024, checkout.path_bytes 4096, checkout.repositories 2: raise these with them\ngit_component.stdout_bytes: 262144 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 4195328 bytes\nworkers[0].limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401238 bytes\n"
    );
}

#[test]
fn a_worker_alone_names_each_shape_departure() {
    use config::Segment::Index;
    let root = Path::root();
    let workers = root.member(b"workers");
    let engine = workers.child(Index(0)).member(b"engine");
    let base = fixtures::worker_alone_document();
    let rows: &[ShapeRow] = &[
        (b"workers", b"[]", vec![(workers.clone(), What::WorkerAlone)]),
        (b"workers[0].engine", br#"{"in_process":"workers"}"#, vec![(engine.clone(), What::WorkerAlone)]),
        (
            b"workers[0].engine",
            br#"{"socket":"192.0.2.1:7000"}"#,
            vec![(engine.clone(), What::Jig(config::What::Plaintext))],
        ),
        (
            b"workers[0].engine",
            br#"{"socket":"127.0.0.1:7000","transport":"tls"}"#,
            vec![(engine.member(b"transport"), What::Jig(config::What::Unknown))],
        ),
        (b"application.workers", b"[]", vec![(root.member(b"application").member(b"workers"), What::WorkerAlone)]),
        (
            b"application.workers",
            br#"[{"root":"one","git":null},{"root":"two","git":null}]"#,
            vec![(root.member(b"application").member(b"workers"), What::WorkerAlone)],
        ),
        (
            b"application",
            br#"{"workers":[{"root":"worker","git":null}],"limits":{"profile":"small","overrides":{}},"forges":[]}"#,
            vec![(root.member(b"application").member(b"forges"), What::WorkerAlone)],
        ),
    ];
    for (member, value, expected) in rows {
        let input = fixtures::with(&base, member, value);
        assert_eq!(&shape_problems(&input), expected, "{}", core::str::from_utf8(member).unwrap());
    }
    let stray = [b"{\"stray\":null,".as_slice(), &base[1..]].concat();
    assert_eq!(shape_problems(&stray), vec![(root.member(b"stray"), What::WorkerAlone)]);
    // Duplicate whole worker documents by their existing raw list body.
    let text = core::str::from_utf8(&base).unwrap();
    let (_, body) = text.split_once("\"workers\":[").unwrap();
    let (body, _) = body.split_once("],\"application\":").unwrap();
    let two = [b"[".as_slice(), body.as_bytes(), b",", body.as_bytes(), b"]"].concat();
    assert_eq!(shape_problems(&fixtures::with(&base, b"workers", &two)), vec![(workers, What::WorkerAlone)]);
}

fn shape_problems(input: &[u8]) -> Vec<(Path, What)> {
    crate::startup::shape(input, &config::Directories::working())
        .unwrap_err()
        .problems
        .iter()
        .map(|problem| (problem.path.clone(), problem.what))
        .collect()
}

type ShapeRow = (&'static [u8], &'static [u8], Vec<(Path, What)>);
#[test]
fn the_cluster_shapes_accept_both_loopback_families_and_the_whole_v4_range() {
    for socket in [br#"{"socket":"127.42.3.4:7100"}"#.as_slice(), br#"{"socket":"[::1]:7100"}"#] {
        let worker = fixtures::with(&fixtures::worker_alone_document(), b"workers[0].engine", socket);
        assert_eq!(
            crate::startup::shape(&worker, &config::Directories::working()),
            Ok(crate::startup::Shape::WorkerAlone)
        );
    }
}

#[test]
fn turn_checks_use_only_what_each_process_knows() {
    let worker = fixtures::worker_settings().jig;
    let mut agent = fixtures::startup_agent();
    let mut found = Problems::empty();
    agent.limits.bodies.turn_body = u32::try_from(worker.limits.hub.turn_bytes).unwrap();
    crate::startup::worker::checks::turns(core::slice::from_ref(&worker), &agent, &mut found);
    assert_eq!(&*lines(&found), b"");
    agent.limits.bodies.turn_body += 1;
    crate::startup::worker::checks::turns(core::slice::from_ref(&worker), &agent, &mut found);
    assert_eq!(
        &*lines(&found),
        b"workers[0].limits.overrides.hub.turn_bytes: 65536 bytes, short of the agent's largest turn, 65537 bytes\n"
    );
}

#[test]
fn installations_past_the_grant_bound_refuse_the_start_naming_the_limit() {
    let document = fixtures::with(
        &fixtures::worker_alone_document(),
        b"application.limits.overrides",
        br#"{"forges.installations":10,"forge_component.installations":10}"#,
    );
    let found = crate::startup::worker_part(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(
        &*lines(&found),
        b"workers.limits.application_value_secret: 8625 bytes, past its bound of 8192\n".as_slice()
    );
}
