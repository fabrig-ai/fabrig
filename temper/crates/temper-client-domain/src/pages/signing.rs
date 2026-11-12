//! Words shared by Work's signers and Team's counts (client/pages/work.md and team.md, 1).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};

pub(super) fn count(number: u32) -> Box<[u8]> {
    match number {
        1 => b"one".as_slice().into(),
        2 => b"two".as_slice().into(),
        3 => b"three".as_slice().into(),
        4 => b"four".as_slice().into(),
        5 => b"five".as_slice().into(),
        6 => b"six".as_slice().into(),
        7 => b"seven".as_slice().into(),
        8 => b"eight".as_slice().into(),
        9 => b"nine".as_slice().into(),
        0 | 10.. => Decimal::of(u64::from(number)).as_bytes().into(),
    }
}

pub(super) fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("read bounds joined words");
    }
    let mut out = Writer::new(size);
    for part in parts {
        out.put(part).expect("part lengths counted");
    }
    out.finish()
}
