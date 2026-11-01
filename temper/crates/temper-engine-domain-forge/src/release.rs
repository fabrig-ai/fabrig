//! Project release (temper/docs/design/draft/playground-reset.md, 4 and 5).
//! The release stub acknowledges immediately; row erasure belongs to rm-release-project.
use crate::{Domain, Request};
use skein_lib::{Queue, Token};

pub(crate) fn release(_d: &mut Domain, request: Token, project: u32, out: &mut Queue<Request>) {
    out.push(Request::ProjectReleased { request, project });
}
