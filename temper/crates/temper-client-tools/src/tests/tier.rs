use super::fixtures as f;
use jig_client::{self as client, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};
fn home() -> c::ReadResult {
    let mut project = super::jig::projects::project();
    project.number = f::PROJECT;
    let mut path = List::with_capacity(4);
    for part in [b"forge".as_slice(), b"forge.example", b"acme", b"site"] {
        path.push(part.into()).unwrap();
    }
    project.resources.get_mut(0).unwrap().part = c::Part {
        connector: wire::number(wire::Owner::Forge),
        kind: c::PartKind::Resource,
        key: c::PartKey::Path(path),
    };
    c::ReadResult::Value(c::ReadValue::Project(Box::new(project)))
}
#[test]
fn tier_commands_send_keyed_home_or_named_requests_and_print_every_outcome() {
    for (verb, success) in [("hold", "held: browser of acme/site\n"), ("release", "released: browser of acme/site\n")] {
        for named in [false, true] {
            for (refusal, expected) in [
                (None, success),
                (Some(w::TierRefusal::NoRepository), "no repository acme/site\n"),
                (Some(w::TierRefusal::NoTier), "no tier browser on acme/site\n"),
                (
                    Some(w::TierRefusal::Held(
                        w::TierHeldBy::new(
                            &w::CEILINGS,
                            w::TierHeldByParts { party: 19, reason: b"maintenance".as_slice().into() },
                        )
                        .unwrap(),
                    )),
                    "browser of acme/site is held already by party 19: maintenance\n",
                ),
                (Some(w::TierRefusal::NotHeld), "browser of acme/site is not held\n"),
                (Some(w::TierRefusal::Words), "reason must be 1 to 1024 bytes of UTF-8\n"),
            ] {
                let mut root = f::signed_in(f::PARTY);
                let mut words = List::with_capacity(14);
                for word in ["tier", verb, "--project", "shop", "browser", "--key", "tier-act"] {
                    words.push(word).unwrap();
                }
                if named {
                    for word in ["--owner", "acme", "--name", "site"] {
                        words.push(word).unwrap();
                    }
                }
                if verb == "hold" {
                    words.push("--reason").unwrap();
                    words.push("maintenance").unwrap();
                }
                let call = root.submit(words.as_slice());
                let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("tier family") };
                assert_eq!(*scope, c::Scope::Project(f::PROJECT));
                assert_eq!(record.version, 22);
                assert!(root.sent_name(call).is_some());
                let w::Request::TierAct(request) =
                    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
                else {
                    panic!("tier request")
                };
                assert_eq!((request.project(), request.tier()), (f::PROJECT, b"browser".as_slice()));
                assert_eq!(request.repository().is_some(), named);
                if let Some(repository) = request.repository() {
                    assert_eq!((repository.owner(), repository.name()), (b"acme".as_slice(), b"site".as_slice()));
                }
                match request.act() {
                    w::TierAct::Hold(value) => {
                        assert_eq!(verb, "hold");
                        assert_eq!(value.reason(), b"maintenance");
                    }
                    w::TierAct::Release => assert_eq!(verb, "release"),
                }
                let refused = refusal.is_some();
                root.adoption_answer(
                    call,
                    w::Outcome::Tier(w::TierOutcome::new(&w::CEILINGS, w::TierOutcomeParts { refusal }).unwrap()),
                );
                if !named {
                    assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Project { project: f::PROJECT });
                    root.serve(call, home());
                }
                let printed = root.answer(call).unwrap();
                assert_eq!(printed.status, if refused { 2 } else { 0 });
                assert_eq!(if refused { printed.stderr } else { printed.stdout }.as_ref(), expected.as_bytes());
            }
        }
    }
}
