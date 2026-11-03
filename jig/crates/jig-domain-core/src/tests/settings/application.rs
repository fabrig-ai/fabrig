//! Declaring application kinds has no startup behavior (jig/docs/design/domain/engine.md, 17.5).
use crate::settings::{Kind, KindScope};
use crate::tests::*;

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "ordinary unit proof compares complete Debug outputs because Request does not derive equality"
)]
fn declaring_an_application_kind_preserves_start_outputs() {
    for seed in [0, 1, u64::MAX] {
        let limits = limits();
        let configured = Config {
            kinds: Box::new([Kind {
                number: 1,
                scope: KindScope::Project,
                connector: 2,
                value_bytes: 64,
                per_scope: 4,
                charters: 1,
            }]),
            ..config(seed)
        };
        let mut declared = Core::new(configured, &limits).unwrap();
        let mut empty = Core::new(config(seed), &limits).unwrap();
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let declared_outputs = finish_start(&mut declared, &env);
        let empty_outputs = finish_start(&mut empty, &env);
        assert!(!empty_outputs.is_empty(), "the start emits its writes and asks");
        assert_eq!(format!("{declared_outputs:?}"), format!("{empty_outputs:?}"), "seed {seed}");
    }
}
