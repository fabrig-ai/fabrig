//! Private request files beside an otherwise simulated agent process.

use skein_scratch::Scratch;
use smith_agent_process_world::{
    PRIVATE_GRANT,
    testing::dump::{BOUND, TURNS, accepted, dump_errors, files, world},
};
use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn a_request_s_file_holds_the_bytes_the_provider_read() {
    for (anthropic, suffix) in [(false, "-openai.json"), (true, "-anthropic.json")] {
        let scratch = Scratch::new("request-dump-body");
        let directory = scratch.path().join("private/dumps");
        let mut world = world(directory.clone(), BOUND, anthropic, true);
        assert!(world.settle());
        accepted(&world);
        let files = files(&directory);
        let requests: Vec<_> =
            files.iter().filter(|(path, _)| path.to_str().expect("UTF-8 dump path").ends_with(suffix)).collect();
        assert_eq!(requests.len(), usize::try_from(TURNS).expect("valid dump fixture"));
        assert_eq!(world.bodies().len(), requests.len());
        for ((path, bytes), body) in requests.into_iter().zip(world.bodies()) {
            assert_eq!(bytes.as_slice(), body.as_ref(), "{}", path.display());
        }
        for (path, bytes) in files {
            assert!(skein_lib::bytes::find(&bytes, PRIVATE_GRANT).is_none());
            assert_eq!(fs::metadata(&path).expect("dump file metadata").permissions().mode() & 0o777, 0o600);
            assert!(!path.file_name().expect("dump filename").to_str().expect("UTF-8 dump path").starts_with(".dump-"));
        }
        assert_eq!(fs::metadata(&directory).expect("dump directory metadata").permissions().mode() & 0o777, 0o700);
        assert_eq!(
            fs::metadata(directory.parent().expect("nested dump directory"))
                .expect("valid dump fixture")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(dump_errors(&world).len(), 0);
    }
}
