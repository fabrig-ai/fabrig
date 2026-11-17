use super::fixtures;
use crate::{Defaults, Directories, Engine, Fixed, FixedClients, Path, Problem, Problems, What};
use skein_lib::Duration;

fn parse(bytes: &[u8], fixed: &Fixed) -> Result<Engine, Problems> {
    let document = crate::document(bytes, &crate::DocumentLimits { bytes: 8192, tokens: 1536, depth: 16 }).unwrap();
    crate::engine(&document, 0, &Directories::working(), Defaults::Document, fixed)
}
fn fixed() -> Fixed {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).unwrap();
    Fixed {
        deployment: Some(engine.deployment),
        providers: Some(engine.providers),
        connectors: Some(engine.connectors),
        acts: Some(Box::new([jig_domain_people::ActGrant {
            connector: 1,
            may: Box::new([jig_domain_people::ActKind::Adopt]),
        }])),
        projects: Some(Box::new([])),
        clients: Some(FixedClients { heartbeat: Duration::from_millis(10_000), link: true, approval: true }),
    }
}
fn absent() -> Box<[u8]> {
    let bytes = fixtures::without_projects(fixtures::ENGINE_SMALL);
    let bytes = fixtures::with(&bytes, b",\n\"projects\": []", b"");
    let bytes = fixtures::with(&bytes, b"\"clients\": {\n\"heartbeat_ms\": 10000,\n\"link\": true\n},\n", b"");
    let bytes = fixtures::with(&bytes, b"\"deployment\": {\"directory\": \"state\", \"seed\": 7},\n", b"");
    let bytes = fixtures::with(
        &bytes,
        b"\"providers\": [{\"number\": 1, \"kind\": \"link\"}, {\"number\": 2, \"kind\": \"deployment\"}],\n",
        b"",
    );
    fixtures::with(&bytes, b"\"connectors\": [1],\n", b"")
}
#[test]
fn fixed_members_are_consumed_and_refused_at_their_paths() {
    let absent = absent();
    for (member, value, path) in [
        (b"deployment".as_slice(), b"null".as_slice(), Path::root().member(b"deployment")),
        (b"providers", b"null", Path::root().member(b"providers")),
        (b"connectors", b"[1]", Path::root().member(b"connectors")),
        (b"acts", b"null", Path::root().member(b"acts")),
        (b"projects", b"null", Path::root().member(b"projects")),
        (b"clients", br#"{"heartbeat_ms":null}"#, Path::root().member(b"clients").member(b"heartbeat_ms")),
        (b"clients", br#"{"link":true}"#, Path::root().member(b"clients").member(b"link")),
        (b"clients", br#"{"approval":null}"#, Path::root().member(b"clients").member(b"approval")),
    ] {
        let mut writer = skein_lib::Writer::new(member.len() + value.len() + 5);
        for bytes in [b"{".as_slice(), b"\"", member, b"\":", value, b","] {
            writer.put(bytes).unwrap();
        }
        let document = fixtures::with(
            &absent,
            b"{\n\"limits\"",
            &skein_lib::bytes::concat(&[writer.finish().as_ref(), b"\n\"limits\""]),
        );
        let found = parse(&document, &fixed()).unwrap_err();
        assert_eq!(found.problems.as_slice(), &[Problem { path, what: What::Fixed }]);
    }
}
#[test]
fn absent_fixed_members_supply_values_and_clients_remains_optional() {
    for clients in [b"".as_slice(), b"\"clients\":{},"] {
        let document = fixtures::with(
            &absent(),
            b"{\n\"limits\"",
            &skein_lib::bytes::concat(&[b"{".as_slice(), clients, b"\n\"limits\""]),
        );
        let engine = parse(&document, &fixed()).unwrap();
        assert_eq!(engine.deployment.directory.as_ref(), b"state");
        assert_eq!(engine.deployment.seed, Some(7));
        assert_eq!(engine.providers.as_ref(), fixed().providers.unwrap().as_ref());
        assert_eq!(engine.connectors.as_ref(), [1]);
        assert_eq!(engine.acts.as_ref(), fixed().acts.unwrap().as_ref());
        assert_eq!(engine.limits.core.people.act_grants, 1);
        assert!(engine.projects.is_empty());
        assert!(engine.seed_policies.is_empty());
        assert_eq!(engine.clients.heartbeat, Duration::from_millis(10_000));
        assert!(engine.clients.link);
        assert!(engine.clients.approval);
        assert!(engine.clients.provider.is_none());
    }
}
#[test]
fn fixed_values_join_reference_checks() {
    let mut given = fixed();
    given.providers.as_mut().unwrap()[0].number = 9;
    let found = parse(&absent(), &given).unwrap_err();
    assert_eq!(
        found.problems.as_slice(),
        &[Problem {
            path: Path::root().member(b"owners").child(crate::Segment::Index(0)).member(b"provider"),
            what: What::Refers,
        }]
    );
    let mut given = fixed();
    given.connectors = Some(Box::new([9]));
    let found = parse(&absent(), &given).unwrap_err();
    assert_eq!(
        found.problems.as_slice(),
        &[Problem {
            path: Path::root().member(b"acts").child(crate::Segment::Index(0)).member(b"connector"),
            what: What::Refers,
        }]
    );
}
