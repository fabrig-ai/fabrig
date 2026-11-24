use super::*;
use crate::tests::directory_facts::{Disk, USER};
use skein_fake_machine::Item;
use std::path::Path;

#[test]
fn program_directory_preflight_precedes_the_missing_secret_and_worker_root() {
    for (items, reason) in [
        (vec![], None),
        (vec![Item::directory(b"programs").mode(0o700)], None),
        (
            vec![Item::directory(b"real-programs").mode(0o700), Item::link(b"programs", b"real-programs")],
            Some("symbolic link"),
        ),
        (vec![Item::directory(b"programs").mode(0o750)], Some("nonprivate mode bits 0o50")),
        (vec![Item::file(b"programs", b"not a directory")], Some("not a directory")),
    ] {
        let disk = Disk::new(&items);
        let programs = Path::new("/fixture/programs");
        let mut worker = Some(service::startup::NamedWorker {
            root: b"/fixture/workspaces".as_slice().into(),
            agent: b"/fixture/agent.json".as_slice().into(),
            program: b"/fixture/temper-agent".as_slice().into(),
            arguments: Box::default(),
        });
        let mut checked_secret = false;
        let why = before_worker_root(
            &mut worker,
            |_| {
                agent::check_program_directory(smith_agent_document::program::private_directory_with(
                    programs,
                    USER,
                    disk.stat(programs),
                ))?;
                Ok(None)
            },
            || {
                checked_secret = true;
                disk.assert_absent(Path::new("/fixture/worker.secret"));
                Err("workers[0].secret: /fixture/worker.secret: absent".into())
            },
            |_, _| panic!("no worker root on refusal"),
        )
        .err()
        .expect("directory or secret refused");
        let why = serve_error("temper", &why);
        if let Some(reason) = reason {
            assert_eq!(
                why,
                format!("temper serve: programs.claude_code.directory: {}: {reason}\n", programs.display())
            );
            assert!(!checked_secret, "the directory is refused first");
        } else {
            assert!(why.contains("workers[0].secret"), "{why}");
            assert!(!why.contains("programs.claude_code.directory"), "{why}");
            assert!(checked_secret, "admitted directory, then the secret check");
        }
        disk.assert_absent(Path::new("/fixture/workspaces"));
        if items.is_empty() {
            disk.assert_absent(programs);
        }
    }
}
