//! Two engine lives that end without admitting the worker, after a restart cut
//! (jig/docs/design/protocol/channel/worker-link.md, 1.1, and jig/docs/design/protocol/channel/testing.md, 1).
use crate::{fixtures, processes::ServiceHost};
use jig_protocol_worker_wire as wire;
use jig_protocol_worker_wire::Record;
use skein_channel::Control;
use skein_io::kernel::{Fd, Op, Submit};
use skein_lib::{Reader, Secret, Writer};

#[derive(Default)]
pub(crate) struct Starts {
    pub(crate) roots: Vec<Fd>,
    pub(crate) ended: u32,
    phase: Phase,
}
#[derive(Default)]
enum Phase {
    #[default]
    Normal,
    Refusing,
    Cutting,
    Draining,
}
impl Starts {
    pub(crate) fn arm(&mut self) {
        self.phase = Phase::Refusing;
    }
    pub(crate) fn corrupt_open(&mut self, submit: &mut Submit) {
        if !matches!(self.phase, Phase::Refusing) {
            return;
        }
        let Op::Send { bytes, from: 0, .. } = &mut submit.kind else { return };
        let Some((head, body)) = bytes.split_at_checked(8) else { return };
        let Ok(header) = skein_channel::parse_header(head) else { return };
        if header.kind != 1 {
            return;
        }
        // Rewrite the worker's whole Open before transport can split it. The real responder refuses the wrong secret.
        let (_, limits) = fixtures::peer_bounds();
        let mut open = skein_channel::decode_control(header.kind, body, &limits).expect("whole Open");
        let Control::Open { credential, .. } = &mut open else { unreachable!("Open kind") };
        let mut parts =
            wire::Credential::decode(&wire::CEILINGS, &mut Reader::new(credential)).expect("whole opening credential");
        let mut secret = parts.secret.expose().to_vec();
        *secret.first_mut().expect("nonempty fixture secret") ^= 1;
        parts.secret = Secret::new(secret.into_boxed_slice());
        let mut encoded = Writer::new(credential.len());
        parts.write(&wire::CEILINGS, &mut encoded).expect("same-sized credential");
        *credential = encoded.finish();
        let frame = skein_channel::control_frame(&open, &limits).expect("same-sized Open");
        bytes.copy_from_slice(frame.bytes());
        self.phase = Phase::Normal;
    }
    pub(crate) fn cut_record(&mut self, submit: &Submit) -> bool {
        let Op::Send { bytes, from: 0, .. } = &submit.kind else { return false };
        let Some(header) = bytes.get(..8).and_then(|bytes| skein_channel::parse_header(bytes).ok()) else {
            return false;
        };
        if matches!(self.phase, Phase::Refusing) && header.kind == 3 {
            self.phase = Phase::Draining;
        } else if matches!(self.phase, Phase::Cutting) && header.kind == 0x0188 {
            self.phase = Phase::Draining;
            return true;
        }
        false
    }
    pub(crate) fn stepped(&self, service: &mut jig_test_service::Service) {
        if matches!(self.phase, Phase::Draining) {
            service.shutdown();
        }
    }
}

pub(crate) fn advance(host: &mut ServiceHost) -> Result<(), String> {
    if matches!(host.failed_starts.phase, Phase::Normal)
        && host.failed_starts.ended == 2
        && let Some(end) = host.service.end()
    {
        return Err(format!("third engine start ended: {end:?}"));
    }
    if !matches!(host.failed_starts.phase, Phase::Draining) || !host.service.is_empty() {
        return Ok(());
    }
    {
        let (observed_value, wanted) = (&(host.wire.admitted_hellos), &(0));
        if observed_value != wanted {
            return Err(format!(
                "failed starts send no admission record; assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
            ));
        }
    }
    host.failed_starts.ended += 1;
    host.service = crate::processes::engine_prepared()
        .startup_with_workers(&fixtures::files(false), None, &fixtures::seeds(), None, None, fixtures::workers())
        .map_err(|why| format!("next engine start: {why:?}"))?;
    let root = host.failed_starts.roots.pop().ok_or("independent root for next start")?;
    host.service.adopt_store(root, 1000).map_err(|why| format!("next start store: {why:?}"))?;
    host.files.root(root);
    host.wire.restarted();
    host.revision.0 += 1_000_000;
    host.failed_starts.phase = if host.failed_starts.ended == 1 { Phase::Cutting } else { Phase::Normal };
    Ok(())
}
