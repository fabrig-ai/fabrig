//! Public reading on the box's demo (temper/docs/design/client/browser.md, 8.4).
use crate::{Tier, bundle, configuration, forgery, temper};

impl Tier {
    /// The demo with public reading figured on and the CSV delegate tracked as a goal.
    #[must_use]
    pub fn public_demo(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::public_demo(scratch.path(), &temper(), &bundle());
        Self { scratch, forged: forgery::request(), command: "box" }
    }
}
