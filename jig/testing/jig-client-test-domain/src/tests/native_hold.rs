use crate::family;
use jig_client::contract::{ApplicationPart, Part, PartKey, PartKind};
use skein_lib::List;

#[test]
fn a_native_hold_account_uses_only_its_supplied_connector_and_number() {
    for connector in [0, crate::FIRST, crate::SECOND, u16::MAX] {
        for task in [0, u64::MAX] {
            let native = Part { connector, kind: PartKind::Hold, key: PartKey::Number(task) };
            assert_eq!(family::owner_hold(&native), Some(ApplicationPart::Hold { connector, task }));
        }
    }
    for kind in [
        PartKind::Hold,
        PartKind::Resource,
        PartKind::Pool,
        PartKind::Result,
        PartKind::Procedure,
        PartKind::Effect,
        PartKind::Outbox,
    ] {
        let mut path = List::with_capacity(2);
        path.push(Box::<[u8]>::from(*b"same/number")).unwrap();
        path.push(Box::<[u8]>::from([255, 0])).unwrap();
        assert_eq!(family::owner_hold(&Part { connector: crate::FIRST, kind, key: PartKey::Path(path) }), None);
        if kind != PartKind::Hold {
            assert_eq!(family::owner_hold(&Part { connector: crate::FIRST, kind, key: PartKey::Number(7) }), None);
        }
    }
}
