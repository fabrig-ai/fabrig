use crate::{
    machine::BoxKind,
    setup_file::{self, Refused, SetupFile},
};
use jig_protocol_config::Directories;

#[test]
fn setup_files_keep_every_member_and_refuse_other_versions() {
    let directories = Directories::given(b"/config", b"/data", b"/cache").unwrap();
    for (kind, input) in [
        (BoxKind::Owned, br#"{"version":1,"kind":"owned","listen":{"socket":"127.0.0.1:3000"},"remote":"http://127.0.0.1:3000","person":{"login":"ada","email":"ada@example.invalid"},"public":"https://git.example.invalid","forgejo":"fetched","runner":{"binary":"fetched","labels":["temper-host:host"],"capacity":2,"path":"/bin"},"mirror":{"forgejo":"https://forge.example.invalid:443","runner":"https://runner.example.invalid:8443"},"sign_in":null}"#.as_slice()),
        (BoxKind::Fake, br#"{"version":1,"kind":"fake","listen":{"socket":"127.0.0.1:3001"},"remote":"http://127.0.0.1:3001","git":"demo/bin/git"}"#),
        (BoxKind::Owned, br#"{"version":1,"kind":"owned","listen":{"socket":"[::1]:3002"},"remote":"http://[::1]:3002","person":{"login":"ada","email":"ada@localhost"},"forgejo":"/bin/forgejo","runner":null,"sign_in":"forgejo"}"#),
    ] {
        let setup = setup_file::decode(input, &directories).unwrap();
        assert_eq!(setup.kind(), kind);
        let encoded = setup_file::encode(&setup);
        assert_eq!(setup_file::decode(&encoded, &directories).unwrap(), setup);
        assert_eq!(setup.entry(17, &directories).unwrap().forge, 17);
        match &setup {
            SetupFile::Owned(o) => assert_eq!(o.person.login.as_ref(), b"ada"),
            SetupFile::Fake(f) => assert_eq!(f.git.as_ref(), b"/data/demo/bin/git"),
        }
        let changed = crate::check::joined(&[b"{\"version\":2", encoded.strip_prefix(b"{\"version\":1").unwrap()]);
        let Err(Refused::Version(version)) = setup_file::decode(&changed, &directories) else { panic!("another version is refused") }; assert_eq!(version, 2);
    }
    let Err(Refused::Json) = setup_file::decode(b"{", &directories) else { panic!("malformed JSON") };
    let Err(Refused::Members(_)) = setup_file::decode(br#"{"version":1,"kind":"other"}"#, &directories) else {
        panic!("unknown kind")
    };
}
