//! Optional push startup and deployment read (attention.md, 7.3).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests edit configuration fixtures"
)]
use crate::startup::{self, application};
use crate::tests::fixtures;
use jig_protocol_config as config;
use skein_lib::{List, Secret};
const PEM: &[u8] = include_bytes!("../../../../tests/one-process/src/fixtures/vapid.pem");
fn input(push: serde_json::Value) -> Box<[u8]> {
    let mut input: serde_json::Value = serde_json::from_slice(&fixtures::document()).unwrap();
    input["application"]["push"] = push;
    serde_json::to_vec(&input).unwrap().into_boxed_slice()
}
fn configured() -> serde_json::Value {
    serde_json::json!({"key":"push.key","subject":"mailto:operator@example.test","services":null})
}
#[test]
fn optional_push_reads_its_private_key_services_and_public_half() {
    for input in [fixtures::document(), input(serde_json::Value::Null)] {
        let parsed = startup::parse(&input, &config::Directories::working()).unwrap();
        assert!(parsed.application.push.is_none());
    }
    let document = input(configured());
    let parsed = startup::parse(&document, &config::Directories::working()).unwrap();
    let push = parsed.application.push.as_ref().unwrap();
    assert_eq!(push.subject.as_ref(), b"mailto:operator@example.test");
    assert_eq!(
        push.services.as_ref(),
        [
            Box::from(b"https://fcm.googleapis.com".as_slice()),
            Box::from(b"https://updates.push.services.mozilla.com".as_slice()),
            Box::from(b"https://web.push.apple.com".as_slice())
        ]
    );
    let files = application::files(&parsed.application);
    let file = files.iter().find(|file| file.path.as_ref() == b"push.key").unwrap();
    assert_eq!(file.max, u32::try_from(skein_push::VAPID_PEM_BYTES).unwrap());
    assert!(file.private);
    assert_eq!(file.at, config::Path::root().member(b"application").member(b"push").member(b"key"));
    let mut files = crate::settings::Files { entries: List::with_capacity(1) };
    files.entries.push((b"push.key".as_slice().into(), Secret::new(PEM.into()))).unwrap();
    let mut problems = startup::Problems::empty();
    let resolved = application::resolve(&parsed.application, &files, &mut problems).unwrap();
    assert!(problems.is_empty());
    assert_eq!(resolved.push.unwrap().key.public(), skein_push::Vapid::from_pkcs8_pem(PEM).unwrap().public());
    let mut explicit = configured();
    explicit["subject"] = "https://operator.example.test".into();
    explicit["services"] =
        serde_json::json!(["https://push.example.test:8443", "http://127.0.0.1:31999", "http://localhost:31999"]);
    let parsed = startup::parse(&input(explicit), &config::Directories::working()).unwrap();
    assert_eq!(parsed.application.push.unwrap().services.len(), 3);
    let mut bounded = configured();
    bounded["subject"] = format!("mailto:{}", "a".repeat(skein_push::SUBJECT_BYTES - 7)).into();
    bounded["services"] = serde_json::json!(vec!["https://push.example.test"; 8]);
    let parsed = startup::parse(&input(bounded), &config::Directories::working()).unwrap();
    let mut pem = PEM.to_vec();
    pem.resize(skein_push::VAPID_PEM_BYTES, b'\n');
    let mut files = crate::settings::Files { entries: List::with_capacity(1) };
    files.entries.push((b"push.key".as_slice().into(), Secret::new(pem.into_boxed_slice()))).unwrap();
    let resolved = application::resolve(&parsed.application, &files, &mut problems).unwrap();
    assert!(problems.is_empty());
    assert_eq!(resolved.push.unwrap().services.len(), 8);
}
#[test]
fn push_refusals_name_each_member_and_mail_stays_unknown() {
    for (member, value) in [
        ("subject", serde_json::json!("ftp:operator")),
        ("subject", serde_json::json!("mailto:has space")),
        ("subject", serde_json::json!(format!("mailto:{}", "a".repeat(skein_push::SUBJECT_BYTES)))),
        ("services", serde_json::json!([])),
        ("services", serde_json::json!(vec!["https://push.example.test"; 9])),
        ("services", serde_json::json!(["https://push.example.test/path"])),
        ("services", serde_json::json!(["http://push.example.test"])),
        ("services", serde_json::json!(["https://user@push.example.test"])),
    ] {
        let mut push = configured();
        push[member] = value;
        let Err(problems) = startup::parse(&input(push), &config::Directories::working()) else {
            panic!("invalid push accepted: {member}");
        };
        assert!(String::from_utf8_lossy(&startup::lines(&problems)).contains(&format!("application.push.{member}")));
    }
    let parsed = startup::parse(&input(configured()), &config::Directories::working()).unwrap();
    for value in [None, Some(Vec::from(b"not a key".as_slice())), Some(vec![b'a'; skein_push::VAPID_PEM_BYTES + 1])] {
        let mut files = crate::settings::Files { entries: List::with_capacity(1) };
        if let Some(value) = value {
            files.entries.push((b"push.key".as_slice().into(), Secret::new(value.into_boxed_slice()))).unwrap();
        }
        let mut problems = startup::Problems::empty();
        assert!(application::resolve(&parsed.application, &files, &mut problems).is_none());
        assert!(String::from_utf8_lossy(&startup::lines(&problems)).contains("application.push.key"));
    }
    let mut document: serde_json::Value = serde_json::from_slice(&fixtures::document()).unwrap();
    document["application"]["mail"] = serde_json::Value::Null;
    let Err(problems) = startup::parse(&serde_json::to_vec(&document).unwrap(), &config::Directories::working()) else {
        panic!("mail must stay unknown");
    };
    assert!(String::from_utf8_lossy(&startup::lines(&problems)).contains("application.mail: unknown member"));
}
