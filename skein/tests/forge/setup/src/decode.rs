//! Bounded tokenizer driver for reconcile's HTTP answers.
use skein_forge::forgejo::{
    client,
    decode::{Malformed, Step},
    document::{Document, Row},
    limits::Limits,
};
use skein_json::tokenizer;
#[derive(Debug)]
pub struct Decoded {
    pub rows: Vec<Row>,
    pub document: Result<Document, Malformed>,
}
#[must_use]
pub fn decode_answer(mut decoder: client::Decoder, body: &[u8], limits: &Limits) -> Decoded {
    let mut rows = Vec::new();
    let mut document = None;
    let first = client::first(&decoder);
    drive(body, limits, first, |event| match client::step(&mut decoder, limits, event) {
        Step::Demand(next) => Some(next),
        Step::Row(row, next) => {
            rows.push(row);
            Some(next)
        }
        Step::Done(value) => {
            document = Some(Ok(value));
            None
        }
        Step::Malformed(error) => {
            document = Some(Err(error));
            None
        }
    });
    Decoded { rows, document: document.expect("terminal decoder") }
}
fn drive(
    body: &[u8],
    limits: &Limits,
    mut demand: tokenizer::Request,
    mut event: impl FnMut(&tokenizer::Event) -> Option<tokenizer::Request>,
) {
    // No decoded string can exceed the complete input, including malformed
    // mutations. Keep the configured bound while reserving only useful room.
    let mut json = limits.json;
    json.string = json.string.min(u32::try_from(body.len()).expect("body fits"));
    let mut feed = tokenizer::Feed::new(body, &json);
    for _answer in 0..=body.len() {
        match event(&feed.answer(demand)) {
            Some(next) => demand = next,
            None => return,
        }
    }
    panic!("decoder exhausted its input step bound")
}
