//! Stock link size bounds (temper/docs/design/protocol/channel.md, 8).
use super::fixtures::{
    self,
    link_profiles::{Base, settings},
};
use crate::limits;
use jig_protocol_engine_workers as kit;
use jig_protocol_worker_wire::{self as wire, Kind};

#[test]
fn stock_profiles_make_both_links_and_fit_every_nonstart_frame() {
    for profile in [Base::Fixture, Base::Stock] {
        let settings = settings(profile);
        let engine = crate::workers::limits(&settings).expect("stock engine link limits");
        let mut worker = fixtures::worker_settings();
        worker.jig.limits = Some(settings.jig.limits);
        let payload = crate::worker_settings::payload_limits(&limits::worker(&settings.limits)).unwrap();
        let worker = crate::worker_settings::link_limits(&worker.jig, &payload, settings.limits.forge.forges)
            .expect("stock worker link limits");
        assert_eq!(engine.room, settings.jig.limits.io.output);
        assert_eq!(worker.room, settings.jig.limits.io.output);
        assert!(kit::valid(&engine), "{profile:?}");
        assert!(jig_protocol_worker_link::valid(&worker), "{profile:?}");
        for rule in wire::KINDS {
            let start = match rule.body {
                Kind::Assignment | Kind::Job | Kind::Figures => true,
                Kind::Hello
                | Kind::Turn
                | Kind::Call
                | Kind::Answer
                | Kind::Fact
                | Kind::Unauthorized
                | Kind::Usage
                | Kind::UsageRead
                | Kind::ReadUsage
                | Kind::Limited
                | Kind::AccountRefused
                | Kind::Message
                | Kind::Relayed
                | Kind::TurnKept
                | Kind::TurnBusy
                | Kind::Acknowledge
                | Kind::Cancel
                | Kind::Grant
                | Kind::Sources
                | Kind::Check
                | Kind::ReadList
                | Kind::ListRead
                | Kind::Draining
                | Kind::SignOut
                | Kind::Logins
                | Kind::SignedOut
                | Kind::JobAnswer
                | Kind::Ask
                | Kind::Restarting
                | Kind::Asked
                | Kind::Wires => false,
            };
            if start {
                continue;
            }
            let (sizes, room) = match rule.direction {
                skein_channel::Direction::FromResponder => (&engine.sizes, engine.room),
                skein_channel::Direction::FromInitiator => (&worker.sizes, worker.room),
            };
            let bytes = wire::sizes::largest_body(sizes, rule.body).unwrap().checked_add(8).unwrap();
            assert!(bytes <= room, "{profile:?} {:?}: {bytes} > {room}", rule.body);
        }
    }
}

#[test]
fn the_checkout_and_its_raise_fit_both_stock_links() {
    for (profile, conflicts, saved, answer) in [
        (Base::Fixture, 512, 265_248, 269_737),
        (Base::Fixture, 1024, 530_464, 534_953),
        (Base::Stock, 512, 265_248, 397_208),
        (Base::Stock, 1024, 530_464, 662_424),
    ] {
        let mut settings = settings(profile);
        assert_eq!(settings.limits.checkout.conflicts, 512);
        settings.limits.checkout.conflicts = conflicts;
        let engine = crate::workers::limits(&settings).expect("engine link at checkout capacity");
        let mut worker = fixtures::worker_settings();
        worker.jig.limits = Some(settings.jig.limits);
        let payload = crate::worker_settings::payload_limits(&limits::worker(&settings.limits)).unwrap();
        let worker = crate::worker_settings::link_limits(&worker.jig, &payload, settings.limits.forge.forges).unwrap();
        assert!(kit::valid(&engine), "{profile:?}, {conflicts}");
        assert!(jig_protocol_worker_link::valid(&worker), "{profile:?}, {conflicts}");
        for (sizes, room) in [(engine.sizes, engine.room), (worker.sizes, worker.room)] {
            assert_eq!(sizes.saved, saved, "{profile:?}, {conflicts}");
            assert_eq!(
                wire::sizes::largest_body(&sizes, Kind::Answer).unwrap() + 8,
                answer,
                "{profile:?}, {conflicts}"
            );
            assert_eq!(room, 1_048_576);
            assert!(answer <= room);
        }
    }
}
