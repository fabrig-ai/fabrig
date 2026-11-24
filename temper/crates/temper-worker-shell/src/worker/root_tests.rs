use super::*;
use skein_io::kernel::Kind;

#[test]
fn unsafe_worker_roots_are_refused_before_making_or_pinning() {
    let worker = service::startup::NamedWorker {
        root: b"/fixture/workspaces".as_slice().into(),
        agent: b"/fixture/agent.json".as_slice().into(),
        program: b"/fixture/temper-agent".as_slice().into(),
        arguments: Box::default(),
    };
    for (kind, owner, mode) in [
        (Kind::Symlink, 1000, 0o700),
        (Kind::Directory, 1001, 0o700),
        (Kind::Directory, 1000, 0o710),
        (Kind::Directory, 1000, 0o701),
        (Kind::File, 1000, 0o700),
    ] {
        let mut owned = Vec::new();
        let why = root_with(
            &worker,
            b"{}",
            1000,
            &mut owned,
            |path| {
                assert_eq!(path, Path::new("/fixture/workspaces"));
                Ok(Stat { kind, owner, mode, size: 0, links: 1 })
            },
            |_, _| panic!("refusal neither makes nor opens the root"),
            |_, _, _| panic!("refusal pins nothing"),
        )
        .expect_err("unsafe root refused");
        assert_eq!(
            format!("temper serve: {why}\n"),
            "temper serve: application.workers[0].root: must be a private directory of this user's, not reached through a link\n"
        );
        assert_eq!(owned, [] as [Fd; 0]);
    }
}
