//! Public reading on the box's demo (temper/docs/design/client/web/public.md, 4).
use crate::{Tier, bundle, configuration, forgery, temper};

impl Tier {
    /// Whether a public landings page's text, run together, is a form the demo's page takes
    /// (temper/docs/design/client/web/public.md, 4): the queue's one member while the change
    /// waits, then no landing or the stock queue's merge, with the public sign-in top bar.
    #[must_use]
    pub fn public_landings_reads(shown: &str) -> bool {
        let Some(mut end) = shown.strip_prefix("temperSign inLandings") else {
            return false;
        };
        if let Some(member) = end.strip_prefix("Queuetask ") {
            let Some((task, member)) = member.split_once(" · Goal ") else {
                return false;
            };
            let Some((goal, time)) = member.split_once(" · Ready ") else {
                return false;
            };
            if task.len() != 8 || !lowercase_hex(task) || goal.len() != 8 || !lowercase_hex(goal) {
                return false;
            }
            if let Some(rest) = time.strip_prefix("just now").or_else(|| time.strip_prefix("a minute ago")) {
                end = rest;
            } else {
                let Some((minutes, rest)) = time.split_once(" minutes ago") else {
                    return false;
                };
                if minutes.is_empty() || !minutes.bytes().all(|digit| digit.is_ascii_digit()) {
                    return false;
                }
                end = rest;
            }
        }
        if end == "No landings to show yet." {
            return true;
        }
        let Some((task, commit)) = end.strip_prefix("landed · task ").and_then(|line| line.split_once(" · ")) else {
            return false;
        };
        task.len() == 8 && lowercase_hex(task) && commit.len() == 7 && lowercase_hex(commit)
    }

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
        Self { scratch, forged: forgery::request(), command: "" }
    }
}

fn lowercase_hex(digits: &str) -> bool {
    digits.bytes().all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
}

#[cfg(test)]
mod tests {
    use crate::Tier;

    const LANDINGS_PAGES: &[(&str, bool)] = &[
        ("temperLandingsNo landings to show yet.", false),
        ("temperSign inSign inLandingsNo landings to show yet.", false),
        ("temperSign inLandingsNo landings to show yet.", true),
        ("temperSign inLandingslanded · task 18c22892 · 789ada1", true),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready just nowNo landings to show yet.", true),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 3f2a9c01 · Ready a minute agoNo landings to show yet.", true),
        (
            "temperSign inLandingsQueuetask 18c22892 · Goal 3f2a9c01 · Ready 2 minutes agolanded · task 18c22892 · 789ada1",
            true,
        ),
        (
            "temperSign inLandingsQueuetask 18c22892 · Goal 3f2a9c01 · Ready 12 minutes agoNo landings to show yet.",
            true,
        ),
        ("temperSign inLandings", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready just now", false),
        ("temperSign inLandingsQueueNo landings to show yet.", false),
        (
            "temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready just nowtask 18c22892 · Goal 18c22892 · Ready just nowNo landings to show yet.",
            false,
        ),
        ("temperSign inLandingsQueuetask 18c2289 · Goal 18c22892 · Ready just nowNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c228920 · Goal 18c22892 · Ready just nowNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18C22892 · Goal 18c22892 · Ready just nowNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 3f2a9c0 · Ready just nowNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 3f2a9c010 · Ready just nowNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 3F2a9c01 · Ready just nowNo landings to show yet.", false),
        ("temperSign inLandingslanded · task 18c2289 · 789ada1", false),
        ("temperSign inLandingslanded · task 18c228920 · 789ada1", false),
        ("temperSign inLandingslanded · task 18C22892 · 789ada1", false),
        ("temperSign inLandingslanded · task 18c22892 · 789ada", false),
        ("temperSign inLandingslanded · task 18c22892 · 789ada10", false),
        ("temperSign inLandingslanded · task 18c22892 · 789Ada1", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Held just nowNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready 1 minute agoNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready an hour agoNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready a day agoNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready minutes agoNo landings to show yet.", false),
        ("temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready  minutes agoNo landings to show yet.", false),
        (
            "temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready -2 minutes agoNo landings to show yet.",
            false,
        ),
        (
            "temperSign inLandingsQueuetask 18c22892 · Goal 18c22892 · Ready 2.5 minutes agoNo landings to show yet.",
            false,
        ),
        ("temperSign inLandingsNo landings to show yet. extra", false),
        ("temperSign inLandingslanded · task 18c22892 · 789ada1 extra", false),
        ("extra temperSign inLandingsNo landings to show yet.", false),
    ];

    #[test]
    fn a_public_landings_page_reads_the_forms_it_draws_and_no_other() {
        for &(shown, accepted) in LANDINGS_PAGES {
            assert_eq!(Tier::public_landings_reads(shown), accepted, "public landings page: {shown}");
        }
    }
}
