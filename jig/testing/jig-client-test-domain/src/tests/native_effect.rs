use crate::family::{self, Kind, Scope, Subject};
use jig_client::contract::{ApplicationPart, Part, PartKey, PartKind, ProposalKey, Proposer};
use skein_lib::List;

#[test]
fn a_native_effect_reference_keeps_its_connector_number_and_separate_namespace() {
    for connector in [0, crate::FIRST, crate::SECOND, u16::MAX] {
        for proposal in [0, u64::MAX] {
            let native = Part { connector, kind: PartKind::Effect, key: PartKey::Number(proposal) };
            let part = family::owner_effect(&native).unwrap();
            assert_eq!(part, ApplicationPart::OwnerEffect { connector, proposal });
            assert_eq!(crate::parts::connector(&part), connector);
            for scope in [Scope::Project(0), Scope::Project(u32::MAX), Scope::Deployment] {
                let subject = family::subject(&part, scope).unwrap();
                assert_eq!(subject, Subject { scope, kind: Kind::OwnerEffect(proposal) });
                for proposer in [Proposer::Task(0), Proposer::Party(u64::MAX)] {
                    let key = ProposalKey { proposer, number: proposal };
                    assert_ne!(
                        subject,
                        family::subject(&ApplicationPart::Effect { connector, proposal: key }, scope).unwrap()
                    );
                }
            }
            assert_eq!(family::owner_hold(&native), None);
        }
    }
    for kind in [
        PartKind::Effect,
        PartKind::Hold,
        PartKind::Resource,
        PartKind::Pool,
        PartKind::Result,
        PartKind::Procedure,
        PartKind::Outbox,
    ] {
        let mut path = List::with_capacity(2);
        path.push(Box::<[u8]>::from(*b"number/17")).unwrap();
        path.push(Box::<[u8]>::from([255, 0])).unwrap();
        assert_eq!(family::owner_effect(&Part { connector: crate::FIRST, kind, key: PartKey::Path(path) }), None);
        if kind != PartKind::Effect {
            assert_eq!(family::owner_effect(&Part { connector: crate::FIRST, kind, key: PartKey::Number(17) }), None);
        }
    }
}
