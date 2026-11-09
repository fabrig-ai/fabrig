use crate::Files;
use jig_protocol_store::Root;
use skein_io::file;
use skein_lib::Token;
const OWNER: Token = Token::new(100);
fn opened(files: &mut Files, root: Root, create: bool) -> Token {
    let request = if create {
        file::Request::CreateNoFollow { owner: OWNER, root: root.fd, name: Box::from(b"turn".as_slice()), mode: 0o600 }
    } else {
        file::Request::OpenWriteNoFollow { owner: OWNER, root: root.fd, name: Box::from(b"turn".as_slice()) }
    };
    let file::Event::Opened { file, .. } = files.answer(request).event else {
        panic!("file opens");
    };
    file
}
fn write(files: &mut Files, file: Token) {
    let event = files
        .answer(file::Request::WriteAt { owner: OWNER, file, offset: 0, bytes: Box::from(b"turn bytes".as_slice()) })
        .event;
    assert_eq!(event, file::Event::Written { owner: OWNER });
}
fn sync(files: &mut Files, root: Root, file: Token) {
    assert_eq!(files.answer(file::Request::Sync { owner: OWNER, file }).event, file::Event::Synced { owner: OWNER });
    assert_eq!(
        files.answer(file::Request::SyncDirectory { owner: OWNER, root: root.fd }).event,
        file::Event::Synced { owner: OWNER }
    );
}
fn durable() -> Files {
    let mut files = Files::new();
    let root = files.root();
    let file = opened(&mut files, root, true);
    write(&mut files, file);
    sync(&mut files, root, file);
    files
}
fn read(files: &mut Files) -> Box<[u8]> {
    let root = files.root();
    let file = opened(files, root, false);
    let file::Event::Read { bytes, .. } =
        files.answer(file::Request::ReadAt { owner: OWNER, file, offset: 0, max: 64 }).event
    else {
        panic!("file readable");
    };
    bytes
}
#[test]
fn a_synced_write_survives_a_crash() {
    let mut files = durable();
    files.crash(18);
    assert_eq!(read(&mut files).as_ref(), b"turn bytes");
}
#[test]
fn an_unsynced_cut_keeps_the_old_length_after_a_crash() {
    let mut files = durable();
    let root = files.root();
    let file = opened(&mut files, root, false);
    let event = files.answer(file::Request::Cut { owner: OWNER, file, length: 4 }).event;
    assert_eq!(event, file::Event::Cut { owner: OWNER });
    files.crash(18);
    assert_eq!(read(&mut files).as_ref(), b"turn bytes");
}
#[test]
fn a_removed_missing_file_answers_not_found() {
    let mut files = Files::new();
    let root = files.root();
    let event = files
        .answer(file::Request::Remove {
            owner: OWNER,
            root: root.fd,
            name: Box::from(b"missing".as_slice()),
            directory: false,
        })
        .event;
    assert_eq!(
        event,
        file::Event::Failed { owner: OWNER, error: skein_io::kernel::Error::NotFound, committed: false, residue: None }
    );
}

mod routes;

#[test]
fn a_kill_before_the_first_operation_loses_the_commit_and_past_the_last_keeps_it() {
    use crate::{Fault, KillFrom, Store};
    use jig_test_domain as root;
    let limits = jig_test_profile::limits();
    let header = jig_domain_core::Deployment {
        id: [0; 16],
        tasks: 1,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: 1,
        projects: 0,
        applications: 0,
        commits: 1,
    };
    let writes = Box::new([
        root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
            header,
        )))),
        root::Write::Save(root::Record::Connector {
            number: 1,
            record: jig_test_connector::Record::Result { task: 1, code: 7 },
        }),
    ]);
    let mut reference = Store::on_disk(&limits, &[]);
    reference.submit(1, writes.clone(), 0);
    assert_eq!(reference.tick(false), Ok(Some(1)));
    let count = reference.operation_count(1);
    assert!(count > 0);
    for (operation, durable) in [(0, false), (count + 1, true)] {
        let mut store = Store::on_disk(&limits, &[]);
        store.fault(Fault::Kill { from: KillFrom::Commit(1), operation });
        store.submit(1, writes.clone(), 0);
        assert_eq!(store.tick(false), Ok(durable.then_some(1)));
        assert_eq!(store.reached_kill(), !durable);
        store.crash(18);
        assert_eq!(store.applied, u64::from(durable));
        assert_eq!(store.rows.len(), 2 * usize::from(durable));
        assert_eq!(store.readback(), store.rows);
    }
}
