//! Orders beside the readers (hearth/docs/design/domain/render/README.md, 5.5).
use hearth_mmdebstrap_world::{
    fixtures::LIMITS,
    generate,
    world::{self, End, Step},
};
use hearth_protocol_mmdebstrap::{output::Seen, packages::Found};
use skein_lib::Rng;

fn pipes(mut check: impl FnMut(&[Step<Seen>])) {
    for seed in 0..16 {
        let mut rng = Rng::new(0x0704_0000 + seed);
        let bytes = generate::output(&mut rng, &LIMITS);
        for end in [End::Whole, world::end(&mut rng, &bytes)] {
            check(&world::output_steps(&bytes, seed, &LIMITS, end));
        }
    }
}

fn lists(mut check: impl FnMut(&[Step<Found>])) {
    for seed in 0..16 {
        let mut rng = Rng::new(seed);
        let bytes = generate::index(&mut rng, &LIMITS);
        for end in [End::Whole, world::end(&mut rng, &bytes)] {
            check(&world::index_steps(&bytes, seed, &LIMITS, end));
        }
    }
}

#[test]
fn a_pipe_reader_has_one_scan_out_whatever_the_cuts() {
    pipes(|steps| {
        for step in steps {
            assert!(step.below.len() <= 1, "at most one scan out: {step:?}");
        }
    });
}

#[test]
fn a_delivery_gives_a_pipe_s_caller_at_most_one_thing() {
    pipes(|steps| {
        for step in steps {
            assert!(step.above.len() <= 1, "at most one thing a delivery: {step:?}");
        }
    });
}

fn ends_once_and_last<T: std::fmt::Debug>(steps: &[Step<T>], terminal: impl Fn(&T) -> bool) {
    let items: Vec<_> = steps.iter().flat_map(|step| &step.above).collect();
    assert_eq!(items.iter().filter(|item| terminal(item)).count(), 1, "one terminal event: {steps:?}");
    assert!(terminal(items.last().expect("terminal event")), "terminal event comes last: {steps:?}");
    for step in steps.iter().rev().take(3) {
        assert!(step.above.is_empty() && step.below.is_empty(), "strays emit nothing: {step:?}");
    }
}

#[test]
fn a_pipe_ends_once_and_last() {
    pipes(|steps| ends_once_and_last(steps, |seen| matches!(seen, Seen::Ended | Seen::Failed(_))));
}

#[test]
fn a_list_reader_has_one_scan_out_whatever_the_cuts() {
    lists(|steps| {
        for step in steps {
            assert!(step.below.len() <= 1, "at most one scan out: {step:?}");
        }
    });
}

#[test]
fn a_delivery_gives_a_list_s_caller_at_most_one_stanza() {
    lists(|steps| {
        for step in steps {
            let stanzas = step.above.iter().filter(|found| matches!(found, Found::Stanza(_))).count();
            assert!(stanzas <= 1, "at most one stanza a delivery: {step:?}");
        }
    });
}

#[test]
fn a_list_s_end_follows_its_last_stanza_and_comes_once() {
    lists(|steps| ends_once_and_last(steps, |found| matches!(found, Found::Ended | Found::Failed(_))));
    let stanza = "Version: 1\nArchitecture: amd64\nFilename: pool/p.deb\nSize: 1\nSHA256: ";
    let digest = "0".repeat(64);
    let bytes = format!("Package: first\n{stanza}{digest}\n\nPackage: last\n{stanza}{digest}\n");
    for seed in 0..8 {
        let steps = world::index_steps(bytes.as_bytes(), seed, &LIMITS, End::Whole);
        ends_once_and_last(&steps, |found| matches!(found, Found::Ended | Found::Failed(_)));
        let items: Vec<_> = steps.iter().flat_map(|step| &step.above).collect();
        assert!(
            matches!(items.as_slice(), [Found::Stanza(_), Found::Stanza(_), Found::Ended]),
            "seed {seed}: {items:?}"
        );
        let end = &steps[steps.len() - 4];
        assert!(matches!(end.above.as_slice(), [Found::Stanza(_), Found::Ended]), "end's stanza then end: {end:?}");
    }
}
