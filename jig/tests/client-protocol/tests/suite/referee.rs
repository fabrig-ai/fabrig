use jig_client_protocol_wire as wire;
use jig_client_protocol_world::referee::{Broken, Rules};
use skein_lib::{Reader, Token, Writer};

#[test]
fn a_record_altered_on_the_wire_is_caught() {
    let value = wire::LinkSignIn::new(
        &wire::CEILINGS,
        wire::LinkSignInParts { secret: skein_lib::Secret::new(Box::new([7; 32])) },
    )
    .expect("fixture record");
    let mut writer = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured record");
    let source = writer.finish();
    let mut actual = source.clone();
    *actual.last_mut().expect("nonempty record") = 8;
    assert!(wire::LinkSignIn::decode(&wire::CEILINGS, &mut Reader::new(&actual)).is_ok());
    assert_eq!(Rules::default().record(b"/jig/v1/sign-in/link", false, &source, &actual), Err(Broken::Record));
}

#[test]
fn a_token_leaked_into_a_domain_event_is_caught() {
    let secret = [73; 40];
    let event = jig_domain_core::Event::RunText {
        retry: 0,
        run: Token::new(1),
        attempt: Token::new(1),
        turn: 1,
        offset: 0,
        text: Box::new(secret),
        call: None,
    };
    let mut rules = Rules::default();
    rules.secret(&secret);
    assert_eq!(rules.domain(&event), Err(Broken::Secret));
}

#[test]
fn registered_secrets_are_found_in_event_fields_and_records() {
    let secrets: [&[u8]; 2] = [&[73; 40], &[0, 1, 7, 73, 128, 255]];
    let mut rules = Rules::default();
    for secret in secrets {
        rules.secret(secret);
        rules.secret(secret);
    }
    for secret in secrets {
        let event = jig_domain_core::Event::RunText {
            retry: 0,
            run: Token::new(1),
            attempt: Token::new(1),
            turn: 1,
            offset: 0,
            text: secret.into(),
            call: None,
        };
        let record = jig_client::Record { key: b"record".as_slice().into(), bytes: secret.into() };
        assert_eq!(rules.domain(&event), Err(Broken::Secret));
        assert_eq!(rules.domain(&record), Err(Broken::Secret));
        assert_eq!(rules.domain(&[0_u8, 1, 7, 73, 128, 254]), Ok(()));
        assert_eq!(rules.domain(&event), Err(Broken::Secret), "reuse after a clean event");
    }
    assert_eq!(rules.domains, 2, "only clean inputs count");
}
