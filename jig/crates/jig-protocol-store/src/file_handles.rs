//! Handles and single-terminal asks owned by an open (protocol/store/opening-rehearsal.md, 1).
use crate::{Below, Io, host::Lane};
use skein_io::file;
use skein_lib::{List, Token};

#[derive(Debug)]
pub(crate) struct Owned {
    pub pending: u32,
    pub handles: List<(Lane, Token)>,
}
impl Owned {
    pub fn new() -> Self {
        Self { pending: 0, handles: List::uncapped() }
    }
    pub fn issued(&mut self, below: &Below, lengths: [u32; 4]) {
        for ((lane, queue), length) in [
            (Lane::Store, &below.store),
            (Lane::Secrets, &below.secrets),
            (Lane::Transcripts, &below.transcripts),
            (Lane::Checkpoints, &below.checkpoints),
        ]
        .into_iter()
        .zip(lengths)
        {
            for request in queue.iter().skip(skein_lib::length::usize_of(length)) {
                self.pending = self.pending.checked_add(1).expect("bounded file asks");
                if let file::Request::Close { file, .. } = request {
                    let mut found = None;
                    for (index, handle) in self.handles.iter().enumerate() {
                        if *handle == (lane, *file) {
                            found = Some(u32::try_from(index).expect("bounded handle index"));
                            break;
                        }
                    }
                    if let Some(index) = found {
                        self.handles.swap_remove(index).expect("found handle");
                    }
                }
            }
        }
    }
    pub fn arriving(&mut self, io: &Io) {
        let (lane, arrived) = match io {
            Io::Store(a) => (Lane::Store, a),
            Io::Secrets(a) => (Lane::Secrets, a),
            Io::Transcripts(a) => (Lane::Transcripts, a),
            Io::Checkpoints(a) => (Lane::Checkpoints, a),
        };
        self.pending = self.pending.checked_sub(1).expect("file terminal has an ask");
        if let file::Event::Opened { file, .. } = &arrived.event {
            self.handles.push((lane, *file)).expect("owned handles are uncapped");
        }
    }
    pub fn drain(&self, below: &mut Below) -> bool {
        if self.pending != 0 {
            return false;
        }
        let Some((lane, file)) = self.handles.iter().next().copied() else { return true };
        let queue = match lane {
            Lane::Store => &mut below.store,
            Lane::Secrets => &mut below.secrets,
            Lane::Transcripts => &mut below.transcripts,
            Lane::Checkpoints => &mut below.checkpoints,
        };
        queue.push(file::Request::Close { owner: crate::directory::OWNER, file });
        false
    }
}
