use super::{frame, list, text};
use crate::Bounds;
use jig_client_domain::contract::*;
use skein_lib::Wall;

#[test]
fn builtin_roles_remain_memberships_without_becoming_numeric_policy_recipients() {
    let roles = [
        HeldRole::Owner,
        HeldRole::Maintainer,
        HeldRole::Member,
        HeldRole::Observer,
        HeldRole::Policy { role: u32::MAX },
    ];
    for role in roles {
        let mut row = frame(1);
        row.projects.get_mut(0).unwrap().role = role;
        row.projects.get_mut(0).unwrap().pool = None;
        let mut truth = crate::Truth::default();
        truth.frames.insert(1, row.clone());
        assert!(truth.visible(1, 1));
        assert!(!truth.visible(1, 2));
        assert_eq!(
            crate::requests::recipient(&truth, 1, &Recipient::Role { project: 1, role: u32::MAX }),
            role == HeldRole::Policy { role: u32::MAX }
        );
        assert!(!crate::requests::recipient(&truth, 1, &Recipient::Role { project: 1, role: 1 }));
        let request = Ask::SetRoles { project: u32::MAX, holdings: list(vec![Holding { party: u64::MAX, role }]) };
        assert!(crate::codec::ask(&request, &Bounds::default()).is_ok());
        let service = Ask::MakeService { project: u32::MAX, name: text(b"actual-service"), role };
        assert!(crate::codec::ask(&service, &Bounds::default()).is_ok());
        row.projects.get_mut(0).unwrap().pool = Some(PoolLedger {
            source: Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
            parent: Some(Funder::Period { project: u32::MAX, period: u64::MAX }),
            numbers: Numbers { budget: u64::MAX, spent: u64::MAX, spent_below: u64::MAX, reserved: u64::MAX },
            left: 0,
            closed: true,
            began: Some(Wall::EPOCH),
            next: None,
        });
        assert!(crate::codec::read(&ReadValue::Me(Box::new(row)), &Bounds::default()).is_ok());
    }
}
