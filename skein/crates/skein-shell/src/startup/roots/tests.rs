use super::{DirectoryCalls, walk};
use skein_io::kernel::Fd;
use std::{ffi::CStr, path::Path};

const MODE: u32 = 0o700;
const BASE: Fd = Fd::new(10);
const STATE: Fd = Fd::new(11);
const PRIVATE: Fd = Fd::new(12);

#[derive(Debug)]
enum Call {
    OpenRoot { path: &'static str, answer: Result<Fd, i32> },
    MakeChild { parent: Fd, name: &'static CStr, mode: u32, answer: Result<(), i32> },
    Sync { directory: Fd, answer: Result<(), i32> },
    OpenChild { parent: Fd, name: &'static CStr, answer: Result<Fd, i32> },
    Close { directory: Fd },
}

struct Script<'a> {
    remaining: &'a [Call],
}

impl Script<'_> {
    fn next(&mut self) -> &Call {
        let (call, rest) = self.remaining.split_first().expect("no unexpected directory call");
        self.remaining = rest;
        call
    }
}

impl DirectoryCalls for Script<'_> {
    fn open_root(&mut self, path: &Path) -> Result<Fd, i32> {
        let call = self.next();
        let Call::OpenRoot { path: expected, answer } = call else { panic!("open_root({path:?}); expected {call:?}") };
        assert_eq!(path, Path::new(expected), "root path");
        *answer
    }

    fn make_child(&mut self, parent: Fd, name: &CStr, mode: u32) -> Result<(), i32> {
        let call = self.next();
        let Call::MakeChild { parent: dir, name: leaf, mode: bits, answer } = call else {
            panic!("make_child({parent:?}, {name:?}); expected {call:?}")
        };
        assert_eq!((parent, name, mode), (*dir, *leaf, *bits), "make child");
        *answer
    }

    fn sync(&mut self, directory: Fd) -> Result<(), i32> {
        let call = self.next();
        let Call::Sync { directory: expected, answer } = call else { panic!("sync({directory:?}); expected {call:?}") };
        assert_eq!(directory, *expected, "sync the parent");
        *answer
    }

    fn open_child(&mut self, parent: Fd, name: &CStr) -> Result<Fd, i32> {
        let call = self.next();
        let Call::OpenChild { parent: dir, name: leaf, answer } = call else {
            panic!("open_child({parent:?}, {name:?}); expected {call:?}")
        };
        assert_eq!((parent, name), (*dir, *leaf), "open child from its parent");
        *answer
    }

    fn close(&mut self, directory: Fd) {
        let call = self.next();
        let Call::Close { directory: expected } = call else { panic!("close({directory:?}); expected {call:?}") };
        assert_eq!(directory, *expected, "close the parent exactly once");
    }
}

#[test]
fn make_root_walks_up_then_makes_syncs_and_opens_children_in_order() {
    const PATH: &str = "base/state/private";
    for made in [Ok(()), Err(libc::EEXIST)] {
        let mut calls = vec![
            Call::OpenRoot { path: PATH, answer: Err(libc::ENOENT) },
            Call::OpenRoot { path: "base/state", answer: Err(libc::ENOENT) },
            Call::OpenRoot { path: "base", answer: Ok(BASE) },
            Call::MakeChild { parent: BASE, name: c"state", mode: MODE, answer: made },
        ];
        if made.is_ok() {
            calls.push(Call::Sync { directory: BASE, answer: Ok(()) });
        }
        calls.extend([
            Call::OpenChild { parent: BASE, name: c"state", answer: Ok(STATE) },
            Call::Close { directory: BASE },
            Call::MakeChild { parent: STATE, name: c"private", mode: MODE, answer: Ok(()) },
            Call::Sync { directory: STATE, answer: Ok(()) },
            Call::OpenChild { parent: STATE, name: c"private", answer: Ok(PRIVATE) },
            Call::Close { directory: STATE },
        ]);
        let mut script = Script { remaining: &calls };
        assert_eq!(walk(Path::new(PATH), MODE, &mut script), Ok(PRIVATE), "hand up the opened last directory");
        assert!(script.remaining.is_empty(), "each other descriptor was closed; unmade calls: {:?}", script.remaining);
    }
}

#[test]
fn make_root_refuses_a_final_or_parent_file_without_changes_or_handles() {
    for (label, path) in [("final file", "base/file"), ("parent file", "base/file/private")] {
        let calls = [Call::OpenRoot { path, answer: Err(libc::ENOTDIR) }];
        let mut script = Script { remaining: &calls };
        assert_eq!(walk(Path::new(path), MODE, &mut script), Err(libc::ENOTDIR), "{label}");
        assert!(script.remaining.is_empty(), "{label}: only the read-only open was made and no handle was acquired");
    }
}

#[test]
fn make_root_closes_the_parent_on_each_preparation_failure() {
    const PATH: &str = "base/state";
    for failure in ["make", "sync", "open"] {
        let mut calls = vec![
            Call::OpenRoot { path: PATH, answer: Err(libc::ENOENT) },
            Call::OpenRoot { path: "base", answer: Ok(BASE) },
            Call::MakeChild {
                parent: BASE,
                name: c"state",
                mode: MODE,
                answer: if failure == "make" { Err(libc::EROFS) } else { Ok(()) },
            },
        ];
        if failure != "make" {
            calls.push(Call::Sync { directory: BASE, answer: if failure == "sync" { Err(libc::EIO) } else { Ok(()) } });
        }
        if failure == "open" {
            calls.push(Call::OpenChild { parent: BASE, name: c"state", answer: Err(libc::ENOTDIR) });
        }
        calls.push(Call::Close { directory: BASE });
        let error = match failure {
            "make" => libc::EROFS,
            "sync" => libc::EIO,
            "open" => libc::ENOTDIR,
            _ => unreachable!("only scripted failures"),
        };
        let mut script = Script { remaining: &calls };
        assert_eq!(walk(Path::new(PATH), MODE, &mut script), Err(error), "{failure}");
        assert!(script.remaining.is_empty(), "{failure}: close the base despite the error");
    }
}
