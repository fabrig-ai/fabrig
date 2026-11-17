#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests insert refused document members"
)]
use super::fixtures;
use crate::startup;
use jig_protocol_config::Directories;

#[test]
fn moved_worker_members_print_their_paths_and_destinations_in_every_shape() {
    for document in [fixtures::document(), fixtures::worker_alone_document()] {
        for (name, words) in [
            ("graces", "moved: temper fixes a worker's graces (temper help server)"),
            ("programs", "moved: temper fixes the login checks' interval"),
        ] {
            let input = core::str::from_utf8(&document)
                .unwrap()
                .replace("\"workers\":[{", &format!("\"workers\":[{{\"{name}\":null,"));
            let found = startup::shape(input.as_bytes(), &Directories::working()).unwrap_err();
            assert_eq!(startup::lines(&found).as_ref(), format!("workers[0].{name}: {words}\n").as_bytes());
        }
    }
    for document in [fixtures::document(), fixtures::worker_alone_document(), fixtures::engine_alone_document()] {
        let input =
            core::str::from_utf8(&document).unwrap().replace("\"application\":{", "\"application\":{\"workers\":null,");
        let found = startup::shape(input.as_bytes(), &Directories::working()).unwrap_err();
        assert_eq!(
            startup::lines(&found).as_ref(),
            b"application.workers: moved: a worker's workspaces are the data directory's; its git is found on PATH\n"
        );
    }
}
