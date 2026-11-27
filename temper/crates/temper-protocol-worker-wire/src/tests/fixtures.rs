use crate::{Bounds, v1};
use alloc::boxed::Box;
use skein_lib::{List, Secret};

pub const BOUNDS: Bounds = Bounds {
    repositories: 2,
    name_bytes: 16,
    remote_bytes: 48,
    conflicts: 2,
    path_bytes: 8,
    token_bytes: 40,
    entries: 3,
    entry_token_bytes: 24,
    text_bytes: 16,
    reds: 2,
    red_key_bytes: 12,
};

pub fn limits() -> v1::Limits {
    crate::limits(&BOUNDS).expect("test bounds within ceilings")
}

pub fn repository(tag: u64, remote: Box<[u8]>) -> v1::Repository {
    let branch = v1::Branch { branch: Box::from(&b"main"[..]) };
    let writable = v1::Writable {
        push: Box::from(&b"temper/work"[..]),
        expected: None,
        landing: None,
        delivered: None,
        pin: None,
    };
    v1::Repository {
        admit: false,
        save: None,
        tag,
        name: Box::from(&b"shop"[..]),
        remote,
        git: None,
        start: v1::Start::Base(branch),
        access: v1::Access::Writable(writable),
        identity: 1,
        account: None,
    }
}

pub fn items(count: u32) -> v1::Items {
    let mut repositories = List::with_capacity(count);
    for tag in 0..count {
        let repository = repository(u64::from(tag), Box::from(&b"https://forge/acme/shop.git"[..]));
        repositories.push(repository).expect("counted room");
    }
    v1::Items { repositories, rerun: None }
}

pub fn identity(token: &[u8]) -> v1::Identity {
    v1::Identity {
        kind: v1::Kind::Forgejo(v1::Token { token: Secret::new(Box::from(token)) }),
        login: Box::from(&b"temper"[..]),
        name: Box::from(&b"Temper"[..]),
        email: Box::from(&b"t@forge.example"[..]),
    }
}
