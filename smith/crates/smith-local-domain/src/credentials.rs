//! Names, forms and validity of grants lent to the child agent (domain/host.md,
//! section 7). Secret credential bytes stay below the domain.

use skein_lib::List;
use smith_host_domain::Grant;

/// Grants collected for a start; each requested account settles once.
#[derive(Debug)]
pub(crate) struct Grants {
    pub(crate) values: List<Grant>,
    pub(crate) awaiting: u32,
}
