//! The observed red boundary, its green turn and the remaining member's round
//! (temper/docs/design/domain/landing-search.md, 3–5 and 8).
use super::{
    Judge, Phase, Process, Proof, Script, Turn, Value, change, gate_runs, get, hold, landing, landings, main,
    member_branch, members, sha,
};
use skein_fake_llm_domain::api::Finish;

#[derive(Clone, Copy, Debug)]
pub(super) enum Step {
    One,
    Two,
    Boundary,
    Remainder,
}

pub(super) fn hold_repair(scripts: &mut Vec<Script>) {
    // Stop after the repair is asked. This delegate never finishes or pushes a
    // new head, so the blamed change cannot re-enter the train in this story.
    scripts.push(Script {
        cue: b"Repair the failed landing gate".as_slice().into(),
        turns: Box::new([Turn { lines: Box::new([hold(b"batch-repair")]), finish: Finish::Stop, tokens: 20 }]),
    });
}

impl Judge<'_> {
    fn receipt(&self, run: u64, prefix: &str, verdict: &str) -> bool {
        let actual = get(self.forge, &format!("actions/runs/{run}"));
        assert_eq!(actual["commit_sha"], prefix);
        if actual["status"] != verdict {
            return false;
        }
        let statuses = get(self.forge, &format!("commits/{prefix}/status"));
        statuses["statuses"]
            .as_array()
            .expect("statuses")
            .iter()
            .any(|status| status["context"] == "temper/gate/gate" && status["status"] == verdict)
    }

    fn probe(&mut self, boundary: usize, verdict: &str) {
        let first = self.first.as_ref().expect("top proof");
        let (previous, prefix) = if boundary == 1 {
            (first.run, sha(&members(&first.panel)[2]["candidate"]))
        } else {
            (self.probes[0], sha(&members(&first.panel)[0]["candidate"]))
        };
        if !self.receipt(previous, prefix, verdict) {
            return;
        }
        let Some(run) = self.barrier.waiting(Some(previous)) else { return };
        let member = &members(&first.panel)[boundary - 1];
        self.dispatched(run, sha(&member["candidate"]));
        let branch = member_branch(first.panel["task"].as_u64().expect("landing task"), boundary);
        assert_eq!(get(self.forge, &format!("branches/{branch}"))["commit"]["id"], sha(&member["candidate"]));
        assert_eq!(gate_runs(self.forge).len(), boundary + 1, "one probe at a time, no push runs");
        assert_eq!(main(self.forge), self.base, "no turn until the boundary is known");
        self.probes.push(run);
        self.barrier.release(run);
        self.phase = Phase::Search(if boundary == 1 { Step::Two } else { Step::Boundary });
    }

    fn blamed(&self, panel: &Value) -> bool {
        let member = &members(panel)[1];
        if member["left"].is_null() {
            return false;
        }
        assert!(member["landed"].is_null(), "the blamed member did not land");
        let change = change(self.root, member["change"].as_u64().expect("blamed change"));
        if change["step"] != "repairing" || change["runs"].as_u64().is_none() {
            return false;
        }
        assert!(change["merge"].is_null());
        let number = member["pull"]["number"].as_u64().expect("blamed pull");
        let pull = get(self.forge, &format!("pulls/{number}"));
        assert_eq!(pull["state"], "open");
        assert_eq!(pull["merged"], false);
        true
    }

    fn green_turn(&self, task: u64, prepared: &Value) {
        // The next round replaces round.pull. Find the first turn by its member
        // branch, not by the candidate pull closed during the search.
        let branch = member_branch(task, 1);
        let pulls = get(self.forge, "pulls?state=all&limit=50");
        assert!(pulls.as_array().expect("pulls").len() < 50, "one page holds every pull");
        let turns: Vec<_> = pulls
            .as_array()
            .expect("pulls")
            .iter()
            .filter(|pull| pull["head"]["ref"] == branch && pull["merged"] == true)
            .collect();
        assert_eq!(turns.len(), 1, "one merged landing pull from the green member's branch");
        let prefix = sha(&prepared["candidate"]);
        assert_eq!(turns[0]["base"]["ref"], "main");
        assert_eq!(turns[0]["head"]["sha"], prefix);
        assert_eq!(turns[0]["merge_commit_sha"], prefix, "fast-forward only, no extra merge commit");
    }

    fn boundary(&mut self) {
        let first = self.first.as_ref().expect("top proof");
        if !self.receipt(self.probes[1], sha(&members(&first.panel)[1]["candidate"]), "failure") {
            return;
        }
        let Some(run) = self.barrier.waiting(Some(self.probes[1])) else { return };
        let task = first.panel["task"].as_u64().expect("landing task");
        let panel = landing(self.root, task);
        if !self.member_landed(&members(&panel)[0], &members(&first.panel)[0]) {
            return;
        }
        let green = sha(&members(&first.panel)[0]["candidate"]);
        assert_eq!(main(self.forge), green, "the first turn lands only the green prefix");
        self.green_turn(task, &members(&first.panel)[0]);
        if !self.blamed(&panel) {
            return;
        }
        assert_eq!(
            landings(self.root)["landings"].as_array().expect("landings").len(),
            1,
            "the suffix stays in its landing"
        );
        assert_eq!(panel["phase"], "proving");
        assert_eq!(panel["round"]["number"], 2);
        let third = &members(&panel)[2];
        assert_eq!(third["change"], 5);
        assert!(third["left"].is_null());
        assert!(third["landed"].is_null());
        assert_eq!(sha(&third["base"]), green, "the next round stacks onto the first turn's tip");
        let prefix = sha(&third["candidate"]);
        self.dispatched(run, prefix);
        let branch = panel["round"]["branch"].as_str().expect("next round's landing branch");
        assert_eq!(get(self.forge, &format!("branches/{branch}"))["commit"]["id"], prefix);
        let commit = get(self.forge, &format!("git/commits/{prefix}"));
        assert_eq!(commit["parents"].as_array().expect("parents").len(), 2);
        assert_eq!(commit["parents"][0]["sha"], green);
        assert_eq!(commit["parents"][1]["sha"], sha(&third["head"]));
        assert_eq!(third["head"], members(&first.panel)[2]["head"], "no member head rewritten");
        assert_eq!(gate_runs(self.forge).len(), 4, "top, two probes and the next round only");
        self.barrier.release(run);
        self.second = Some(Proof { panel, run });
        self.phase = Phase::Search(Step::Remainder);
    }

    fn remainder(&self) -> bool {
        let second = self.second.as_ref().expect("second round");
        let third = &members(&second.panel)[2];
        let prefix = sha(&third["candidate"]);
        if !self.receipt(second.run, prefix, "success") {
            return false;
        }
        let panel = landing(self.root, second.panel["task"].as_u64().expect("landing task"));
        if panel["phase"] != "landed" || !self.member_landed(&members(&panel)[2], third) || !self.blamed(&panel) {
            return false;
        }
        let first = self.first.as_ref().expect("top proof");
        assert!(self.member_landed(&members(&panel)[0], &members(&first.panel)[0]));
        assert_eq!(main(self.forge), prefix, "main contains the third's proved prefix");
        let number = panel["round"]["pull"]["number"].as_u64().expect("second turn pull");
        let pull = get(self.forge, &format!("pulls/{number}"));
        assert_eq!(pull["merged"], true);
        assert_eq!(pull["head"]["ref"], second.panel["round"]["branch"]);
        assert_eq!(pull["head"]["sha"], prefix);
        assert_eq!(pull["merge_commit_sha"], prefix, "the next turn also fast-forwards");
        assert_eq!(gate_runs(self.forge).len(), 4, "no merge or repair runs");
        true
    }

    pub(super) fn search(&mut self, step: Step, procs: &mut [Process]) {
        match step {
            Step::One => self.probe(1, "failure"),
            Step::Two => self.probe(2, "success"),
            Step::Boundary => self.boundary(),
            Step::Remainder => {
                if self.remainder() {
                    // The owner and repair stay held; no repair result is needed
                    // to witness the next round's turn. Terminate drains them.
                    self.stop(procs);
                }
            }
        }
    }
}
