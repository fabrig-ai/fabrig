//! Both peers admit the worker's ask envelope; answers take each side's bounds (protocol/channel.md, 8).
use super::fixtures;
use temper_protocol_worker_wire as payload;

#[test]
fn ask_sizes_take_the_worker_ceiling_and_reds_answers_take_each_sides_bounds() {
    // Answer takes its largest arm. When Reds is smaller, Unadmitted sets the bound:
    // worker-wire-v1.schema defines 512 bytes of words, plus a 4-byte length and 1-byte Answer tag.
    const UNADMITTED_ANSWER_BYTES: u32 = 517;
    let mut settings = fixtures::engine();
    let worker = fixtures::worker_settings();
    let agent = fixtures::worker_agent();
    for (count, key, answer) in [(64, 256, 18_949), (3, 19, UNADMITTED_ANSWER_BYTES)] {
        settings.limits.forge.branch_reds = count;
        settings.limits.forge.red_key_bytes = key;
        let engine = crate::workers::limits(&settings).unwrap();
        assert_eq!((engine.sizes.application_ask, engine.sizes.application_answer), (305_383, answer));
        let limits = crate::worker_settings::payload_limits(&crate::limits::worker(&settings.limits)).unwrap();
        assert_eq!((limits.reds_reds, limits.red_key), (256, 512));
        let link = crate::worker_settings::link_limits(&worker.jig, &limits, settings.limits.forge.forges).unwrap();
        assert_eq!((link.sizes.application_ask, link.sizes.application_answer), (305_383, 141_317));
        let root = crate::worker_settings::root_limits(&worker, &agent, &crate::limits::worker(&settings.limits));
        assert_eq!((root.host.application_ask_bytes, root.host.application_answer_bytes), (305_383, 141_317));
    }
}

#[test]
fn red_payload_ceilings_refuse_startup_with_the_forge_limit_named() {
    let own = temper_limits_fixtures::small();
    let normal = crate::limits::worker(&own).bounds();
    for (bounds, name) in [
        (payload::Bounds { reds: 257, ..normal }, b"forge.branch_reds".as_slice()),
        (payload::Bounds { red_key_bytes: 513, ..normal }, b"forge.red_key_bytes"),
    ] {
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::channel(Some(&bounds), None, None, &own, &mut problems);
        assert_eq!(problems.problems.len(), 1);
        assert_eq!(
            problems.problems.get(0).unwrap().path,
            jig_protocol_config::Path::root()
                .member(b"application")
                .member(b"limits")
                .member(b"overrides")
                .member(name)
        );
    }
}
