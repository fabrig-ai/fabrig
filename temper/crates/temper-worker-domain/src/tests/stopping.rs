//! The two independently armed stop clocks never extend one another.
use super::harness::Harness;
use crate::{Event, Request, agent, fire};
use skein_lib::{Duration, Time};

#[test]
fn the_earlier_of_shutdown_and_contact_bounds_holds_without_resetting_either() {
    for (stop, rest, first_abandons) in [(3, 8, true), (8, 3, false)] {
        let mut h = Harness::new();
        h.started();
        h.env.limits.stop_bound = Duration::from_secs(stop);
        h.env.limits.drain = Duration::from_secs(20);
        h.env.limits.after_grace = Duration::from_secs(rest);
        h.env.limits.agent.cancel_grace = Duration::from_secs(60);
        h.run(Event::Shutdown { drain: true });
        h.env.now = Time::from_nanos(1_000_000_000);
        h.run(Event::GracePassed);
        // A repeated notification, and a new channel, extend neither clock.
        h.run(Event::Ready);
        h.run(Event::Shutdown { drain: true });
        h.run(Event::GracePassed);
        let at = if first_abandons { stop } else { rest + 1 };
        h.env.now = Time::from_nanos(at * 1_000_000_000 - 1);
        fire(&mut h.domain, &h.env, &mut h.out);
        let rows = h.run(Event::Lost);
        assert!(!rows.contains(&Request::Abandon));
        assert!(!dropped(&rows));
        h.env.now = Time::from_nanos(at * 1_000_000_000);
        fire(&mut h.domain, &h.env, &mut h.out);
        let rows = h.run(Event::Lost);
        assert_eq!(rows.contains(&Request::Abandon), first_abandons, "stop={stop}, rest={rest}");
        assert_eq!(dropped(&rows), !first_abandons, "stop={stop}, rest={rest}");
    }
}

fn dropped(rows: &[Request]) -> bool {
    for row in rows {
        if let Request::Process(request) = row
            && let agent::process::Request::Signal { signal: agent::Signal::Terminate, .. } = request.as_ref()
        {
            return true;
        }
    }
    false
}
