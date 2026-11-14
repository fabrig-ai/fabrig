//! temper's parts on jig's channel (protocol/channel.md, sections 3 to 6 and 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
mod generated {
    pub mod v1;
}

pub use generated::v1;

use alloc::boxed::Box;
use skein_lib::{Secret, Writer};

/// Domain bounds shared by both sides of the channel (protocol/channel.md,
/// section 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    pub repositories: u32,
    pub name_bytes: u32,
    pub remote_bytes: u32,
    pub conflicts: u32,
    pub path_bytes: u32,
    pub token_bytes: u32,
    pub entries: u32,
    pub entry_token_bytes: u32,
    pub text_bytes: u32,
    pub reds: u32,
    pub red_key_bytes: u32,
}

/// Derive every schema limit from its domain bound, refusing past a ceiling.
#[must_use]
pub fn limits(bounds: &Bounds) -> Option<v1::Limits> {
    let limits = v1::Limits {
        repository_name: bounds.name_bytes,
        merge_branch: bounds.name_bytes,
        merged_branch: bounds.name_bytes,
        merged_heads: v1::CEILINGS.merged_heads,
        branch_branch: bounds.name_bytes,
        writable_push: bounds.name_bytes,
        prefixes_branches: v1::CEILINGS.prefixes_branches,
        prefixes_branches_item: bounds.name_bytes,
        writable_landing_some: bounds.name_bytes,
        repository_save_some: bounds.name_bytes,
        repository_remote: bounds.remote_bytes,
        repository_git_some: v1::CEILINGS.repository_git_some,
        items_repositories: bounds.repositories,
        job_spec_command: v1::CEILINGS.job_spec_command,
        job_spec_request: v1::CEILINGS.job_spec_request,
        left_landed: bounds.repositories,
        saved_repositories: bounds.repositories,
        files_files: bounds.conflicts,
        files_files_item: bounds.path_bytes,
        diagnostic_output: v1::CEILINGS.diagnostic_output,
        token_token: bounds.token_bytes,
        entries_entries: bounds.entries,
        entry_token: bounds.entry_token_bytes,
        entry_account_login: bounds.text_bytes,
        identity_login: bounds.text_bytes,
        identity_name: bounds.text_bytes,
        identity_email: bounds.text_bytes,
        flake_key: v1::CEILINGS.flake_key,
        flake_log: v1::CEILINGS.flake_log,
        check_results_tree: v1::CEILINGS.check_results_tree,
        check_results_base_some: v1::CEILINGS.check_results_base_some,
        check_results_passed: bounds.reds,
        check_results_passed_item: bounds.red_key_bytes,
        check_results_failed: bounds.reds,
        check_results_failed_item: bounds.red_key_bytes,
        check_results_flakes: v1::CEILINGS.check_results_flakes,
        reds_reds: bounds.reds,
        red_key: bounds.red_key_bytes,
        red_head: v1::CEILINGS.red_head,
    };
    match v1::check_limits(&limits) {
        Ok(()) => Some(limits),
        Err(_) => None,
    }
}

/// Largest application parts as jig's worker wire sizes take them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Largest {
    pub items: u32,
    pub left: u32,
    pub saved: u32,
    pub resource: u32,
    pub grant: u32,
    pub ask: u32,
    pub answer: u32,
}

/// The generated worst cases, refused if a size does not fit a u32.
#[must_use]
pub fn largest(limits: &v1::Limits) -> Option<Largest> {
    Some(Largest {
        items: narrow(v1::Items::encoded_bound(limits)?)?,
        left: narrow(v1::Left::encoded_bound(limits)?)?,
        saved: narrow(v1::Saved::encoded_bound(limits)?)?,
        resource: narrow(v1::Preparation::encoded_bound(limits)?)?,
        grant: narrow(v1::Identity::encoded_bound(limits)?)?,
        // The worker sends at the schema ceilings. The engine must admit that
        // envelope on jig's channel before applying its own report decode bounds
        // (protocol/channel.md, 8; domain/forge/reds.md, 5.6).
        ask: narrow(v1::Ask::encoded_bound(&v1::CEILINGS)?)?,
        answer: narrow(v1::Answer::encoded_bound(limits)?)?,
    })
}

fn narrow(bytes: u64) -> Option<u32> {
    u32::try_from(bytes).ok()
}

/// Encode Items into exactly its measured room.
#[must_use]
pub fn items(value: &v1::Items) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured length fits usize"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode Left into exactly its measured room.
#[must_use]
pub fn left(value: &v1::Left) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured length fits usize"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode Saved into exactly its measured room.
#[must_use]
pub fn saved(value: &v1::Saved) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured length fits usize"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode Preparation into exactly its measured room.
#[must_use]
pub fn preparation(value: &v1::Preparation) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured length fits usize"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode an Ask into exactly its measured room.
#[must_use]
pub fn ask(value: &v1::Ask) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured length fits usize"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode an Answer into exactly its measured room.
#[must_use]
pub fn answer(value: &v1::Answer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured length fits usize"));
    value.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Commit lengths admitted by the application (domain/forge/reds.md, 5.4 and 5.6).
#[must_use]
pub fn red_head(head: &[u8]) -> bool {
    head.len() == 20 || head.len() == 32
}

/// Encode an identity directly into a secret (protocol/channel.md, section 6).
#[must_use]
pub fn identity(value: &v1::Identity) -> Secret {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured length fits usize"));
    value.encode(&mut writer).expect("measured room");
    Secret::new(writer.finish())
}

/// Check configuration's login and author rules (protocol/channel.md, section 6).
#[must_use]
pub fn valid(identity: &v1::Identity) -> bool {
    for byte in identity.login() {
        if *byte == b':' || byte.is_ascii_control() {
            return false;
        }
    }
    author(identity.name()) && author(identity.email())
}

fn author(text: &[u8]) -> bool {
    if text.is_empty() {
        return false;
    }
    for byte in text {
        if *byte == 0 || *byte == b'\n' || *byte == b'<' || *byte == b'>' {
            return false;
        }
    }
    true
}

mod application;
pub use application::{application, spoken};

#[cfg(test)]
mod tests;

/// This crate's spoken family and layout (temper/docs/design/protocol/wires.md, 3.1).
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "temper-worker-wire";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = 9;

    /// The oldest version this build speaks.
    pub const FLOOR: u16 = crate::patches::kept::temper_worker_wire::FLOOR;

    #[expect(clippy::disallowed_macros, reason = "the wire identity embeds its current schema")]
    const TEXT: &str = include_str!("../schema/worker-wire-v1.schema");

    /// The current schema's family fingerprint.
    #[must_use]
    pub fn layout() -> [u8; 32] {
        skein_codec::schema::parse(TEXT.as_bytes())
            .expect("the embedded current schema is valid")
            .fingerprints()
            .family()
            .digest()
    }

    #[cfg(test)]
    mod tests {
        use super::{FAMILY, FLOOR, TEXT, VERSION, layout};

        #[test]
        fn identity_tracks_schema() {
            let schema = skein_codec::schema::parse(TEXT.as_bytes()).expect("current schema");
            assert_eq!(FAMILY.as_bytes(), schema.spelled(schema.family()));
            assert_eq!(VERSION, schema.version());
            assert_eq!(FLOOR, crate::patches::kept::temper_worker_wire::FLOOR);
            assert_eq!(layout(), schema.fingerprints().family().digest());

            let changed = TEXT.replacen("tag: u64", "tag: u32", 1);
            assert_ne!(changed, TEXT, "the mutation changes a field's type");
            let changed = skein_codec::schema::parse(changed.as_bytes()).expect("changed schema");
            assert_ne!(layout(), changed.fingerprints().family().digest());
        }
    }
}

#[path = "../patches/mod.rs"]
pub mod patches;
