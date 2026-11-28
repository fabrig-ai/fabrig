//! Landing results in the task's words (temper/docs/design/domain/root-translation.md, 1).
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge::{self as forge, change_words::landing_words, commit_text, routing::append};

/// Render the landing identity, verification and report within the task's window.
#[must_use]
pub fn words(
    forge: &forge::Domain,
    task: u64,
    merge: &skein_forge::api::Commit,
    pull: u64,
    bound: u32,
    cancelled: bool,
) -> Box<[u8]> {
    let row = forge.change(task).expect("landed change");
    let verification = verification_words(forge, row, merge, bound, cancelled);
    let mut out = change_landing_identity(forge, row, merge, pull, bound, &verification);
    let digits = commit_text(merge);
    let number = skein_lib::Decimal::of(pull);
    let parts: [&[u8]; 7] = [
        b"Landed: pull request #",
        number.as_bytes(),
        b" merged into ",
        &row.base,
        b" as ",
        digits.get(..12).expect("a commit has at least 40 hexadecimal digits"),
        b".",
    ];
    let mut bytes = 1_usize;
    for part in parts {
        bytes = bytes.checked_add(part.len()).expect("bounded landing line");
    }
    // The line names the landing so that the run that asked hears that the change landed (temper/docs/design/domain/root-translation.md, 1).
    if skein_lib::bytes::find(out.as_slice(), b"\npull request #").is_some() && bytes <= skein_lib::usize_of(out.room())
    {
        append(&mut out, b"\n");
        for part in parts {
            append(&mut out, part);
        }
    }
    let Some(head) = row.change.last_head else { return out.into_boxed() };
    let room = bound.saturating_sub(out.len()).saturating_sub(1);
    let Some(evidence) = row.result.reported(&row.report, &head, bound).fitted(room) else { return out.into_boxed() };
    append(&mut out, b"\n");
    append(&mut out, &evidence);
    out.into_boxed()
}

/// Describe the landing's verification, within the task's window.
#[must_use]
pub fn verification_words(
    forge: &forge::Domain,
    row: &forge::ChangeRow,
    merge: &skein_forge::api::Commit,
    bound: u32,
    cancelled: bool,
) -> Box<[u8]> {
    forge::change_words::verification_words(
        &row.change.state,
        forge.landing_verification(row.repository, merge),
        forge.change_ends_at_landing(row.task),
        forge.change_waits_landed(row.task),
        cancelled,
        bound,
    )
}

/// Put verification inside the task document's window before spending room on the pull.
fn change_landing_identity(
    forge: &forge::Domain,
    row: &forge::ChangeRow,
    merge: &skein_forge::api::Commit,
    pull: u64,
    bound: u32,
    verification: &[u8],
) -> List<u8> {
    let repository = forge.repository(row.repository);
    let configured = forge.configured(row.repository.forge);
    let reserve = verification.len().saturating_add(1);
    let digits = commit_text(merge);
    let keep_verification =
        !verification.is_empty() && digits.len().saturating_add(reserve) <= skein_lib::usize_of(bound);
    let identity_bound = if keep_verification {
        bound.saturating_sub(u32::try_from(reserve).expect("verification within bound"))
    } else {
        bound
    };
    let identity = landing_words(merge, identity_bound, repository, pull, configured);
    let mut out = List::with_capacity(bound);
    let id_end = skein_lib::bytes::find(&identity, b"\n").unwrap_or(identity.len());
    append(&mut out, identity.get(..id_end).expect("id within landing words"));
    if keep_verification {
        append(&mut out, b"\n");
        append(&mut out, verification);
    }
    append(&mut out, identity.get(id_end..).expect("pull within landing words"));
    out
}
