use crate::translate::settings;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::Duration;
#[test]
fn provider_identity_value_and_accounts_fault_cross_the_client_wire() {
    let limits = super::fixtures::limits();
    let provider = c::ProviderSetting {
        provider: c::Provider {
            number: 4,
            name: b"local".as_slice().into(),
            endpoint: Some(b"local".as_slice().into()),
            direct: Some(b"direct".as_slice().into()),
            list: Some(c::ProviderList {
                keep: Box::new([b"keep-*".as_slice().into()]),
                drop: Box::new([b"*-drop".as_slice().into()]),
            }),
            signed_in_plan: true,
            manage_usage: Some(b"https://usage.example/".as_slice().into()),
            keys: true,
            programs: Box::new([b"runner".as_slice().into()]),
            sign_in: Some(c::Registration {
                issuer: b"https://issuer.example".as_slice().into(),
                client: b"public-client".as_slice().into(),
                scopes: Box::new([b"chat".as_slice().into()]),
                name_hint: Some(b"hint".as_slice().into()),
                authorization: b"https://issuer.example/authorize".as_slice().into(),
                token: b"https://issuer.example/token".as_slice().into(),
                revocation: Some(b"https://issuer.example/revoke".as_slice().into()),
                keys: b"https://issuer.example/keys".as_slice().into(),
                endpoint_transport: c::Transport::Tls {
                    server_name: b"issuer.example".as_slice().into(),
                    trust: Some(b"issuer.pem".as_slice().into()),
                },
                must_grant: Box::new([b"chat".as_slice().into()]),
                resource: Some(b"https://resource.example".as_slice().into()),
                redirect_path: b"callback".as_slice().into(),
                redirect_port: Some(8081),
            }),
        },
        endpoints: Box::new([c::Endpoint {
            number: 5,
            name: b"local".as_slice().into(),
            provider: b"local".as_slice().into(),
            dialect: 3,
            program: Some(b"runner".as_slice().into()),
            address: c::EndpointAddress::Host { name: b"models.example".as_slice().into(), port: 443 },
            transport: c::Transport::Tls { server_name: b"models.example".as_slice().into(), trust: None },
            headers: Box::new([c::Header { name: b"X-Client".as_slice().into(), value: b"jig".as_slice().into() }]),
            connect: Duration::from_millis(10_000),
            handshake: Duration::from_millis(20_000),
        }]),
        withdrawn: true,
    };
    let value = c::SettingValue::Provider(Box::new(provider));
    let wire = settings::wire_setting_value(&value, limits.wire()).unwrap();
    let mut writer = skein_lib::Writer::new(skein_lib::usize_of(
        wire.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    wire.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    let wire = w::SettingValue::decode(limits.receiving(), &mut skein_lib::Reader::new(&writer.finish())).unwrap();
    assert_eq!(settings::restore_setting_value(&wire, &limits).unwrap(), value);
    let identity = c::Setting::Provider(c::SettingNumber { number: 4 });
    assert_eq!(settings::restore_setting(&settings::wire_setting(&identity)), identity);
    assert_eq!(settings::restore_setting_fault(&w::SettingFault::Accounts), c::SettingFault::Accounts);
}
