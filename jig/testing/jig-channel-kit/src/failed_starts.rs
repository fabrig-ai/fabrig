//! Two engine lives that end without admitting the worker, after a restart cut
//! (jig/docs/design/protocol/channel/README.md, 9.1 and 11).
use crate::{fixtures, processes::ServiceHost};
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::Queue;

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
    pub(crate) fn receive(&mut self, input: &mut Queue<Complete>) {
        if !matches!(self.phase, Phase::Refusing) {
            return;
        }
        for _ in 0..input.len() {
            let mut complete = input.pop().expect("completion count");
            if let (Op::Recv { buf, .. }, Ok(Done::Count(count))) = (&mut complete.kind, &complete.result) {
                let bytes = &mut buf[..usize::try_from(*count).expect("receive count")];
                if bytes.len() >= 8 && skein_channel::parse_header(&bytes[..8]).is_ok_and(|head| head.kind == 1) {
                    // Wrong credentials make the real responder refuse the opening.
                    // The calm story delivers this small opening in one receive.
                    assert!(bytes.len() > 22, "opening carries credentials");
                    *bytes.last_mut().expect("credential byte") ^= 1;
                }
            }
            input.push(complete);
        }
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
    host.service = jig_test_service::startup(
        &fixtures::engine_document(),
        &fixtures::files(false),
        None,
        &fixtures::seeds(),
        None,
        None,
    )
    .map_err(|why| format!("next engine start: {why:?}"))?;
    let root = host.failed_starts.roots.pop().ok_or("independent root for next start")?;
    host.service.adopt_store(root, 1000).map_err(|why| format!("next start store: {why:?}"))?;
    host.files.root(root);
    host.wire.restarted();
    host.revision.0 += 1_000_000;
    host.failed_starts.phase = if host.failed_starts.ended == 1 { Phase::Cutting } else { Phase::Normal };
    Ok(())
}
