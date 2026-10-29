//! Failed Keep replies name their own step (skein/docs/design/ui-web.md, 14.2).
use super::{Play, Script, Step, step_failure};
use crate::tests::face;
use skein_browser::boundary::{Event, Read, Refusal, Request};
use skein_lib::Time;

#[test]
fn a_failed_keep_reports_its_own_step() {
    for (reply, why) in [
        (Ok(Read::NoElement), "no element"),
        (Ok(Read::NoAttribute), "no attribute"),
        (Err(Refusal::Protocol), "browser refused read: Protocol"),
    ] {
        let mut script = Script::new(Box::new([
            Step::StopServer,
            Step::Keep { selector: b"#hidden".as_slice().into(), attribute: None, name: b"read".as_slice().into() },
        ]));
        let mut face = face();
        script.next(&mut face, Time::ZERO);
        script.next(&mut face, Time::ZERO);
        let Request::Attribute { op, .. } = face.requests().next().expect("Keep's read") else {
            panic!("Keep asks for an attribute")
        };
        face.event(&match reply {
            Ok(read) => Event::Attribute { op, read },
            Err(why) => Event::Refused { op, why },
        });
        assert_eq!(
            step_failure(&mut script, &mut face, Time::ZERO),
            Some((1, format!("keep selector #hidden attribute value: {why}"))),
        );
    }
}
