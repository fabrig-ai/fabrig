//! Start admission and ownership checks (domain/host.md, section 3).
//! No state changes or process effects occur before these checks succeed.
use crate::{AnsweredCall, Delivery, Invalid, Limits, Start};

pub(super) fn valid_start(start: &Start, limits: &Limits) -> Option<Invalid> {
    if start.messages.len() > usize::try_from(limits.messages).expect("u32 fits") {
        return Some(Invalid::Messages);
    }
    for (at, message) in start.messages.iter().enumerate() {
        match smith_domain::run::rendered_bytes(&message.label, &message.text) {
            Some(bytes) if bytes <= limits.message_bytes => {}
            Some(_) | None => return Some(Invalid::Messages),
        }
        for earlier in start.messages.get(..at).expect("enumerated prefix") {
            if earlier.name == message.name {
                return Some(Invalid::Messages);
            }
        }
    }
    if start.activation == 0 {
        return Some(Invalid::Activation);
    }
    if start.charter.owned_bytes() > limits.charter_bytes {
        return Some(Invalid::Charter);
    }
    let transcript_fits = match &start.transcript {
        Some(transcript) => transcript.owned_bytes() <= limits.transcript_bytes,
        None => true,
    };
    if !transcript_fits {
        return Some(Invalid::Transcript);
    }
    match answered_bytes(&start.answered, limits.name_bytes) {
        Some(bytes) if bytes <= limits.answered_bytes => {}
        Some(_) | None => return Some(Invalid::Answered),
    }
    if start.directories.len() > usize::try_from(limits.directories).expect("u32 fits usize") {
        return Some(Invalid::Directories);
    }
    match start.workspace {
        Some(_) if start.directories.is_empty() => return Some(Invalid::Directories),
        None if !start.directories.is_empty() => return Some(Invalid::Directories),
        Some(_) | None => {}
    }
    if limits.path_bytes > 4096 {
        return Some(Invalid::Directories);
    }
    for directory in &start.directories {
        if !within(&directory.host, u64::from(limits.path_bytes))
            || !within(&directory.name, u64::from(limits.name_bytes))
            || directory.conflicts.len() > usize::try_from(limits.conflicts).expect("u32 fits usize")
        {
            return Some(Invalid::Directories);
        }
        for path in &directory.conflicts {
            if !within(path, u64::from(limits.path_bytes)) {
                return Some(Invalid::Directories);
            }
        }
    }
    for (at, directory) in start.directories.iter().enumerate() {
        if !safe_name(&directory.name) || (!directory.git && !directory.conflicts.is_empty()) {
            return Some(Invalid::Directories);
        }
        for earlier in start.directories.get(..at).expect("enumerated directory") {
            if directory.name == earlier.name {
                return Some(Invalid::Directories);
            }
        }
        for (position, path) in directory.conflicts.iter().enumerate() {
            if !relative(path, limits.path_bytes) {
                return Some(Invalid::Directories);
            }
            for earlier in directory.conflicts.get(..position).expect("enumerated conflict") {
                if earlier == path {
                    return Some(Invalid::Directories);
                }
            }
        }
    }
    if start.grants.len() > usize::try_from(limits.accounts).expect("u32 fits usize") {
        return Some(Invalid::Grants);
    }
    for (at, grant) in start.grants.iter().enumerate() {
        if grant.generation == 0 {
            return Some(Invalid::Grants);
        }
        for earlier in start.grants.get(..at).expect("enumerated grant") {
            if grant.account == earlier.account {
                return Some(Invalid::Grants);
            }
        }
    }
    None
}

fn safe_name(name: &[u8]) -> bool {
    if name.is_empty() || name == b"." || name == b".." {
        return false;
    }
    for byte in name {
        if *byte == 0 || *byte == b'/' {
            return false;
        }
    }
    true
}

pub(super) fn within(bytes: &[u8], limit: u64) -> bool {
    u64::try_from(bytes.len()).expect("length fits u64") <= limit
}

fn answered_bytes(calls: &[AnsweredCall], name_bytes: u32) -> Option<u64> {
    if calls.len() > 128 {
        return None;
    }
    let mut total = 0_u64;
    for call in calls {
        if call.tool.is_empty() || !within(&call.tool, u64::from(name_bytes)) {
            return None;
        }
        total = total.checked_add(u64::try_from(call.tool.len()).ok()?)?;
        let reply = match &call.reply {
            crate::SavedReply::Host { body, .. } => u64::try_from(body.len()).ok()?,
            crate::SavedReply::Delivery(delivery) => match delivery.as_ref() {
                Delivery::Delivered(delivered) => delivered.owned_bytes(),
                Delivery::Refused(refusal) => refusal.owned_bytes(),
                Delivery::Nothing | Delivery::Failed(_) | Delivery::Stale => 0,
            },
            crate::SavedReply::TooLarge => 0,
        };
        total = total.checked_add(reply)?;
    }
    Some(total)
}

fn relative(path: &[u8], limit: u32) -> bool {
    if path.is_empty() || !within(path, u64::from(limit)) {
        return false;
    }
    let mut from = 0;
    for (at, byte) in path.iter().enumerate() {
        if *byte == 0 {
            return false;
        }
        if *byte == b'/' {
            let part = path.get(from..at).expect("ordered path positions");
            if part.is_empty() || part == b"." || part == b".." {
                return false;
            }
            from = at.checked_add(1).expect("bounded path");
        }
    }
    let part = path.get(from..).expect("bounded path suffix");
    !part.is_empty() && part != b"." && part != b".."
}
