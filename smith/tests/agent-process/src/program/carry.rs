//! Host-owned window transfers; the service receives only ordinary wire frames.
use skein_lib::Reader;

/// Extract the transcript bodies the agent gave its host, without inspecting domains.
#[must_use]
pub fn from_frames(frames: &[skein_fake_channel::Observed]) -> Vec<Box<[u8]>> {
    frames
        .iter()
        .filter(|frame| frame.kind == 0x0109)
        .map(|frame| {
            let turn = smith_channel::Turn::decode(&smith_channel::CEILINGS, &mut Reader::new(&frame.body))
                .expect("turn envelope");
            Box::from(turn.body())
        })
        .collect()
}

/// Supply a saved window with the next run's charter and grant.
#[must_use]
pub fn start_with_window(
    charter: &[u8],
    value: &smith_channel::GrantValue,
    turns: &[Box<[u8]>],
) -> skein_channel::Frame {
    let message = super::waiting::initial(true);
    let frame = crate::start_workspace_message(charter, 8, false, message.as_ref(), value, None);
    crate::wire::edit_start(&frame, |start| {
        start.transcript = smith_protocol_channel::transcript_piece(turns).expect("saved window");
    })
}
