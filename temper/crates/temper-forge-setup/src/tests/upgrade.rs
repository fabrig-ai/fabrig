use super::fixtures::{self, Drive};
use crate::{
    place::File,
    prepare::{Event, Exit, Failure, Request},
    spec::{Command, Which},
    state,
    upgrade::{self, Course},
};
use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};

#[test]
fn the_upgrade_dumps_an_older_forgejo_before_it_migrates() {
    let mut drive = Drive::new(fixtures::owned());
    let backup = fixtures::place().backup(&fixtures::backup(b"16.0.5"));
    assert_eq!(drive.reach(fixtures::older(Some(b"16.0.5"), None)), Request::Layout);
    assert_eq!(drive.send(Event::Done), Request::Exists { path: backup.clone() });
    assert_eq!(drive.send(Event::Exists(false)), fixtures::run_command(Command::Dump));
    assert_eq!(
        drive.send(fixtures::ran_ok()),
        Request::Rename { from: fixtures::place().path(File::DumpPartial), to: backup }
    );
    let Request::Write { file: File::AppIni, .. } = drive.send(Event::Done) else { panic!("settings after dump") };
    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Migrate));
    let Request::Write { file: File::State, contents } = drive.send(fixtures::ran_ok()) else {
        panic!("recorded last")
    };
    assert_eq!(&*state::decode(contents.expose()).unwrap().forgejo, VERSION.as_bytes());
    assert_eq!(drive.send(Event::Done), Request::End(Ok(())));
}
#[test]
fn the_upgrade_names_its_dump_by_both_versions() {
    assert_eq!(&*upgrade::backup_name(b"16.0.5", b"17.0.0"), b"forgejo-16.0.5-to-17.0.0.zip");
}
#[test]
fn the_upgrade_does_not_dump_again_when_the_backup_is_there() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(drive.reach(fixtures::older(Some(b"16.0.5"), None)), Request::Layout);
    assert_eq!(
        drive.send(Event::Done),
        Request::Exists { path: fixtures::place().backup(&fixtures::backup(b"16.0.5")) }
    );
    let Request::Write { file: File::AppIni, .. } = drive.send(Event::Exists(true)) else { panic!("kept backup") };
    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Migrate));
}
#[test]
fn the_upgrade_compares_versions_by_number_not_text() {
    for (recorded, pinned, expected) in [
        (b"9.9.9".as_slice(), b"10.0.0".as_slice(), Course::Older),
        (b"1.10.0", b"1.9.0", Course::Newer),
        (b"1.0", b"1.0.0", Course::Same),
        (b"1.2.3.4", b"1.2.3.3", Course::Newer),
        (b"999999999.0.0", b"999999998.0.0", Course::Newer),
    ] {
        assert_eq!(upgrade::course(recorded, pinned), expected, "{recorded:?}");
    }
}
#[test]
fn the_upgrade_prepares_an_older_runner_again_without_a_dump() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(drive.reach(fixtures::older(None, Some(b"13.1.0"))), Request::Lock(File::RunnerLock));
    assert_eq!(drive.send(Event::Done), Request::Layout);
    let Request::Write { file: File::AppIni, .. } = drive.send(Event::Done) else { panic!("settings") };
    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Register));
    let Request::Write { file: File::RunnerConfiguration, .. } = drive.send(fixtures::ran_ok()) else {
        panic!("connection")
    };
    let Request::Write { file: File::State, contents } = drive.send(Event::Done) else { panic!("recorded last") };
    assert_eq!(state::decode(contents.expose()).unwrap().runner.as_deref(), Some(RUNNER_VERSION.as_bytes()));
    assert_eq!(drive.send(Event::Done), Request::End(Ok(())));
}
#[test]
fn a_runner_added_is_registered_under_its_lock() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(drive.reach(fixtures::runner_added()), Request::Lock(File::RunnerLock));
    assert_eq!(drive.send(Event::Done), Request::Layout);
    let Request::Write { file: File::AppIni, .. } = drive.send(Event::Done) else { panic!("settings") };
    assert_eq!(drive.send(Event::Done), Request::Draw);
    assert_eq!(
        drive.send(Event::Drew(fixtures::draw(4))),
        Request::Write {
            file: File::RunnerSecret,
            contents: fixtures::secret(b"0404040404040404040404040404040404040404")
        }
    );
    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Register));
    let Request::Write { file: File::RunnerConfiguration, .. } = drive.send(fixtures::ran_ok()) else {
        panic!("connection")
    };
    let Request::Write { file: File::State, contents } = drive.send(Event::Done) else { panic!("recorded") };
    assert_eq!(state::decode(contents.expose()).unwrap().runner.as_deref(), Some(RUNNER_VERSION.as_bytes()));
}
#[test]
fn a_runner_added_keeps_its_existing_secret() {
    let mut found = fixtures::runner_added();
    found.runner_secret = true;
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(drive.reach(found), Request::Lock(File::RunnerLock));
    assert_eq!(drive.send(Event::Done), Request::Layout);
    let Request::Write { file: File::AppIni, .. } = drive.send(Event::Done) else { panic!("settings") };
    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Register));
}
#[test]
fn a_runner_removed_writes_no_runner_file_and_records_none() {
    let mut owned = fixtures::owned();
    owned.runner = None;
    let mut drive = Drive::new(owned);
    assert_eq!(drive.reach(fixtures::older(None, Some(b"unreadable"))), Request::Layout);
    let Request::Write { file: File::AppIni, contents } = drive.send(Event::Done) else { panic!("settings") };
    assert!(skein_lib::bytes::find(contents.expose(), b"ENABLED = false").is_some());
    let Request::Write { file: File::State, contents } = drive.send(Event::Done) else { panic!("recorded") };
    assert_eq!(state::decode(contents.expose()).unwrap().runner, None);
    assert_eq!(drive.send(Event::Done), Request::End(Ok(())));
}

#[test]
fn the_upgrade_refuses_a_newer_forgejo_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(Some(b"999.0.0".as_slice()), None)),
        Request::End(Err(Failure::Newer { which: Which::Forgejo, recorded: b"999.0.0".as_slice().into() }))
    );
}

#[test]
fn the_upgrade_refuses_a_newer_runner_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(None, Some(b"999.0.0".as_slice()))),
        Request::End(Err(Failure::Newer { which: Which::Runner, recorded: b"999.0.0".as_slice().into() }))
    );
}

#[test]
fn the_upgrade_refuses_garbage_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(Some(b"another-version".as_slice()), None)),
        Request::End(Err(Failure::Unreadable { which: Which::Forgejo }))
    );
}

#[test]
fn the_upgrade_refuses_an_empty_version_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(Some(b"".as_slice()), None)),
        Request::End(Err(Failure::Unreadable { which: Which::Forgejo }))
    );
}

#[test]
fn the_upgrade_refuses_five_components_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(Some(b"1.2.3.4.5".as_slice()), None)),
        Request::End(Err(Failure::Unreadable { which: Which::Forgejo }))
    );
}

#[test]
fn the_upgrade_refuses_ten_digits_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(Some(b"1234567890.0.0".as_slice()), None)),
        Request::End(Err(Failure::Unreadable { which: Which::Forgejo }))
    );
}

#[test]
fn the_upgrade_refuses_a_prerelease_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(Some(b"17.0.0-rc1".as_slice()), None)),
        Request::End(Err(Failure::Unreadable { which: Which::Forgejo }))
    );
}

#[test]
fn the_upgrade_refuses_an_unreadable_runner_before_writing() {
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(
        drive.reach(fixtures::older(None, Some(b"garbage".as_slice()))),
        Request::End(Err(Failure::Unreadable { which: Which::Runner }))
    );
}

#[test]
fn the_upgrade_stops_where_dump_failed_and_repeats_next_time() {
    let found = fixtures::older(Some(b"16.0.5"), None);
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(drive.reach(found.clone()), Request::Layout);
    drive.send(Event::Done);
    drive.send(Event::Exists(false));
    assert_eq!(
        drive.send(fixtures::failed()),
        Request::End(Err(Failure::Command {
            command: Command::Dump,
            exit: Exit::Code(1),
            tail: b"broken".as_slice().into()
        }))
    );
    let mut again = Drive::new(fixtures::owned());
    assert_eq!(again.reach(found), Request::Layout);
}

#[test]
fn the_upgrade_stops_where_migrate_failed_and_repeats_next_time() {
    let found = fixtures::older(Some(b"16.0.5"), None);
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(drive.reach(found.clone()), Request::Layout);
    drive.send(Event::Done);
    drive.send(Event::Exists(true));
    drive.send(Event::Done);
    assert_eq!(
        drive.send(fixtures::failed()),
        Request::End(Err(Failure::Command {
            command: Command::Migrate,
            exit: Exit::Code(1),
            tail: b"broken".as_slice().into()
        }))
    );
    let mut again = Drive::new(fixtures::owned());
    assert_eq!(again.reach(found), Request::Layout);
}

#[test]
fn the_upgrade_stops_where_register_failed_and_repeats_next_time() {
    let found = fixtures::older(None, Some(b"13.1.0"));
    let mut drive = Drive::new(fixtures::owned());
    assert_eq!(drive.reach(found.clone()), Request::Lock(File::RunnerLock));
    drive.send(Event::Done);
    drive.send(Event::Done);
    drive.send(Event::Done);
    assert_eq!(
        drive.send(fixtures::failed()),
        Request::End(Err(Failure::Command {
            command: Command::Register,
            exit: Exit::Code(1),
            tail: b"broken".as_slice().into()
        }))
    );
    let mut again = Drive::new(fixtures::owned());
    assert_eq!(again.reach(found), Request::Lock(File::RunnerLock));
}
