//! Private loads survive the store's rename between open and stat (io.md, 5.2–5.3).

use skein_fake_machine::{How, Item, Machine as Files, Opened};
use skein_io::file::Event;
use skein_io_world::private::{Judge, Owner, Story, USER};
use skein_lib::Queue;
use skein_sim::{Answer, Ask, Call, Config, Handle, Reply};
use skein_world::{Machine, Memory, World};

const OLD: &[u8] = b"old";
// The private store owner's replacement fixture.
const NEW: &[u8] = b"opaque";

#[derive(Debug)]
struct Replacing {
    files: Files,
    root: Opened,
    reader_handles: Vec<Handle>,
    opened: Option<Answer>,
    rename: Option<Call>,
    replaced: bool,
    unlinked_stats: usize,
}

impl Replacing {
    fn new() -> Self {
        let mut files = Files::for_user(USER);
        let root = files.lay(&[Item::directory(b"secret").mode(0o700), Item::file(b"secret/record", OLD).mode(0o600)]);
        Self { files, root, reader_handles: Vec::new(), opened: None, rename: None, replaced: false, unlinked_stats: 0 }
    }

    fn startup(&mut self) -> Handle {
        Handle::new(self.files.open(self.root, b".", How::Directory).expect("startup root").raw())
    }

    fn release(&mut self, answers: &mut Queue<Answer>) {
        if self.opened.is_some() && self.rename.is_some() {
            // The read descriptor exists before rename; its stat has not been submitted.
            skein_fake_machine::step(&mut self.files, self.rename.take().expect("store rename"), answers);
            answers.push(self.opened.take().expect("reader open completion"));
            self.replaced = true;
        }
    }
}

impl Machine for Replacing {
    fn step(&mut self, call: Call, answers: &mut Queue<Answer>) {
        if !self.replaced && matches!(call.ask, Ask::Rename { .. }) {
            self.rename = Some(call);
            self.release(answers);
            return;
        }
        let reader_open = matches!(&call.ask, Ask::Open { root, .. } if self.reader_handles.contains(root));
        let record = reader_open && matches!(&call.ask, Ask::Open { path, .. } if path.as_ref() == b"record");
        let mut observed = Queue::with_capacity(1);
        skein_fake_machine::step(&mut self.files, call, &mut observed);
        let answer = observed.pop().expect("one machine answer");
        if reader_open && let Ok(Reply::Opened(handle)) = &answer.result {
            self.reader_handles.push(*handle);
        }
        if let Ok(Reply::Stat(stat)) = &answer.result {
            self.unlinked_stats += usize::from(stat.links == 0);
        }
        if record && !self.replaced {
            self.opened = Some(answer);
            self.release(answers);
        } else {
            answers.push(answer);
        }
    }
}

#[test]
fn a_private_load_survives_replacement_between_open_and_stat() {
    let mut files = Replacing::new();
    let reader_root = files.startup();
    let writer_root = files.startup();
    files.reader_handles.push(reader_root);
    let mut world = World::new(7, Config::calm(), Judge, Memory::Checked).with_machine(files);
    world.spawn_root(reader_root, |fd| Owner::new(fd, Story::FileMode, USER));
    world.spawn_root(writer_root, |fd| Owner::new(fd, Story::Replace, USER));
    let mut outcome = world.run();
    assert!(outcome.machine.replaced, "the store's rename crossed the open/stat boundary");
    assert_eq!(outcome.machine.unlinked_stats, 1, "the reader stat saw its replaced inode");
    assert!(outcome.procs[1].events.iter().any(|event| matches!(event, Event::Stored { .. })));
    assert!(
        outcome.procs[0].events.iter().any(
            |event| matches!(event, Event::Loaded { bytes, .. } if bytes.as_ref() == OLD || bytes.as_ref() == NEW)
        ),
        "the reader gets a whole version, never a refusal: {:?}",
        outcome.procs[0].events
    );
    assert!(outcome.machine.opened.is_none() && outcome.machine.rename.is_none());
    outcome.machine.files.close(outcome.machine.root);
    assert_eq!(outcome.machine.files.open_handles(), 0);
}
