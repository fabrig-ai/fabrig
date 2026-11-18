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

    /// Read every task row's phase, separator and time in the layout story's text.
    #[must_use]
    pub fn public_task_lines_read(shown: &[&str]) -> bool {
        let mut rows = 0;
        let mut words = shown.iter().copied().peekable();
        while let Some(name) = words.next() {
            let Some(id) = name.strip_prefix("Goal ").or_else(|| name.strip_prefix("task ")) else { continue };
            // A row's identity is followed by its title, unlike the heading,
            // lineage links and queue members (client/web/public.md, 3).
            if id.len() != 8 || !lowercase_hex(id) || words.peek() != Some(&" ") {
                continue;
            }
            words.next();
            if words.next().is_none() {
                return false;
            }
            if words.peek() == Some(&"…") {
                words.next();
            }
            let Some(mut phase) = words.next() else { return false };
            if ["an agent", "a procedure", "a person", "a job"].contains(&phase) {
                if words.next() != Some(" · ") {
                    return false;
                }
                let Some(word) = words.next() else { return false };
                phase = word;
            }
            let ended = ENDED_PHASES.contains(&phase);
            if !ended && !LIVE_PHASES.contains(&phase) {
                return false;
            }
            if words.next() != Some(" · ") {
                return false;
            }
            let Some(time) = words.next() else { return false };
            if !(if ended { ago_form(time) } else { time.starts_with("for ") }) {
                return false;
            }
            rows += 1;
        }
        rows > 0
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
        Self { scratch, forged: forgery::request() }
    }
}

fn lowercase_hex(digits: &str) -> bool {
    digits.bytes().all(|digit| digit.is_ascii_digit() || (b'a'..=b'f').contains(&digit))
}

// Public rows use full units, then calendar words for older endings (client/web/public.md, 3).
// These are the reader's literals, independent of the view's time formatter.
const AGO_FORMS: &[&str] = &[
    "just now",
    "a minute ago",
    "N minutes ago",
    "an hour ago",
    "N hours ago",
    "an hour a minute ago",
    "an hour N minutes ago",
    "N hours a minute ago",
    "N hours N minutes ago",
    "at HH:MM",
    "yesterday",
    "Sun",
    "Mon",
    "Tue",
    "Wed",
    "Thu",
    "Fri",
    "Sat",
    "N MONTH",
    "N MONTH N",
];
const ENDED_PHASES: &[&str] = &["landed", "done", "failed", "ended"];
const LIVE_PHASES: &[&str] = &[
    "waiting",
    "running",
    "active",
    "preparing",
    "held",
    "waiting for news",
    "due to run",
    "preparing a host",
    "claimed by a host",
    "executor active",
    "retrying",
    "closing",
];

fn ago_form(time: &str) -> bool {
    AGO_FORMS.iter().any(|form| {
        let mut words = time.split(' ');
        let fits = form.split(' ').all(|part| {
            let Some(word) = words.next() else { return false };
            match part {
                "N" => !word.is_empty() && word.bytes().all(|byte| byte.is_ascii_digit()),
                "HH:MM" => word.split_once(':').is_some_and(|(hour, minute)| {
                    hour.len() == 2
                        && minute.len() == 2
                        && hour.bytes().chain(minute.bytes()).all(|byte| byte.is_ascii_digit())
                        && hour.parse::<u8>().is_ok_and(|hour| hour < 24)
                        && minute.parse::<u8>().is_ok_and(|minute| minute < 60)
                }),
                "MONTH" => [
                    "January",
                    "February",
                    "March",
                    "April",
                    "May",
                    "June",
                    "July",
                    "August",
                    "September",
                    "October",
                    "November",
                    "December",
                ]
                .contains(&word),
                literal => word == literal,
            }
        });
        fits && words.next().is_none()
    })
}

#[cfg(test)]
mod tests {
    use crate::Tier;

    #[test]
    fn a_task_phase_is_read_in_its_row_not_the_changes_step() {
        let mut shown = vec![
            "Goal e78b5cff",
            "Goal e78b5cff",
            " ",
            "Export widget's table as CSV…",
            "a procedure",
            " · ",
            "landed",
            " · ",
            "just now",
            " · ",
            "20 spent",
            "task 9e5605aa",
            " ",
            "Write the CSV export",
            "done",
            " · ",
            "just now",
            "landed",
            "CI at its head",
            " · ",
            "Passed",
        ];
        assert!(Tier::public_task_lines_read(&shown), "the change's landed step is not a task phase");
        for (at, invalid) in [(7, ""), (8, "for a minute"), (14, "unknown"), (15, ""), (16, "")] {
            let original = std::mem::replace(&mut shown[at], invalid);
            assert!(!Tier::public_task_lines_read(&shown), "task fact {at} must still be checked");
            shown[at] = original;
        }
        shown[6] = "running";
        shown[8] = "for a minute";
        assert!(Tier::public_task_lines_read(&shown));
        shown[8] = "just now";
        assert!(!Tier::public_task_lines_read(&shown), "a live row needs its duration, not an ending time");
    }

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
