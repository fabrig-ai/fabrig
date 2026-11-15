//! Recorded closures compared by name, then architecture (hearth/docs/design/domain/store/records.md, 4).
use crate::resolution::Package;
pub use hearth_domain_render_vocabulary::diff::{Listed, Moved, ResolutionChange};
use hearth_lib::encode::Digest;
use skein_lib::List;

#[must_use]
pub(crate) fn resolution(
    from: Digest,
    to: Digest,
    before: Option<&[Package]>,
    after: Option<&[Package]>,
) -> ResolutionChange {
    if from == to {
        return ResolutionChange::Same;
    }
    let (Some(before), Some(after)) = (before, after) else { return ResolutionChange::NewKey };
    let capacity = u32::try_from(before.len().saturating_add(after.len())).expect("bounded closures");
    let mut added = List::with_capacity(capacity);
    let mut removed = List::with_capacity(capacity);
    let mut versions_changed = List::with_capacity(capacity);
    let mut from = before.iter().peekable();
    let mut to = after.iter().peekable();
    for _ in 0..capacity {
        match (from.peek(), to.peek()) {
            (Some(old), Some(new)) if identity(old) == identity(new) => {
                if old.version != new.version {
                    versions_changed
                        .push(Moved {
                            name: new.name.clone(),
                            architecture: new.architecture.clone(),
                            before: old.version.clone(),
                            after: new.version.clone(),
                        })
                        .expect("closure capacity");
                }
                from.next();
                to.next();
            }
            (Some(old), Some(new)) if identity(old) < identity(new) => {
                removed.push(listed(old)).expect("closure capacity");
                from.next();
            }
            (Some(_) | None, Some(new)) => {
                added.push(listed(new)).expect("closure capacity");
                to.next();
            }
            (Some(old), None) => {
                removed.push(listed(old)).expect("closure capacity");
                from.next();
            }
            (None, None) => break,
        }
    }
    ResolutionChange::Resolved {
        added: added.into_boxed(),
        removed: removed.into_boxed(),
        versions_changed: versions_changed.into_boxed(),
    }
}
fn identity(package: &Package) -> (&[u8], &[u8]) {
    (&package.name, &package.architecture)
}
fn listed(package: &Package) -> Listed {
    Listed { name: package.name.clone(), architecture: package.architecture.clone(), version: package.version.clone() }
}
