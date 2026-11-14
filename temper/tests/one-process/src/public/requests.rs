//! Signed-in control requests beside public reads (jig/docs/design/protocol/client/README.md, 3–4).
use super::{service, visitor::Visitor};
use crate::fixtures;
use jig_client::contract as c;
use jig_client_protocol_wire::v22 as w;
use skein_lib::{Reader, Writer};

pub(super) fn owned(read: &w::Read) -> w::Owned {
    let mut writer = Writer::new(usize::try_from(read.measure()).expect("read size"));
    read.encode(&mut writer).expect("read encoding");
    w::Owned::new(&w::CEILINGS, w::OwnedParts { owner: w::Owner::Jig, version: 22, record: writer.finish() })
        .expect("read envelope")
}
pub(super) fn read_body(read: &w::Read) -> Box<[u8]> {
    let record = owned(read);
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("envelope size"));
    record.encode(&mut writer).expect("envelope encoding");
    writer.finish()
}
pub(super) fn keyed(ask: &c::Ask, key: u8) -> Box<[u8]> {
    let ask = jig_client_protocol_link::translate::asks::ask(ask, &w::CEILINGS).expect("fixture ask");
    let mut writer = Writer::new(usize::try_from(ask.measure()).expect("ask size"));
    ask.encode(&mut writer).expect("ask encoding");
    let record = w::Keyed::new(
        &w::CEILINGS,
        w::KeyedParts { owner: w::Owner::Jig, version: 22, request: writer.finish(), key: [key; 16], value: None },
    )
    .expect("request envelope");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("request size"));
    record.encode(&mut writer).expect("request encoding");
    writer.finish()
}
pub(super) fn ask(visitor: &mut Visitor, ask: &c::Ask, key: u8) -> u64 {
    let bearer = [b"Bearer ".as_slice(), service::bearer().as_ref()].concat();
    visitor.send_body(
        fixtures::CLIENTS,
        skein_http::Method::Post,
        b"/jig/v1/request",
        &[(b"Host", b"one.example"), (b"Content-Type", b"application/octet-stream"), (b"Authorization", &bearer)],
        &keyed(ask, key),
    )
}
pub(super) fn answer(bytes: &[u8]) -> Result<c::Answer, String> {
    Ok({
        let record = w::KeyedAnswer::decode(&w::CEILINGS, &mut Reader::new(bytes))
            .map_err(|why| format!("keyed answer: {why:?}"))?;
        jig_client_protocol_link::translate::answers::answer(
            record,
            &jig_client_protocol_link::Limits::new(
                temper_client_shell_terminal::limits::native().client.client.domain,
                temper_client_shell_terminal::limits::native().client.client.exchanges,
            )
            .map_err(|why| format!("native limits: {why:?}"))?,
        )
        .map_err(|why| format!("answer translation: {why:?}"))?
    })
}
