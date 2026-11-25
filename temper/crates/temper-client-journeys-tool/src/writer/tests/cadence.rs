use super::{Answer, BRANCH, COMMIT, FRESH, NEXT, OTHER, PROJECT, START, Script, Stream, TIP};

impl Script {
    fn ready(&mut self, tip: &str, now: u64) {
        self.states(None);
        self.git(&["status", "--porcelain", "--untracked-files=no"], "");
        self.fetch();
        self.clock(tip, now);
    }

    fn last_run(&mut self, tip: &str, start: u64) {
        self.last(Answer::Bytes(format!("{tip} {start}\n").into_bytes().into_boxed_slice()));
    }

    fn ancestors(&mut self, older: &str, newer: &str, reachable: bool) {
        self.command("git", &["merge-base", "--is-ancestor", older, newer], i32::from(!reachable), "", Stream::Keep);
    }

    fn look_landings_since(&mut self, last: &str, tip: &str, count: &str) {
        self.git(&["log", "--format=%(trailers:key=Look-landed,keyonly)", &format!("{last}..{tip}")], count);
    }

    fn finishes_at(&mut self, tip: &str, now: u64) {
        self.checkout_at(BRANCH, tip);
        self.print(0, FRESH);
        self.landings(tip, &[(tip, &[])]);
        self.unchanged(tip, now);
    }
}

#[test]
fn a_finished_run_controls_when_the_writer_prints_again() {
    let mut same = Script::fresh(BRANCH, None);
    same.ready(TIP, START + 3600);
    same.last_run(TIP, START);
    same.locked_run(&[], 0, &format!("main's pages are {TIP}'s print\n"), "");

    let mut early = Script::fresh(BRANCH, None);
    early.ready(NEXT, START + 3599);
    early.last_run(TIP, START);
    early.ancestors(TIP, NEXT, true);
    early.look_landings_since(TIP, NEXT, "\n\n");
    early.locked_run(&[], 0, &format!("waiting on the hour after {TIP:.12}\n"), "");

    for (now, ancestor, look) in [(START + 3600, true, false), (START + 60, false, false), (START + 60, true, true)] {
        let mut due = Script::fresh(BRANCH, None);
        due.ready(NEXT, now);
        due.last_run(TIP, START);
        if now < START + 3600 {
            due.ancestors(TIP, NEXT, ancestor);
            if look {
                due.look_landings_since(TIP, NEXT, "\nLook-landed\n\n");
            }
        }
        due.finishes_at(NEXT, now);
        due.locked_run(&[], 0, &format!("{FRESH}main's pages are {NEXT}'s print\n"), "");
    }
}

#[test]
fn an_unreadable_finished_run_is_warned_of_and_replaced() {
    for (answer, why) in [
        (Answer::Bytes(b"broken\n".as_slice().into()), "/writer/.git/journey-pages.finished: expected TIP SECONDS"),
        (Answer::Failed("permission denied".into()), "permission denied"),
    ] {
        let mut script = Script::fresh(BRANCH, None);
        script.ready(TIP, START);
        script.last(answer);
        script.finishes_at(TIP, START);
        script.locked_run(
            &[],
            0,
            &format!("{FRESH}main's pages are {TIP}'s print\n"),
            &format!("journey-pages: ignoring unreadable {why}\n"),
        );
    }
}

#[test]
fn only_an_ended_offer_whose_print_landed_finishes_its_run() {
    for squash in [false, true] {
        let mut script = Script::locked(BRANCH, None);
        script.kept(&format!("task 42\nrun {TIP} {COMMIT} {START}\n"));
        script.task_show(r#"{"task":{"chip":{"phase":"ended"}}}"#);
        script.ready(NEXT, START + 60);
        script.ancestors(COMMIT, "origin/main", !squash);
        if squash {
            script.git(
                &["log", "--topo-order", "--format=%(trailers:key=Printed,valueonly,unfold)", "origin/main"],
                &format!("{TIP}\n"),
            );
        }
        script.finished(TIP, START);
        script.remove(super::OFFER);
        script.ancestors(TIP, NEXT, true);
        script.look_landings_since(TIP, NEXT, "\n");
        script.locked_run(PROJECT, 0, &format!("waiting on the hour after {TIP:.12}\n"), "");
    }

    let mut rejected = Script::locked(BRANCH, None);
    rejected.kept(&format!("task 42\nrun {TIP} {COMMIT} {START}\n"));
    rejected.task_show(r#"{"task":{"chip":{"phase":"failed"}}}"#);
    rejected.ready(TIP, START + 60);
    rejected.ancestors(COMMIT, "origin/main", false);
    rejected.git(&["log", "--topo-order", "--format=%(trailers:key=Printed,valueonly,unfold)", "origin/main"], "\n");
    rejected.remove(super::OFFER);
    rejected.last(Answer::Missing);
    rejected.checkout_at(BRANCH, TIP);
    rejected.print(0, FRESH);
    rejected.landings(TIP, &[(TIP, &[])]);
    rejected.offer_page(OTHER);
    // This offer is at the same TIP but the key and saved commit belong to the new print.
    rejected.locked_run(PROJECT, 0, &format!("{FRESH}wrote {BRANCH} {OTHER}\n"), "");
}

#[test]
fn a_modern_local_offer_waits_without_moving_the_finished_run() {
    let mut script = Script::locked(BRANCH, None);
    script.kept(&format!("commit {COMMIT}\nrun {TIP} {COMMIT} {START}\n"));
    script.fetch();
    script.ancestors(COMMIT, "origin/main", false);
    script.git(&["log", "--topo-order", "--format=%(trailers:key=Printed,valueonly,unfold)", "origin/main"], "\n");
    script.locked_run(&[], 0, &format!("waiting on {COMMIT}\n"), "");
}
