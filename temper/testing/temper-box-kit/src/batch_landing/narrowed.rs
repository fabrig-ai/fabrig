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
    fn receipt(&self, run: u64, prefix: &str, verdict: &str) -> Result<bool, String> {
        Ok({
            let actual = get(self.forge, &format!("actions/runs/{run}"));
            {
                let (left, right) = (&(actual["commit_sha"]), &(prefix));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            if actual["status"] != verdict {
                return Ok(false);
            }
            let statuses = get(self.forge, &format!("commits/{prefix}/status"));
            statuses["statuses"]
                .as_array()
                .ok_or(String::from("statuses"))?
                .iter()
                .any(|status| status["context"] == "temper/gate/gate" && status["status"] == verdict)
        })
    }

    fn probe(&mut self, boundary: usize, verdict: &str) -> Result<(), String> {
        let first = self.first.as_ref().ok_or(String::from("top proof"))?;
        let (previous, prefix) = if boundary == 1 {
            (first.run, sha(&members(&first.panel)?[2]["candidate"])?)
        } else {
            (self.probes[0], sha(&members(&first.panel)?[0]["candidate"])?)
        };
        if !self.receipt(previous, prefix, verdict)? {
            return Ok(());
        }
        let Some(run) = self.barrier.waiting(Some(previous)) else { return Ok(()) };
        let member = &members(&first.panel)?[boundary - 1];
        self.dispatched(run, sha(&member["candidate"])?)?;
        let branch = member_branch(first.panel["task"].as_u64().ok_or(String::from("landing task"))?, boundary);
        {
            let (left, right) =
                (&(get(self.forge, &format!("branches/{branch}"))["commit"]["id"]), &(sha(&member["candidate"])?));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        if gate_runs(self.forge)?.len() != boundary + 1 {
            return Err(String::from("one probe at a time, no push runs"));
        }
        if main(self.forge)? != self.base {
            return Err(String::from("no turn until the boundary is known"));
        }
        self.probes.push(run);
        self.barrier.release(run);
        self.phase = Phase::Search(if boundary == 1 { Step::Two } else { Step::Boundary });

        Ok(())
    }

    fn blamed(&self, panel: &Value) -> Result<bool, String> {
        Ok({
            let member = &members(panel)?[1];
            if member["left"].is_null() {
                return Ok(false);
            }
            if !member["landed"].is_null() {
                return Err(String::from("the blamed member did not land"));
            }
            let change = change(self.root, member["change"].as_u64().ok_or(String::from("blamed change"))?)?;
            if change["step"] != "repairing" || change["runs"].as_u64().is_none() {
                return Ok(false);
            }
            if !change["merge"].is_null() {
                return Err(String::from("assertion failed: change[\"merge\"].is_null()"));
            }
            let number = member["pull"]["number"].as_u64().ok_or(String::from("blamed pull"))?;
            let pull = get(self.forge, &format!("pulls/{number}"));
            {
                let (left, right) = (&(pull["state"]), &("open"));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            {
                let (left, right) = (&(pull["merged"]), &(false));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            true
        })
    }

    fn green_turn(&self, task: u64, prepared: &Value) -> Result<(), String> {
        // The next round replaces round.pull. Find the first turn by its member
        // branch, not by the candidate pull closed during the search.
        let branch = member_branch(task, 1);
        let pulls = get(self.forge, "pulls?state=all&limit=50");
        if pulls.as_array().ok_or(String::from("pulls"))?.len() >= 50 {
            return Err(String::from("one page holds every pull"));
        }
        let turns: Vec<_> = pulls
            .as_array()
            .ok_or(String::from("pulls"))?
            .iter()
            .filter(|pull| pull["head"]["ref"] == branch && pull["merged"] == true)
            .collect();
        if turns.len() != 1 {
            return Err(String::from("one merged landing pull from the green member's branch"));
        }
        let prefix = sha(&prepared["candidate"])?;
        {
            let (left, right) = (&(turns[0]["base"]["ref"]), &("main"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(turns[0]["head"]["sha"]), &(prefix));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        if turns[0]["merge_commit_sha"] != prefix {
            return Err(String::from("fast-forward only, no extra merge commit"));
        }

        Ok(())
    }

    fn boundary(&mut self) -> Result<(), String> {
        let first = self.first.as_ref().ok_or(String::from("top proof"))?;
        if !self.receipt(self.probes[1], sha(&members(&first.panel)?[1]["candidate"])?, "failure")? {
            return Ok(());
        }
        let Some(run) = self.barrier.waiting(Some(self.probes[1])) else { return Ok(()) };
        let task = first.panel["task"].as_u64().ok_or(String::from("landing task"))?;
        let panel = landing(self.root, task)?;
        if !self.member_landed(&members(&panel)?[0], &members(&first.panel)?[0])? {
            return Ok(());
        }
        let green = sha(&members(&first.panel)?[0]["candidate"])?;
        if main(self.forge)? != green {
            return Err(String::from("the first turn lands only the green prefix"));
        }
        self.green_turn(task, &members(&first.panel)?[0])?;
        if !self.blamed(&panel)? {
            return Ok(());
        }
        if landings(self.root)?["landings"].as_array().ok_or(String::from("landings"))?.len() != 1 {
            return Err(String::from("the suffix stays in its landing"));
        }
        {
            let (left, right) = (&(panel["phase"]), &("proving"));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(panel["round"]["number"]), &(2));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        let third = &members(&panel)?[2];
        {
            let (left, right) = (&(third["change"]), &(5));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        if !third["left"].is_null() {
            return Err(String::from("assertion failed: third[\"left\"].is_null()"));
        }
        if !third["landed"].is_null() {
            return Err(String::from("assertion failed: third[\"landed\"].is_null()"));
        }
        if sha(&third["base"])? != green {
            return Err(String::from("the next round stacks onto the first turn's tip"));
        }
        let prefix = sha(&third["candidate"])?;
        self.dispatched(run, prefix)?;
        let branch = panel["round"]["branch"].as_str().ok_or(String::from("next round's landing branch"))?;
        {
            let (left, right) = (&(get(self.forge, &format!("branches/{branch}"))["commit"]["id"]), &(prefix));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        let commit = get(self.forge, &format!("git/commits/{prefix}"));
        {
            let (left, right) = (&(commit["parents"].as_array().ok_or(String::from("parents"))?.len()), &(2));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(commit["parents"][0]["sha"]), &(green));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        {
            let (left, right) = (&(commit["parents"][1]["sha"]), &(sha(&third["head"])?));
            if left != right {
                return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
            }
        }
        if third["head"] != members(&first.panel)?[2]["head"] {
            return Err(String::from("no member head rewritten"));
        }
        if gate_runs(self.forge)?.len() != 4 {
            return Err(String::from("top, two probes and the next round only"));
        }
        self.barrier.release(run);
        self.second = Some(Proof { panel, run });
        self.phase = Phase::Search(Step::Remainder);

        Ok(())
    }

    fn remainder(&self) -> Result<bool, String> {
        Ok({
            let second = self.second.as_ref().ok_or(String::from("second round"))?;
            let third = &members(&second.panel)?[2];
            let prefix = sha(&third["candidate"])?;
            if !self.receipt(second.run, prefix, "success")? {
                return Ok(false);
            }
            let panel = landing(self.root, second.panel["task"].as_u64().ok_or(String::from("landing task"))?)?;
            if panel["phase"] != "landed"
                || !self.member_landed(&members(&panel)?[2], third)?
                || !self.blamed(&panel)?
            {
                return Ok(false);
            }
            let first = self.first.as_ref().ok_or(String::from("top proof"))?;
            if !self.member_landed(&members(&panel)?[0], &members(&first.panel)?[0])? {
                return Err(String::from(
                    "assertion failed: self.member_landed(&members(&panel)?[0], &members(&first.panel)?[0])?",
                ));
            }
            if main(self.forge)? != prefix {
                return Err(String::from("main contains the third's proved prefix"));
            }
            let number = panel["round"]["pull"]["number"].as_u64().ok_or(String::from("second turn pull"))?;
            let pull = get(self.forge, &format!("pulls/{number}"));
            {
                let (left, right) = (&(pull["merged"]), &(true));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            {
                let (left, right) = (&(pull["head"]["ref"]), &(second.panel["round"]["branch"]));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            {
                let (left, right) = (&(pull["head"]["sha"]), &(prefix));
                if left != right {
                    return Err(format!("assertion `left == right` failed\n  left: {left:?}\n right: {right:?}"));
                }
            }
            if pull["merge_commit_sha"] != prefix {
                return Err(String::from("the next turn also fast-forwards"));
            }
            if gate_runs(self.forge)?.len() != 4 {
                return Err(String::from("no merge or repair runs"));
            }
            true
        })
    }

    pub(super) fn search(&mut self, step: Step, procs: &mut [Process]) -> Result<(), String> {
        match step {
            Step::One => self.probe(1, "failure")?,
            Step::Two => self.probe(2, "success")?,
            Step::Boundary => self.boundary()?,
            Step::Remainder => {
                if self.remainder()? {
                    // The owner and repair stay held; no repair result is needed
                    // to witness the next round's turn. Terminate drains them.
                    self.stop(procs)?;
                }
            }
        }

        Ok(())
    }
}
