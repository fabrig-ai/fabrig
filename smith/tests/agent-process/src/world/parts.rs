//! The scripted parent's durable part acknowledgements, using only observed frames.
use skein_lib::{Duration, Reader, Time};
use smith_channel::Record;

pub(super) struct Host {
    after: Duration,
    kept: u32,
    pending: Option<(Time, u32)>,
    pub(super) acknowledgements: Vec<(Time, u32, usize)>,
}

impl Host {
    pub(super) fn new(after: Duration) -> Self {
        Self { after, kept: 0, pending: None, acknowledgements: Vec::new() }
    }

    pub(super) fn deadline(&self) -> Option<Time> {
        self.pending.map(|(at, _)| at)
    }

    pub(super) fn acknowledge(&mut self, now: Time, peer: &mut skein_fake_channel::ScriptedPeer) {
        if self.pending.is_none() {
            let next =
                peer.observed().iter().filter(|frame| frame.kind == smith_channel::kind::PART).find_map(|frame| {
                    let part = smith_channel::Part::decode(&smith_channel::CEILINGS, &mut Reader::new(&frame.body))
                        .expect("told part");
                    (part.number > self.kept).then_some(part.number)
                });
            self.pending = next.map(|number| (now.saturating_add(self.after), number));
        }
        if let Some((at, number)) = self.pending
            && at <= now
        {
            self.acknowledgements.push((now, number, peer.observed().len()));
            peer.play(
                smith_channel::frame(smith_channel::Body::PartKept(smith_channel::PartKept { number }))
                    .expect("part kept frame"),
            );
            self.kept = number;
            self.pending = None;
        }
    }
}
