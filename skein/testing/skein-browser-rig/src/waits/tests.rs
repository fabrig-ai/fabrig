use super::{Decision, End, Finding};
use skein_browser::boundary::{Event, Refusal};
use skein_io::kernel::Exit;
use skein_lib::{Duration, List, Time, Token};
use skein_world::Figures;

fn figures(patience: Duration) -> Figures {
    Figures {
        what: "the wait under test",
        patience,
        elapsed: patience,
        thread_waited: None,
        thread_cpu: None,
        tree_waited: None,
    }
}

#[test]
fn only_an_unanswered_browser_wait_can_end_as_starved() {
    let op = Token::new(16);
    let missed = Event::Missed { op, seen: List::with_capacity(0), more: 2 };
    let timeout = Event::Refused { op, why: Refusal::Timeout, detail: Box::default() };
    for (event, wait, patience) in
        [(&missed, "missed; saw", Duration::from_millis(500)), (&timeout, ": Timeout", Duration::from_secs(10))]
    {
        assert!(matches!(End::Browser(event).finding(), Finding::Unanswered { op: Some(found), .. } if found == op));
        assert_eq!(End::Browser(event).finding().decide(None), Decision::EndAndPass);
        let figures = figures(patience);
        let Decision::EndAndFail(report) = End::Browser(event).finding().decide(Some(&figures)) else {
            panic!("an idle expiry must fail after cleanup");
        };
        assert!(report.contains(wait), "{report}");
        assert!(report.ends_with(&figures.to_string()), "{report}");
    }
}

#[test]
fn the_tab_bound_expires_with_or_without_a_reply() {
    let before = Time::ZERO;
    let patience = Duration::from_secs(3);
    let until = before.saturating_add(patience);
    let figures = figures(patience);
    for last in [None, Some("stage Pressing: 0 tabs")] {
        assert_eq!(End::Tabs { now: before, until, last }.finding(), Finding::Observe);
        assert_eq!(End::Tabs { now: until, until, last }.finding().decide(None), Decision::EndAndPass);
        assert_eq!(
            End::Tabs { now: until, until, last }.finding().decide(Some(&figures)),
            Decision::EndAndFail(format!("tabs did not settle; {}; {figures}", last.unwrap_or("no reply")))
        );
    }
}

#[test]
fn failures_are_never_starvation_verdicts() {
    let op = Token::new(16);
    let looked_wrong = Event::Found { op, seen: List::with_capacity(0), more: 0 };
    let refused = Event::Refused { op, why: Refusal::Protocol, detail: Box::default() };
    let figures = figures(Duration::from_secs(1));
    for (end, report) in [
        (End::Unexpected(&looked_wrong), format!("unexpected browser event: {looked_wrong:?}")),
        (End::Unexpected(&refused), format!("browser refused operation {op:?}: Protocol")),
        (End::Exit(Exit::Code(1)), String::from("Chromium exited with Code(1)")),
    ] {
        assert_eq!(end.finding().decide(None), Decision::Fail(report));
    }
    for reply in [&looked_wrong, &refused] {
        assert_eq!(End::Browser(reply).finding().decide(Some(&figures)), Decision::Observe);
    }
}
