//! Forge creation and kept records (temper/docs/design/draft/playground-reset.md, 3).
use crate::{
    Answered, Api, ApiTransport, Bot, Domain, ForgeChoice, ForgeRecord, ForgeState, Kind, Limits, MakeAnswer,
    MakeRefusal, Mode, Request, Stored,
};
use skein_lib::{Env, Queue, Token};

pub(crate) fn make(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    choice: ForgeChoice,
    out: &mut Queue<Request>,
) {
    let answer = if d.next_number == 0 || d.records.len() >= env.limits.forges {
        MakeAnswer::Refused(MakeRefusal::Full)
    } else if let Some(record) = chosen(d, choice) {
        match make_record(d, &env.limits, record, out) {
            Some(forge) => MakeAnswer::Made { forge },
            None => MakeAnswer::Refused(MakeRefusal::Full),
        }
    } else {
        MakeAnswer::Refused(MakeRefusal::NotOffered)
    };
    d.connect.answer(request, token, Answered::MakeForge(answer), out);
}

/// Give a template the next durable forge number and keep it.
pub(crate) fn make_record(
    d: &mut Domain,
    limits: &Limits,
    mut record: ForgeRecord,
    out: &mut Queue<Request>,
) -> Option<u16> {
    if d.next_number == 0 || d.records.len() >= limits.forges {
        return None;
    }
    let number = d.next_number;
    record.number = number;
    record.state = ForgeState::Kept;
    d.next_number = number.checked_add(1).unwrap_or(0);
    keep_record(d, record, out);
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::ForgeNumbers { next: d.next_number } });
    Some(number)
}

fn chosen(d: &Domain, choice: ForgeChoice) -> Option<ForgeRecord> {
    match choice {
        ForgeChoice::Owned => {
            if !d.under_box {
                return None;
            }
            for (_, record) in &d.records {
                if record.mode == Mode::Owned {
                    return None;
                }
            }
            d.machine.owned.clone()
        }
        ForgeChoice::GitHub => Some(d.machine.github.clone()),
        ForgeChoice::Forgejo { api: address, git, web } => {
            if git.is_empty() || git.len() > 1024 {
                return None;
            }
            if let Some(web) = &web
                && web.len() > 1024
            {
                return None;
            }
            Some(ForgeRecord {
                number: 0,
                kind: Kind::Forgejo,
                mode: Mode::Managed,
                api: api(&address)?,
                git,
                git_program: None,
                web,
                read_only: false,
                bot: Bot {
                    login: b"temper".as_slice().into(),
                    name: b"Temper".as_slice().into(),
                    email: b"temper@localhost".as_slice().into(),
                },
                setup: Some(
                    br#"{"bot":{"login":"temper","email":"temper@localhost"},"mode":"managed"}"#.as_slice().into(),
                ),
                first: None,
                playground: false,
                state: ForgeState::Kept,
            })
        }
    }
}

fn api(text: &[u8]) -> Option<Api> {
    let (host, port) = if text == b"::1" {
        (text, 443)
    } else if let Some(rest) = text.strip_prefix(b"[::1]") {
        let port = if rest.is_empty() { 443 } else { port(rest.strip_prefix(b":")?)? };
        (b"::1".as_slice(), port)
    } else {
        let mut colon = None;
        for (index, byte) in text.iter().enumerate() {
            if *byte == b':' {
                if colon.is_some() {
                    return None;
                }
                colon = Some(index);
            } else if !byte.is_ascii_alphanumeric() && *byte != b'.' && *byte != b'-' {
                return None;
            }
        }
        match colon {
            Some(index) => (text.get(..index)?, port(text.get(index.checked_add(1)?..)?)?),
            None => (text, 443),
        }
    };
    if host.is_empty() || host.len() > 253 {
        return None;
    }
    let transport = if host == b"localhost" || host == b"127.0.0.1" || host == b"::1" {
        ApiTransport::Plaintext
    } else {
        ApiTransport::Tls { server_name: host.into(), trust: None }
    };
    Some(Api { host: host.into(), port, transport })
}

fn port(text: &[u8]) -> Option<u16> {
    if text.is_empty() {
        return None;
    }
    let mut number = 0_u16;
    for byte in text {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    if number == 0 { None } else { Some(number) }
}

pub(crate) fn keep_record(d: &mut Domain, record: ForgeRecord, out: &mut Queue<Request>) {
    d.keep_record(record.clone());
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Forge(record.clone()) });
    out.push(Request::Register(record));
    out.push(Request::MarkedForges);
}

pub(crate) fn release_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    d.erase_project_rows(project, out);
    out.push(Request::ProjectReleased { project });
}
