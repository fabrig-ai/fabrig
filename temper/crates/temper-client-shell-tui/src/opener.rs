//! The browser opener's command (temper/docs/design/client/README.md, 7.1).
use skein_io::kernel::{Fd, Pipe, Spawn, Way};

pub struct Opener {
    pub program: Box<[u8]>,
    pub env: Box<[Box<[u8]>]>,
}

/// The address is the opener's one argument, with no shell or terminal descriptors.
#[must_use]
pub fn spawn(opener: &Opener, address: &[u8], root: Fd) -> Spawn {
    Spawn {
        program: opener.program.clone(),
        args: Box::new([Box::from(address), Box::from(b"extra".as_slice())]),
        env: opener.env.clone(),
        root,
        dir: Box::from(b".".as_slice()),
        pipes: Box::new([
            Pipe { child: 0, way: Way::Null, parent: None },
            Pipe { child: 1, way: Way::Null, parent: None },
            Pipe { child: 2, way: Way::Null, parent: None },
        ]),
    }
}
