use super::{PROJECT, REQUESTER, frame, list, text};
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
        let mut row = frame(REQUESTER);
        row.projects.get_mut(0).expect("fixture project").role = role;
        row.projects.get_mut(0).expect("fixture project").pool = None;
        let mut truth = crate::Truth::default();
        truth.frames.insert(REQUESTER, row.clone());
        assert!(truth.visible(REQUESTER, PROJECT));
        assert!(!truth.visible(REQUESTER, 2));
        assert_eq!(
            crate::requests::recipient(&truth, REQUESTER, &Recipient::Role { project: PROJECT, role: u32::MAX }),
            role == HeldRole::Policy { role: u32::MAX }
        );
        assert!(!crate::requests::recipient(&truth, REQUESTER, &Recipient::Role { project: PROJECT, role: 1 }));
        let request = Ask::SetRoles { project: u32::MAX, holdings: list(vec![Holding { party: u64::MAX, role }]) };
        assert!(crate::codec::ask(&request, &Bounds::default()).is_ok());
        let service = Ask::MakeService { project: u32::MAX, name: text(b"actual-service"), role };
        assert!(crate::codec::ask(&service, &Bounds::default()).is_ok());
        row.projects.get_mut(0).expect("fixture project").pool = Some(PoolLedger {
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

#[test]
fn model_new_until_survives_the_independent_frame_encoding() {
    for new_until in [None, Some(Wall::from_nanos(u64::MAX))] {
        let mut value = frame(REQUESTER);
        value.projects.get_mut(0).unwrap().models = list(vec![MeModel {
            new_until,
            number: 3,
            source: ModelSource::List,
            estimated: true,
            name: text(b"listed"),
            provider: Some(7),
            window: 8192,
            output: 1024,
            efforts: list(vec![]),
            effort: None,
            prices: ModelPrices { input: 3, cached: 1, cache_write: None, output: 7, unit: 1000 },
            description: None,
            shown: None,
            accounts: list(vec![42]),
            picker: true,
        }]);
        let decoded = crate::codec::read(&ReadValue::Me(Box::new(value.clone())), &Bounds::default()).unwrap();
        assert_eq!(decoded, ReadValue::Me(Box::new(value)));
    }
}
