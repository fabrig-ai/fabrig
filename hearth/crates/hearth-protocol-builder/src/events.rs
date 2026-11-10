//! Pipe routing for the git child (hearth/docs/design/domain/render/README.md, 2.2).
use crate::{
    git,
    git::{Intake, Stage},
    render,
    terminal::stream_ended,
};
use skein_lib::{Token, stream::Up};

pub(super) fn stream_up(child: &mut git::Child, owner: Token, up: Up) -> Option<Up> {
    let (output, error) = match &mut child.stage {
        Stage::Live { output, error } => (output, error),
        Stage::Closing => return None,
    };
    if let Up::Failed(_) = &up {
        child.fault = Some(render::errors::Ended::Stream);
    }
    if child.error.token() == Some(owner) {
        match up {
            Up::Bytes(bytes) => {
                child.tail.admit(&bytes);
                *error = Intake::Demand;
            }
            Up::End => {
                *error = Intake::Ended;
            }
            Up::Failed(_) => {
                *error = Intake::Ended;
                child.failed = true;
            }
            Up::Room => unreachable!("read-only stderr"),
        }
        return None;
    }
    if child.output.token() == Some(owner) && (*output == Intake::Draining || child.failed) {
        match up {
            Up::Bytes(_) => *output = Intake::Waiting,
            Up::End => *output = Intake::Ended,
            Up::Failed(_) => {
                *output = Intake::Ended;
                child.failed = true;
            }
            Up::Room => unreachable!("read-only stdout"),
        }
        return None;
    }
    if child.output.token() == Some(owner) {
        *output = if stream_ended(&up) { Intake::Ended } else { Intake::Waiting };
    }
    Some(up)
}
