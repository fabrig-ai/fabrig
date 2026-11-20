use super::*;
use crate::settings::{Changed, Default, Head};
use skein_lib::{Map, Wall};
fn provider(number: u32, name: &[u8], endpoint: u32) -> ProviderSetting {
    ProviderSetting {
        provider: Provider {
            number,
            name: name.into(),
            endpoint: Some(name.into()),
            direct: None,
            list: None,
            signed_in_plan: false,
            manage_usage: None,
            sign_in: None,
            keys: false,
            programs: Box::new([]),
        },
        endpoints: Box::new([Endpoint {
            number: endpoint,
            name: name.into(),
            provider: name.into(),
            dialect: 3,
            program: None,
            address: EndpointAddress::Socket(core::net::SocketAddr::from(([127, 0, 0, 1], 8080))),
            transport: Transport::Plaintext,
            headers: Box::new([]),
            connect: Duration::from_millis(10_000),
            handshake: Duration::from_millis(10_000),
        }]),
        withdrawn: false,
    }
}
fn current() -> Current {
    let mut defaults = Map::with_capacity(PROVIDERS);
    defaults
        .insert(
            Setting::Provider(1),
            Default { value: super::super::Value::Provider(Box::new(provider(1, b"openai", 1))), revision: 1 },
        )
        .unwrap();
    Current {
        fixed_providers: Box::new([]),
        resolved: Map::with_capacity(0),
        defaults,
        changes: Map::with_capacity(PROVIDERS),
        applications: Map::with_capacity(0),
        head: Head {
            number: 1,
            against: 0,
            at: Wall::EPOCH,
            by: super::super::Decider::FirstStart,
            proposal: None,
            moved: false,
        },
        oldest: 1,
        keys: 1,
    }
}
fn item(value: ProviderSetting) -> Item {
    Item {
        setting: Setting::Provider(value.provider.number),
        edit: Edit::Set(super::super::Value::Provider(Box::new(value))),
    }
}
#[test]
fn provider_shape_count_and_stable_identities() {
    let mut current = current();
    let local = provider(4, b"local", 5);
    assert!(fits(&local));
    assert_eq!(check_change(&current, &[item(local.clone())]), Ok(()));
    assert_eq!(check_set(&current, false, &[item(local.clone())]), Ok(()));
    for (value, fault) in [
        (provider(1, b"other", 5), Fault::Shape),
        (provider(17, b"local", 5), Fault::Count),
        (ProviderSetting { withdrawn: true, ..local.clone() }, Fault::Shape),
    ] {
        let number = value.provider.number;
        assert_eq!(check_change(&current, &[item(value)]), Err(failure(Setting::Provider(number), fault)));
    }
    let mut header = local.clone();
    header.endpoints.first_mut().unwrap().headers =
        Box::new([Header { name: b"aUtHoRiZaTiOn".as_slice().into(), value: b"not-a-credential".as_slice().into() }]);
    assert!(!fits(&header));
    let mut duplicate = local.clone();
    duplicate.endpoints.first_mut().unwrap().number = 1;
    assert_eq!(check_set(&current, false, &[item(duplicate)]), Err(failure(Setting::Provider(4), Fault::Shape)));
    current
        .changes
        .insert(
            Setting::Provider(4),
            Changed {
                value: Some(super::super::Value::Provider(Box::new(ProviderSetting {
                    withdrawn: true,
                    ..local.clone()
                }))),
                revision: 2,
            },
        )
        .unwrap();
    assert_eq!(
        check_change(&current, &[item(provider(4, b"other", 5))]),
        Err(failure(Setting::Provider(4), Fault::Shape))
    );
    assert_eq!(
        check_change(&current, &[Item { setting: Setting::Provider(4), edit: Edit::Reset }]),
        Err(failure(Setting::Provider(4), Fault::Shape))
    );
    assert_eq!(check_change(&current, &[item(local)]), Ok(()));
    current.fixed_providers = Box::new([provider(3, b"fixed", 3)]);
    assert_eq!(
        check_change(&current, &[item(provider(3, b"fixed", 3))]),
        Err(failure(Setting::Provider(3), Fault::Shape))
    );
    let mut model = crate::Model::hole();
    model.name = b"model".as_slice().into();
    model.provider = 1;
    model.endpoint = 1;
    model.dialect = 3;
    model.window = 1;
    model.output = 1;
    model.price_unit = 1;
    let mut model = super::super::CatalogueModel {
        number: 2,
        model,
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: super::super::ModelSource::Person,
        estimated: false,
        first_read: None,
    };
    let removed = [Item { setting: Setting::Provider(1), edit: Edit::Removed }];
    assert_eq!(super::super::catalogue::shape(&model, 2, &overlay(&current, false, &[])), Ok(()));
    assert_eq!(super::super::catalogue::shape(&model, 2, &overlay(&current, false, &removed)), Err(Fault::Reference));
    model.model.provider = 3;
    model.model.endpoint = 3;
    assert_eq!(super::super::catalogue::shape(&model, 2, &overlay(&current, false, &removed)), Ok(()));
}
