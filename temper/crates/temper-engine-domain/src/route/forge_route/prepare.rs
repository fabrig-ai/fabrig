//! The stack prepare spec and items (temper/docs/design/domain/worker/jobs.md;
//! temper/docs/design/protocol/tooling.md, 4).
use super::{ForgeWorkspace, forge, workspace_repository};
use alloc::boxed::Box;
use skein_forge::api;
use skein_json::writer::{Encoder, Limits};
use skein_lib::{List, Writer};
use temper_protocol_worker_wire::v1;

pub(crate) use temper_engine_domain_forge_landing::Combine;

#[derive(Debug)]
pub(crate) struct PrepareMember {
    pub(crate) number: u64,
    pub(crate) branch: Box<[u8]>,
    pub(crate) prefix: Box<[u8]>,
    pub(crate) head: api::Commit,
    pub(crate) message: Box<[u8]>,
    pub(crate) squash: Box<[u8]>,
}
#[derive(Debug)]
pub(crate) struct PrepareRequest {
    pub(crate) repository: Box<[u8]>,
    pub(crate) main: api::Commit,
    pub(crate) from: api::Commit,
    pub(crate) combine: Combine,
    pub(crate) train_upstream: Option<api::Commit>,
    pub(crate) members: Box<[PrepareMember]>,
}

pub(crate) fn spec(request: &PrepareRequest, bound: u32) -> Option<Box<[u8]>> {
    let limits = Limits { depth: 4, length: bound };
    let mut measure = Encoder::measure(&limits);
    encode(&mut measure, request);
    let length = measure.measured().ok()?;
    let mut json = Encoder::write(length, &limits);
    encode(&mut json, request);
    let spec =
        v1::JobSpec::new(&v1::CEILINGS, v1::JobSpecParts { command: Box::new([]), request: json.finish() }).ok()?;
    if spec.measure() > bound {
        return None;
    }
    let mut writer = Writer::new(usize::try_from(spec.measure()).ok()?);
    spec.encode(&mut writer).ok()?;
    Some(writer.finish())
}
fn encode(json: &mut Encoder, request: &PrepareRequest) {
    json.object_start();
    json.key(b"version");
    json.unsigned(1);
    json.key(b"repository");
    json.string(&request.repository);
    json.key(b"main");
    json.string(&super::commit_text(&request.main));
    json.key(b"from");
    json.string(&super::commit_text(&request.from));
    json.key(b"combine");
    json.string(match request.combine {
        Combine::Merge => b"merge",
        Combine::Squash => b"squash",
    });
    if let Some(upstream) = &request.train_upstream {
        json.key(b"train_upstream");
        json.string(&super::commit_text(upstream));
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
    json.string(&super::commit_text(&member.head));
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

/// The base is an immutable commit start; the request supplies fetched head/upstream commits.
pub(crate) fn items(
    domain: &forge::Domain,
    provider: api::Repository,
    _root: u64,
    task: u64,
    request: &PrepareRequest,
    _name_bytes: u32,
) -> Option<ForgeWorkspace> {
    let repository = domain.repository(provider)?;
    if repository.name != request.repository {
        return None;
    }
    let mut branches = List::with_capacity(u32::try_from(request.members.len()).ok()?);
    for member in &request.members { branches.push(member.prefix.clone()).ok()?; }
    let item = domain.prepare_item(provider, branches.as_slice().first()?.clone(), request.from, task)?;
    let mut repository = workspace_repository(item, None, Box::new([]))?;
    repository.prefixes = branches.into_boxed();
    repository.save = None;
    Some(ForgeWorkspace { repositories: Box::new([repository]) })
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
        assert_eq!(decoded.command(), b"");
        assert_eq!(decoded.request(), br#"{"version":1,"repository":"repo","main":"0101010101010101010101010101010101010101010101010101010101010101","from":"0101010101010101010101010101010101010101010101010101010101010101","combine":"squash","train_upstream":"0303030303030303030303030303030303030303030303030303030303030303","members":[{"number":12,"branch":"temper/7/c12","prefix":"temper/7/l13","head":"0202020202020202020202020202020202020202020202020202020202020202","message":"Merge \"title\"\n\nLanded","squash":"title\n\nTemper-Change: 12"}],"upstreams":[],"gates":[]}"#);
        assert!(spec(&request, u32::try_from(bytes.len()).expect("length") - 1).is_none());
    }
}
