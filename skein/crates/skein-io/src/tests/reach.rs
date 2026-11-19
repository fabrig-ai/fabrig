//! Reach admission and failures (runtime/io/processes.md, 6).

use super::{Kind, Rig, limits, owner, process::command};
use crate::kernel::{self, Fd, Op, Reach, Rights, Rule, Spawn, Unenforced, Way};
use crate::{Error, Event, Limits, Request};
use alloc::boxed::Box;

#[expect(clippy::disallowed_macros, reason = "test rows need a reach at each rule count")]
fn reached(abi: u32, rules: usize, path: &[u8], rights: Rights) -> Reach {
    Reach { abi, rules: vec![Rule { root: Fd::new(91), path: path.into(), rights }; rules].into_boxed_slice() }
}

#[expect(clippy::disallowed_macros, reason = "the test builds a command of an exact byte count")]
fn sized_command(bytes: usize) -> Spawn {
    let mut spawn = command(&[(0, Way::Null), (1, Way::Null)]);
    // Program 9 + 4, argument + 4, environment 9 + 4, two descriptors + 8.
    spawn.args = Box::new([vec![b'a'; bytes.checked_sub(38).expect("room for command framing")].into()]);
    spawn.env = Box::new([Box::from(b"KEY=value".as_slice())]);
    spawn
}

#[test]
fn reach_bounds_at_io_entrance() {
    skein_rows::check(
        [
            ("abi below", (5, 1, b"".as_slice(), Rights::READ, None, false)),
            ("abi floor", (6, 1, b"".as_slice(), Rights::READ, None, true)),
            ("abi above", (7, 1, b"".as_slice(), Rights::READ, None, false)),
            ("no rules", (6, 0, b"".as_slice(), Rights::READ, None, true)),
            ("64 rules", (6, 64, b"".as_slice(), Rights::READ, None, true)),
            ("65 rules", (6, 65, b"".as_slice(), Rights::READ, None, false)),
            ("path 4095", (6, 1, [b'p'; 4095].as_slice(), Rights::READ, None, true)),
            ("path 4096", (6, 1, [b'p'; 4096].as_slice(), Rights::READ, None, false)),
            ("absolute path", (6, 1, b"/path".as_slice(), Rights::READ, None, false)),
            ("NUL path", (6, 1, b"a\0b".as_slice(), Rights::READ, None, false)),
            ("one right", (6, 1, b"".as_slice(), Rights::IOCTL_DEV, None, true)),
            ("empty rights", (6, 1, b"".as_slice(), Rights(0), None, false)),
            ("command 61440", (6, 1, b"".as_slice(), Rights::READ, Some(61440), true)),
            ("command 61441", (6, 1, b"".as_slice(), Rights::READ, Some(61441), false)),
        ],
        |(abi, rules, path, rights, bytes, admitted)| {
            let mut rig = Rig::new(limits());
            let mut spawn = match bytes {
                Some(bytes) => sized_command(bytes),
                None => command(&[(1, Way::Out)]),
            };
            let reach = reached(abi, rules, path, rights);
            spawn.reach = Some(Box::new(reach.clone()));
            let mut out = rig.down(Request::Spawn { owner: owner(1), spawn });
            if admitted {
                assert_eq!(out.events, []);
                assert_eq!(out.subs.len(), 1);
                let submit = out.take(Kind::Spawn);
                let Op::Spawn { spawn } = &submit.kind else { panic!("spawn admitted") };
                assert_eq!(spawn.reach, Some(Box::new(reach)), "admission keeps the reach unchanged");
                rig.complete(submit, Err(kernel::Error::NotFound));
            } else {
                out.nothing();
                assert_eq!(rig.io.entities.len(), 0, "bounds are checked before taking any slot");
                let out = rig.next();
                assert_eq!(
                    out.events,
                    [
                        Event::Failed { owner: owner(1), error: Error::Reach(Unenforced::Bounds) },
                        Event::Closed { owner: owner(1) },
                    ]
                );
                assert_eq!(out.subs, []);
            }
            rig.empty();
        },
    );
}

#[test]
fn backend_reach_failure_reaches_spawn_and_spawn_alone_owners() {
    skein_rows::check([("spawn", false), ("spawn alone", true)], |alone| {
        let mut rig = Rig::new(Limits { sockets: 3, ..limits() });
        let mut spawn = command(&[(0, Way::In), (1, Way::Out)]);
        spawn.reach = Some(Box::new(reached(6, 0, b"", Rights::READ)));
        let request = if alone {
            Request::SpawnAlone { owner: owner(2), spawn }
        } else {
            Request::Spawn { owner: owner(2), spawn }
        };
        let submit = rig.down(request).take(Kind::Spawn);
        let why = Unenforced::Abi { found: 5, needs: 6 };
        let out = rig.complete(submit, Err(kernel::Error::Reach(why)));
        assert_eq!(
            out.events,
            [Event::Failed { owner: owner(2), error: Error::Reach(why) }, Event::Closed { owner: owner(2) }]
        );
        assert_eq!(out.subs, []);
        rig.next().nothing();
        rig.empty();
        // All three slots are available again, for the child and both pipes.
        let submit = rig
            .down(Request::Spawn { owner: owner(3), spawn: command(&[(0, Way::In), (1, Way::Out)]) })
            .take(Kind::Spawn);
        rig.complete(submit, Err(kernel::Error::NotFound));
        rig.empty();
    });
}

#[test]
fn kernel_spawn_validity_holds_reach_bounds_only_with_a_reach() {
    skein_rows::check(
        [
            ("admitted reach", (Some(64), 61440, true)),
            ("refused reach", (Some(65), 61440, false)),
            ("unbounded without reach", (None, 61441, true)),
        ],
        |(rules, bytes, valid)| {
            let mut spawn = sized_command(bytes);
            if let Some(rules) = rules {
                spawn.reach = Some(Box::new(reached(6, rules, b"", Rights::READ)));
            }
            assert_eq!(Op::Spawn { spawn: Box::new(spawn) }.is_valid(), valid);
        },
    );
}

#[test]
fn rights_use_landlock_bits_and_sets() {
    skein_rows::check(
        [
            ("execute", (Rights::EXECUTE, 1_u16 << 0_u32)),
            ("write file", (Rights::WRITE_FILE, 1_u16 << 1_u32)),
            ("read file", (Rights::READ_FILE, 1_u16 << 2_u32)),
            ("read dir", (Rights::READ_DIR, 1_u16 << 3_u32)),
            ("remove dir", (Rights::REMOVE_DIR, 1_u16 << 4_u32)),
            ("remove file", (Rights::REMOVE_FILE, 1_u16 << 5_u32)),
            ("make char", (Rights::MAKE_CHAR, 1_u16 << 6_u32)),
            ("make dir", (Rights::MAKE_DIR, 1_u16 << 7_u32)),
            ("make reg", (Rights::MAKE_REG, 1_u16 << 8_u32)),
            ("make sock", (Rights::MAKE_SOCK, 1_u16 << 9_u32)),
            ("make fifo", (Rights::MAKE_FIFO, 1_u16 << 10_u32)),
            ("make block", (Rights::MAKE_BLOCK, 1_u16 << 11_u32)),
            ("make sym", (Rights::MAKE_SYM, 1_u16 << 12_u32)),
            ("refer", (Rights::REFER, 1_u16 << 13_u32)),
            ("truncate", (Rights::TRUNCATE, 1_u16 << 14_u32)),
            ("ioctl dev", (Rights::IOCTL_DEV, 1_u16 << 15_u32)),
        ],
        |(right, bit)| assert_eq!(right.0, bit),
    );
    assert_eq!(Rights::FULL, Rights(0x7fff));
    assert_eq!(Rights::FULL_BUT_EXECUTE, Rights(0x7ffe));
    assert_eq!(Rights::READ, Rights::READ_FILE.union(Rights::READ_DIR));
    assert_eq!(Rights::READ_EXECUTE, Rights::READ.union(Rights::EXECUTE));
    assert_eq!(Rights::READ_WRITE_FILE, Rights::READ_FILE.union(Rights::WRITE_FILE));
    assert_eq!(Rights::FILE, Rights(0xc007));
    assert!(Rights::READ.contains(Rights::READ_FILE));
    assert!(Rights::READ.contains(Rights::READ));
    assert!(Rights::READ.contains(Rights(0)));
    assert!(!Rights::READ.contains(Rights::WRITE_FILE));
    assert!(!Rights::READ_FILE.contains(Rights::READ));
    assert_eq!(Rights::READ_FILE.union(Rights::READ_FILE), Rights::READ_FILE);
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the Display contract is inspected as text in this test")]
fn unenforced_words_name_the_problem_and_the_remedy() {
    skein_rows::check(
        [
            ("bounds", (Unenforced::Bounds, ["bounds", "64", "4095", "61440"].as_slice())),
            ("unsupported", (Unenforced::Unsupported, ["Linux", "6.12", "Landlock"].as_slice())),
            ("absent", (Unenforced::Absent, ["Linux", "6.12", "Landlock", "ENOSYS", "seccomp"].as_slice())),
            ("disabled", (Unenforced::Disabled, ["Landlock", "EOPNOTSUPP", "landlock", "lsm="].as_slice())),
            ("abi", (Unenforced::Abi { found: 5, needs: 6 }, ["5", "6", "Linux", "6.12"].as_slice())),
            ("rule", (Unenforced::Rule { index: 17, errno: 2_i32 }, ["17", "ENOENT", "2", "path"].as_slice())),
            ("restrict", (Unenforced::Restrict { errno: 1_i32 }, ["EPERM", "1", "seccomp", "Landlock"].as_slice())),
            ("helper", (Unenforced::Helper, ["main", "arguments", "helper", "dispatch", "first"].as_slice())),
        ],
        |(why, facts)| {
            let words = why.to_string();
            for fact in facts {
                assert!(words.contains(*fact), "{why:?} must name {fact}: {words}");
            }
        },
    );
}
