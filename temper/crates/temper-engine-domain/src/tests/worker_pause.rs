//! The core's held pause follows the same link delivery route as a source check.
use super::resume_fixture::root;
use crate::{
    Decision, Delivery,
    decision::{self, JournalOutput},
};
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_link::PauseMode;
use skein_lib::{Queue, Token};

#[test]
fn a_held_pause_reaches_the_named_link_with_every_mode() {
    let channel = Token::new(7);
    for connected in [false, true] {
        for mode in [PauseMode::Resume, PauseMode::Pause, PauseMode::Drain] {
            let (mut domain, env) = root();
            if connected {
                domain.hub_link.connect(channel);
            }
            let mut change = Decision::decide(&mut domain.journal, &env.limits.journal);
            crate::route::held::held(&mut domain, &env, &mut change, core::Held::PauseWorker { channel, mode });
            decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, change, false)
                .unwrap();
            let mut out = Queue::uncapped();
            decision::resume(&mut domain.journal, &mut out);
            let Some(JournalOutput::Deliver(Delivery::Link {
                channel: named,
                down: host::LinkDown::Pause { mode: sent },
            })) = out.pop()
            else {
                panic!("pause delivered on the link: {out:?}")
            };
            assert_eq!((named, sent), (channel, mode));
            assert!(out.is_empty());
        }
    }
}
