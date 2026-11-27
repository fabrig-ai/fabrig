//! The landing's prepare job spec and result translation (temper/docs/design/domain/worker/jobs.md, 2;
//! temper/docs/design/protocol/tooling.md, 4).
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_forge::api;
use skein_json::writer::{Encoder, Limits};
use skein_lib::List;
use temper_engine_domain_forge::{self as forge, LandingCombine as Combine, commit_text, routing::append};
use temper_protocol_worker_wire::v1::{self, Record};

#[derive(Debug)]
pub struct PrepareMember {
    pub number: u64,
    pub branch: Box<[u8]>,
    pub prefix: Box<[u8]>,
    pub head: api::Commit,
    pub message: Box<[u8]>,
    pub squash: Box<[u8]>,
}
#[derive(Debug)]
pub struct PrepareRequest {
    pub repository: Box<[u8]>,
    pub main: api::Commit,
    pub from: api::Commit,
    pub combine: Combine,
    pub train_upstream: Option<api::Commit>,
    pub members: Box<[PrepareMember]>,
}

/// Encode the built-in prepare job's bounded spec, with no external command.
#[must_use]
pub fn spec(request: &PrepareRequest, bound: u32) -> Option<Box<[u8]>> {
    let limits = Limits { depth: 4, length: bound };
    let mut measure = Encoder::measure(&limits);
    encode(&mut measure, request);
    let length = measure.measured().ok()?;
    let mut json = Encoder::write(length, &limits);
    encode(&mut json, request);
    let spec = v1::JobSpec {
        command: Box::new([]),
        request: json.finish(),
        refusal: false,
        result: temper_protocol_worker_wire::DEFAULT_JOB_RESULT_BYTES,
    };
    if spec.measure(&v1::CEILINGS).ok()? > bound {
        return None;
    }
    spec.encode(&v1::CEILINGS).ok()
}
fn encode(json: &mut Encoder, request: &PrepareRequest) {
    json.object_start();
    json.key(b"version");
    json.unsigned(1);
    json.key(b"repository");
    json.string(&request.repository);
    json.key(b"main");
    json.string(&commit_text(&request.main));
    json.key(b"from");
    json.string(&commit_text(&request.from));
    json.key(b"combine");
    json.string(match request.combine {
        Combine::Merge => b"merge",
        Combine::Squash => b"squash",
    });
    if let Some(upstream) = &request.train_upstream {
        json.key(b"train_upstream");
        json.string(&commit_text(upstream));
    }
    json.key(b"members");
    json.array_start();
    for member in &request.members {
        json.object_start();
        json.key(b"number");
        json.unsigned(member.number);
        json.key(b"branch");
        json.string(&member.branch);
        json.key(b"prefix");
        json.string(&member.prefix);
        json.key(b"head");
        json.string(&commit_text(&member.head));
        json.key(b"message");
        json.string(&member.message);
        json.key(b"squash");
        json.string(&member.squash);
        json.object_end();
    }
    json.array_end();
    json.key(b"upstreams");
    json.array_start();
    json.array_end();
    json.key(b"gates");
    json.array_start();
    json.array_end();
    json.object_end();
}

/// Read the per-member outcomes from the first job result in the inbox.
#[must_use]
pub fn outcomes(inbox: &[tasks::Word], members: u32) -> Box<[forge::preparation::Prepared]> {
    for word in inbox {
        if word.kind == tasks::MessageKind::Result(tasks::ResultKind::Job) {
            return forge::preparation::outcomes(&word.words, members).unwrap_or(Box::new([]));
        }
    }
    Box::new([])
}

/// Take the first job result or escalation as the failure's words.
#[must_use]
pub fn failure(inbox: &[tasks::Word]) -> Box<[u8]> {
    for word in inbox {
        if word.kind == tasks::MessageKind::Result(tasks::ResultKind::Job) {
            return word.words.clone();
        }
        if let tasks::MessageKind::Escalation { .. } = word.kind {
            return word.words.clone();
        }
    }
    Box::from(b"job failed past its tries".as_slice())
}

/// Read the conflict file names in result order, or refuse an absent or malformed list.
#[must_use]
pub fn conflict_files(bytes: &[u8]) -> Option<Box<[u8]>> {
    let length = u32::try_from(bytes.len()).ok()?;
    let limits = skein_json::tokenizer::Limits { depth: 8, string: length, number: 20, chunk: 256, length };
    let document =
        skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length }).ok()?;
    let conflicts = document.member(0, b"conflicts")?;
    if document.token(conflicts)?.kind != skein_json::Kind::ArrayStart {
        return None;
    }
    let end = document.value_end(conflicts)?.checked_sub(1)?;
    let mut cursor = conflicts.checked_add(1)?;
    let mut words = List::with_capacity(8192);
    for at in conflicts.checked_add(1)?..end {
        if at != cursor {
            continue;
        }
        let files = document.member(at, b"files")?;
        if document.token(files)?.kind != skein_json::Kind::ArrayStart {
            return None;
        }
        for file in files.checked_add(1)?..document.value_end(files)?.checked_sub(1)? {
            let token = document.token(file)?;
            if token.kind != skein_json::Kind::String {
                return None;
            }
            if !words.is_empty() {
                append(&mut words, b", ");
            }
            append(&mut words, document.text(token)?);
        }
        cursor = document.value_end(at)?;
    }
    Some(words.into_boxed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prepare_spec_is_the_empty_command_and_exact_request() {
        let request = PrepareRequest {
            repository: b"repo".as_slice().into(),
            main: [1; 32],
            from: [1; 32],
            combine: Combine::Squash,
            train_upstream: Some([3; 32]),
            members: Box::new([PrepareMember {
                number: 12,
                branch: b"temper/7/c12".as_slice().into(),
                prefix: b"temper/7/l13".as_slice().into(),
                head: [2; 32],
                message: b"Merge \"title\"\n\nLanded".as_slice().into(),
                squash: b"title\n\nTemper-Change: 12".as_slice().into(),
            }]),
        };
        let bytes = spec(&request, 16_384).expect("bounded prepare");
        let decoded = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("encoded spec");
        assert_eq!(decoded.command.as_ref(), b"");
        assert!(!decoded.refusal);
        assert_eq!(decoded.request.as_ref(), br#"{"version":1,"repository":"repo","main":"0101010101010101010101010101010101010101010101010101010101010101","from":"0101010101010101010101010101010101010101010101010101010101010101","combine":"squash","train_upstream":"0303030303030303030303030303030303030303030303030303030303030303","members":[{"number":12,"branch":"temper/7/c12","prefix":"temper/7/l13","head":"0202020202020202020202020202020202020202020202020202020202020202","message":"Merge \"title\"\n\nLanded","squash":"title\n\nTemper-Change: 12"}],"upstreams":[],"gates":[]}"#);
        assert!(spec(&request, u32::try_from(bytes.len()).expect("length") - 1).is_none());
    }

    #[test]
    fn conflicts_array_presence_and_each_entries_files_are_read_as_json() {
        let result = br#"{"combined":[{"member":1,"by":"merge"}],"conflicts":[{"member":2,"with":"branch","files":["code","docs/a,b"],"cut":false},{"member":4,"with":"member","files":["quoted\"name","line\nname"],"cut":true}]}"#;
        assert_eq!(
            conflict_files(result).expect("conflict files").as_ref(),
            b"code, docs/a,b, quoted\"name, line\nname"
        );
        assert_eq!(conflict_files(br#"{"conflicts":[]}"#).expect("present array").as_ref(), b"");
        assert_eq!(conflict_files(br#"{"combined":[{"member":1,"by":"merge"}]}"#), None);
        assert_eq!(conflict_files(br#"{"message":"\"conflicts\""}"#), None);
        assert_eq!(conflict_files(br#"{"conflicts":{"files":["code"]}}"#), None);
        assert_eq!(conflict_files(br#"{"conflicts":[{"files":"code"}]}"#), None);
        assert_eq!(conflict_files(br#"{"conflicts":[{"files":[7]}]}"#), None);
        assert_eq!(conflict_files(br#"{"conflict":{"files":["code"]}}"#), None);
        assert_eq!(conflict_files(b"{not JSON}"), None);
    }
}
