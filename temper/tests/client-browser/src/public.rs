//! Public reading on the box's demo (temper/docs/design/client/browser.md, 8.4).
use crate::{Tier, bundle, configuration, forgery, temper};

impl Tier {
    /// The public demo's scripted goal with a supplied title for responsive layout stories.
    #[must_use]
    pub fn public_demo_title(name: &str, title: &str) -> Self {
        let tier = Self::public_demo(name);
        let path = tier.scratch.path().join("temper.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("public demo deployment")).expect("JSON");
        let words = value
            .pointer_mut("/application/demo/llm/asked/0/turns/0/lines/1/arguments/batch/0/spec/words")
            .expect("public demo's delegated goal title");
        assert_eq!(words.as_str(), Some("Export widget's table as CSV"));
        *words = title.into();
        configuration::private(&path, &serde_json::to_vec(&value).expect("public demo JSON"));
        tier
    }

    /// The demo with public reading figured on and the CSV delegate tracked as a goal.
    #[must_use]
    pub fn public_demo(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::public_demo(scratch.path(), &temper(), &bundle());
        Self { scratch, forged: forgery::request(), command: "box" }
    }
}
