#![expect(clippy::disallowed_types, reason = "ordinary tests build configuration documents")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests construct and inspect protocol fixtures"
)]
use super::fixtures;
use crate::startup::{self, Problems, application};
use jig_protocol_config as config;
const SIGN_IN: &str = r#"{"google":{"provider":3,"client":"temper-test","secret":"google.secret","endpoints":{"issuer":"http://127.0.0.1:31999","authorization":"http://127.0.0.1:31999/auth","token":"http://127.0.0.1:31999/token","keys":"http://127.0.0.1:31999/keys","transport":"plaintext"}},"members":[{"address":"Sam@example.com","login":"sam"}]}"#;
fn jig() -> config::Engine {
    let mut jig = fixtures::engine().jig;
    let mut providers = skein_lib::List::with_capacity(3);
    for provider in jig.providers {
        providers.push(provider).unwrap();
    }
    providers.push(config::Provider { number: 3, kind: config::ProviderKind::Application }).unwrap();
    jig.providers = providers.into_boxed();
    jig.owners[0].identity =
        jig_domain_people::IdentityKey { provider: 3, subject: b"Sam@example.com".as_slice().into() };
    jig
}
fn parse(input: &str, jig: &config::Engine) -> (Option<application::Members>, Problems) {
    let document =
        config::document(input.as_bytes(), &config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&document, 0, jig, &config::Directories::working(), &mut problems);
    (members, problems)
}
fn input(sign_in: &str) -> String {
    core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"sign_in\":null", &format!("\"sign_in\":{sign_in}"))
}
#[test]
fn a_google_registration_reads_its_members_and_private_file() {
    let (members, problems) = parse(&input(SIGN_IN), &jig());
    assert!(problems.is_empty(), "{:?}", startup::lines(&problems));
    let members = members.unwrap();
    let sign_in = members.sign_in.as_ref().unwrap();
    assert_eq!(sign_in.provider, 3);
    assert_eq!(sign_in.members[0].address.as_ref(), b"Sam@example.com");
    let files = application::files(&members, 4096, 4096);
    let secret = files.iter().find(|file| file.path.as_ref() == b"google.secret").unwrap();
    assert!(secret.private);
    assert_eq!(secret.max, 512);
}
#[test]
fn google_configuration_refusals_name_the_member() {
    for (from, to, expected) in [
        ("\"client\":\"temper-test\"", "\"client\":\"\"", "google.client"),
        ("\"client\":\"temper-test\"", "\"client\":\"has space\"", "google.client"),
        ("Sam@example.com", "not-an-address", "members[0].address"),
        ("Sam@example.com", "@example.com", "members[0].address"),
        ("\"login\":\"sam\"", "\"login\":\"-sam\"", "members[0].login"),
        ("\"login\":\"sam\"", "\"login\":\"sam_\"", "members[0].login"),
        ("\"login\":\"sam\"", "\"login\":\"s/am\"", "members[0].login"),
        ("\"provider\":3", "\"provider\":1", "google.provider"),
        ("\"provider\":3", "\"provider\":99", "google.provider"),
        ("http://127.0.0.1:31999/token", "http://example.com/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "https://user@127.0.0.1/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "http://127.0.0.1:0/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "http://127.0.0.1:+31999/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "http://127.0.0.1:31999/token#fragment", "google.endpoints.token"),
        ("\"login\":\"sam\"}", "\"login\":\"sam\",\"unknown\":1}", "members[0].unknown"),
        ("\"members\":[{\"address\":\"Sam@example.com\",\"login\":\"sam\"}]", "\"members\":[]", "members"),
        (
            "\"members\":[{\"address\":\"Sam@example.com\",\"login\":\"sam\"}]",
            "\"members\":[{\"address\":\"Sam@example.com\",\"login\":\"sam\"},{\"address\":\"SAM@example.com\",\"login\":\"SAM\"}]",
            "members[1].address",
        ),
    ] {
        let (members, problems) = parse(&input(&SIGN_IN.replace(from, to)), &jig());
        assert!(members.is_none(), "{expected}");
        let lines = startup::lines(&problems);
        assert!(
            core::str::from_utf8(&lines).unwrap().contains(expected),
            "{expected}: {:?}",
            String::from_utf8_lossy(&lines)
        );
    }
}
#[test]
fn owners_must_be_members_and_at_least_one_google_owner_is_required() {
    let mut jig = jig();
    jig.owners[0].identity.subject = b"another@example.com".as_slice().into();
    let (_, problems) = parse(&input(SIGN_IN), &jig);
    assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("engine.owners[0]"));
    jig.owners[0].identity.provider = 1;
    let (_, problems) = parse(&input(SIGN_IN), &jig);
    assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("engine.owners:"));
}
#[test]
fn sign_in_is_required_and_each_zero_limit_is_refused() {
    let missing = input("null").replace("\"sign_in\":null,", "");
    let (_, problems) = parse(&missing, &jig());
    assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("application.sign_in: missing"));
    for name in ["members", "pending", "deadline_ms", "fetches", "body_bytes", "url_bytes", "secret_bytes"] {
        let mut limits = crate::limits::profile(crate::limits::Profile::Small);
        crate::limits::apply(&mut limits, format!("sign_in.{name}").as_bytes(), 0).unwrap();
        let mut problems = Problems::empty();
        startup::checks::limits(&limits, &limits.git_component, &mut problems);
        assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains(&format!("sign_in.{name}")));
    }
}
#[test]
fn filling_the_bundle_sets_the_ways_and_changes_its_etag() {
    for google in [false, true] {
        let mut bundle = fixtures::bundle::bundle();
        assert!(bundle.fill_sign_in(google).is_none(), "missing head refused");
        bundle.page = b"<!doctype html><html><head><title>t</title></head><body></body></html>".as_slice().into();
        let before = bundle.etag.clone();
        bundle.fill_sign_in(google).unwrap();
        let meta = if google {
            b"<meta name=\"temper-sign-in\" content=\"google\"></head>".as_slice()
        } else {
            b"<meta name=\"temper-sign-in\" content=\"\"></head>".as_slice()
        };
        assert!(bundle.page.windows(meta.len()).any(|part| part == meta));
        assert_ne!(bundle.etag, before);
        let hash = bundle
            .page
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3));
        assert_eq!(bundle.etag.as_ref(), format!("\"{hash:016x}\"").as_bytes());
    }
}
#[test]
fn every_members_fallback_identity_fits_both_jig_consumers() {
    for clients in [false, true] {
        let mut jig = jig();
        if clients {
            jig.limits.clients.identity_bytes = 5;
        } else {
            jig.limits.core.people.identity_bytes = 5;
        }
        let (members, problems) = parse(&input(SIGN_IN), &jig);
        assert!(members.is_none());
        assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("application.sign_in.members[0]:"));
    }
}

fn provider(jig: &mut config::Engine) {
    jig.clients.provider = Some(config::clients::Provider {
        key: b"provider.key".as_slice().into(),
        addresses: Box::new([3]),
        clients: Box::new([
            config::clients::ProviderClient {
                id: b"forgejo".as_slice().into(),
                secret: b"forgejo.secret".as_slice().into(),
                redirect_uri: b"http://127.0.0.1:9/user/oauth2/temper/callback".as_slice().into(),
            },
            config::clients::ProviderClient {
                id: b"second".as_slice().into(),
                secret: b"second.secret".as_slice().into(),
                redirect_uri: b"http://127.0.0.1:9/user/oauth2/temper/callback".as_slice().into(),
            },
        ]),
    });
}
fn with_forges(rows: &str) -> String {
    input(&SIGN_IN.replacen('{', &format!("{{\"forges\":{rows},"), 1))
}
#[test]
fn forge_sign_in_is_optional_and_joins_the_provider_client() {
    let mut jig = jig();
    provider(&mut jig);
    let (members, problems) = parse(&with_forges(r#"[{"forge":1,"client":"forgejo"}]"#), &jig);
    assert!(problems.is_empty(), "{:?}", startup::lines(&problems));
    let sign_in = members.unwrap().sign_in.unwrap();
    assert_eq!(sign_in.forges.len(), 1);
    assert_eq!(sign_in.forges[0].forge, 1);
    assert_eq!(sign_in.forges[0].client.as_ref(), b"forgejo");
    assert!(parse(&input(SIGN_IN), &jig).0.unwrap().sign_in.unwrap().forges.is_empty());
}
#[test]
fn forge_sign_in_callback_uses_the_owned_public_origin_or_git_without_one() {
    for (public, callback, accepted) in [
        (Some("https://git.example.com"), "https://git.example.com/user/oauth2/temper/callback", true),
        (Some("https://git.example.com"), "http://127.0.0.1:9/user/oauth2/temper/callback", false),
        (Some("https://git.example.com:8443"), "https://git.example.com:8443/user/oauth2/temper/callback", true),
        (Some("https://git.example.com:443"), "https://git.example.com/user/oauth2/temper/callback", true),
        (Some("https://git.example.com"), "https://git.example.com/user/oauth2/temper/callback/extra", false),
        (None, "http://127.0.0.1:9/user/oauth2/temper/callback", true),
        (None, "https://git.example.com/user/oauth2/temper/callback", false),
    ] {
        let mut jig = jig();
        provider(&mut jig);
        jig.clients.provider.as_mut().unwrap().clients[0].redirect_uri = callback.as_bytes().into();
        let public = public.map_or(String::new(), |public| format!(r#", "public":{public:?}"#));
        let setup = format!(
            r#""setup":[{{"forge":1,"directory":"forge-1",
            "bot":{{"login":"temper","email":"temper@example.com"}},"orgs":[],
            "mode":{{"owned":{{"forgejo":"forgejo","runner":null,"listen":{{"socket":"127.0.0.1:9"}},
                "person":{{"login":"sam","email":"sam@example.com"}}{public}}}}}}}]"#
        );
        let doc = with_forges(r#"[{"forge":1,"client":"forgejo"}]"#).replace(r#""setup":[]"#, &setup);
        let (members, problems) = parse(&doc, &jig);
        assert_eq!(members.is_some(), accepted, "{callback}: {:?}", startup::lines(&problems));
        if accepted {
            assert!(problems.is_empty());
            assert_eq!(members.unwrap().forges[0].git.as_ref(), b"http://127.0.0.1:9");
        } else {
            assert_eq!(startup::lines(&problems).as_ref(), b"application.sign_in.forges[0].client: invalid form\n");
        }
    }
}
#[test]
fn forge_sign_in_refusals_name_the_forge_or_client() {
    for (rows, member) in [
        (r#"[{"forge":99,"client":"forgejo"}]"#, "forges[0].forge"),
        (r#"[{"forge":1,"client":"unknown"}]"#, "forges[0].client"),
        (r#"[{"forge":1,"client":"forgejo"},{"forge":1,"client":"second"}]"#, "forges[1].forge"),
        (r#"[{"forge":1,"client":"forgejo"},{"forge":2,"client":"forgejo"}]"#, "forges[1].client"),
    ] {
        let mut jig = jig();
        provider(&mut jig);
        let (members, problems) = parse(&with_forges(rows), &jig);
        assert!(members.is_none());
        let lines = String::from_utf8(startup::lines(&problems).into()).unwrap();
        assert!(lines.contains(member), "{lines}");
    }
}
#[test]
fn forge_sign_in_client_must_be_visible_ascii() {
    for client in ["has space", "control\t", "nonasciié"] {
        let mut jig = jig();
        provider(&mut jig);
        jig.clients.provider.as_mut().unwrap().clients[0].id = client.as_bytes().into();
        let rows = format!(r#"[{{"forge":1,"client":{client:?}}}]"#);
        let (members, problems) = parse(&with_forges(&rows), &jig);
        assert!(members.is_none());
        let lines = String::from_utf8(startup::lines(&problems).into()).unwrap();
        assert!(lines.contains("application.sign_in.forges[0].client:"), "{lines}");
    }
}
#[test]
fn forge_sign_in_requires_forgejo_an_exact_callback_and_a_provider() {
    let doc = with_forges(r#"[{"forge":1,"client":"forgejo"}]"#);
    for (kind, expected) in [(0_u8, "forges[0].forge"), (1, "forges[0].client"), (2, "engine.clients.provider")] {
        let mut jig = jig();
        provider(&mut jig);
        let mut doc = doc.clone();
        match kind {
            0 => doc = doc.replace("\"kind\":\"forgejo\"", "\"kind\":\"github\""),
            1 => {
                jig.clients.provider.as_mut().unwrap().clients[0].redirect_uri =
                    b"http://127.0.0.1:9/wrong".as_slice().into();
            }
            2 => jig.clients.provider = None,
            _ => unreachable!(),
        }
        let (members, problems) = parse(&doc, &jig);
        assert!(members.is_none());
        let lines = String::from_utf8(startup::lines(&problems).into()).unwrap();
        assert!(lines.contains(expected), "{lines}");
    }
}

#[test]
fn an_old_document_names_the_missing_members_and_their_fix() {
    let input = core::str::from_utf8(fixtures::APPLICATION)
        .unwrap()
        .replace("\"sign_in\":null,", "")
        .replace(",\"public\":null", "");
    let (members, problems) = parse(&input, &jig());
    assert!(members.is_none());
    let lines = String::from_utf8(startup::lines(&problems).into()).unwrap();
    assert!(lines.contains("application.sign_in: missing: null for none\n"), "{lines}");
    assert!(lines.contains("application.clients.public: missing: null for none\n"), "{lines}");
}

#[test]
fn forge_metas_escape_attributes_publish_modes_and_change_the_etag() {
    use temper_forge_setup::setup::{Account, Entry, Fetch, Mode, Owned};
    let mut jig = jig();
    provider(&mut jig);
    let (members, problems) = parse(&with_forges(r#"[{"forge":1,"client":"forgejo"}]"#), &jig);
    assert!(problems.is_empty());
    let mut members = members.unwrap();
    let sign_in = members.sign_in.as_mut().unwrap();
    sign_in.forges[0].client = b"forge&\"'< >".as_slice().into();
    members.forges[0].git = b"https://forge.example/?x=\"&y='<>".as_slice().into();
    let mut setup = temper_forge_setup::setup::Setup { entries: Box::new([]) };
    let mut bundle = fixtures::bundle::bundle();
    bundle.page = b"<head></head>".as_slice().into();
    bundle.fill_sign_in(true).unwrap();
    let before = bundle.etag.clone();
    bundle.fill_forge_sign_in(Some(sign_in), &members.forges, &setup, 65_536).unwrap();
    assert_eq!(bundle.page.as_ref(), b"<head><meta name=\"temper-sign-in\" content=\"google\"><meta name=\"temper-forge-sign-in\" content=\"1 forge&amp;&quot;&#39;&lt; &gt; https://forge.example/?x=&quot;&amp;y=&#39;&lt;&gt; other\"></head>");
    assert_ne!(bundle.etag, before);
    setup.entries = Box::new([Entry {
        forge: 1,
        directory: b"forge-1".as_slice().into(),
        bot: Account { login: b"temper".as_slice().into(), email: b"temper@example.com".as_slice().into() },
        orgs: Box::new([]),
        asked: false,
        mode: Mode::Owned(Owned {
            public: None,
            fetch: Fetch { forgejo: false, runner: false, mirror: None },
            forgejo: b"forgejo".as_slice().into(),
            runner: None,
            listen: "127.0.0.1:9".parse().unwrap(),
            person: Account { login: b"sam".as_slice().into(), email: b"sam@example.com".as_slice().into() },
        }),
    }]);
    bundle.page = b"<head></head>".as_slice().into();
    bundle.fill_forge_sign_in(Some(sign_in), &members.forges, &setup, 65_536).unwrap();
    assert!(bundle.page.ends_with(b" owned\"></head>"));
    for (port, origin) in [(443, "https://git.example.com"), (8443, "https://git.example.com:8443")] {
        let Mode::Owned(owned) = &mut setup.entries[0].mode else { unreachable!() };
        owned.public = Some(temper_forge_setup::setup::Origin { host: b"git.example.com".as_slice().into(), port });
        bundle.page = b"<head></head>".as_slice().into();
        bundle.fill_forge_sign_in(Some(sign_in), &members.forges, &setup, 65_536).unwrap();
        assert_eq!(bundle.page.as_ref(), format!("<head><meta name=\"temper-forge-sign-in\" content=\"1 forge&amp;&quot;&#39;&lt; &gt; {origin} owned\"></head>").as_bytes());
    }
    let before = bundle.page.clone();
    assert!(bundle.fill_forge_sign_in(Some(sign_in), &members.forges, &setup, 1).is_none());
    assert_eq!(bundle.page, before);
}
