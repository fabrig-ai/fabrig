//! Root access for faults that the real client normally prevents.
use crate::{jig::runner, root};
use skein_lib::{Env, Queue};
pub(crate) fn told(root: &mut root::Root, env: &Env<crate::jig::Limits>, told: jig_client::Told) -> Queue<runner::Out> {
    let mut out = Queue::with_capacity(crate::tests::fixtures::OUTPUTS);
    runner::told(&mut root.runner, &root.client, env, told, &mut out);
    out
}
pub(crate) fn repeat_number(root: &mut root::Root) {
    root.next = Some(1);
}

pub(crate) fn forge_told(root: &mut root::Root, limits: &root::Limits, told: jig_client::Told) -> Queue<runner::Out> {
    let mut out = Queue::with_capacity(crate::tests::fixtures::OUTPUTS);
    crate::forge::told(&mut root.adopter, limits, told, &mut out);
    out
}
