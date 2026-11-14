//! The observed red boundary, its green turn and the remaining member's round
//! (temper/docs/design/domain/landing-search.md, 3–5 and 8).
use super::{
    Judge, Phase, Process, Proof, Script, Turn, Value, gate_runs, get, hold, main, member_branch, members, sha,
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
            let states = statuses["statuses"].as_array().map(|rows| {
                rows.iter()
                    .map(|status| serde_json::json!({"context":status["context"],"status":status["status"]}))
                    .collect::<Vec<_>>()
            });
            self.observe(
                run,
                serde_json::json!({"prefix":prefix,"wanted":verdict,"run":actual["status"],"states":states}),
            );
            if statuses["total_count"] == 0 && statuses["statuses"].is_null() {
                return Ok(false);
            }
            statuses["statuses"]
                .as_array()
                .ok_or_else(|| format!("statuses: {statuses}"))?
                .iter()
                .any(|status| status["context"] == "temper/gate/gate" && status["status"] == verdict)
        })
    }

    fn probe(&mut self, boundary: usize, verdict: &str) -> Result<(), String> {
        let first = self.first.as_ref().ok_or(String::from("top proof"))?;
        let (previous, prefix) = if self.probes.is_empty() {
            (first.run, super::candidate(&first.panel)?)
        } else {
            let index = usize::from(self.story.ready().len() == 4);
            (self.probes[0], sha(&members(&first.panel)?[index]["candidate"])?)
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
        if gate_runs(self.forge)?.len() != self.probes.len() + 2 {
            return Err(String::from("one probe at a time, no push runs"));
        }
        if main(self.forge)? != self.base {
            return Err(String::from("no turn until the boundary is known"));
        }
        self.probes.push(run);
        self.barrier.release(run);
        self.phase = Phase::Search(if self.probes.len() == 1 { Step::Two } else { Step::Boundary });

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
            let change = self.change(member["change"].as_u64().ok_or(String::from("blamed change"))?)?;
            self.observe(0, serde_json::json!({"step":change["step"],"runs":change["runs"],"hold":change["hold"]}));
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
        let red = if self.story.ready().len() == 4 { self.probes[0] } else { self.probes[1] };
        if !self.receipt(red, sha(&members(&first.panel)?[1]["candidate"])?, "failure")? {
            return Ok(());
        }
        if self.story.ready().len() == 4
            && !self.receipt(self.probes[1], sha(&members(&first.panel)?[0]["candidate"])?, "success")?
        {
            return Ok(());
        }
        let Some(run) = self.barrier.waiting(Some(self.probes[1])) else { return Ok(()) };
        let task = first.panel["task"].as_u64().ok_or(String::from("landing task"))?;
        let panel = self.landing(task)?;
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
        if self.landings()?["landings"].as_array().ok_or(String::from("landings"))?.len() != 1 {
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
        let mut below = green.to_owned();
        for (index, member) in members(&panel)?.iter().enumerate().skip(2) {
            let prefix = sha(&member["candidate"])?;
            let commit = get(self.forge, &format!("git/commits/{prefix}"));
            if commit["parents"].as_array().ok_or("parents")?.len() != 2
                || commit["parents"][0]["sha"] != below
                || commit["parents"][1]["sha"] != sha(&member["head"])?
            {
                return Err("the remaining stack preserves its heads on the green prefix".into());
            }
            if member["head"] != members(&first.panel)?[index]["head"] {
                return Err("no member head rewritten".into());
            }
            prefix.clone_into(&mut below);
        }
        let prefix = super::candidate(&panel)?;
        self.dispatched(run, prefix)?;
        let branch = panel["round"]["branch"].as_str().ok_or("next round branch")?;
        if get(self.forge, &format!("branches/{branch}"))["commit"]["id"] != prefix {
            return Err("the round branch names its top".into());
        }
        if gate_runs(self.forge)?.len() != 4 {
            return Err(String::from("top, two probes and the next round only"));
        }
        // The held branch still names the pull before Forgejo archives its ref.
        let number = panel["round"]["pull"]["number"].as_u64().ok_or(String::from("round pull"))?;
        let pull = get(self.forge, &format!("pulls/{number}"));
        if pull["head"]["ref"] != branch || pull["head"]["sha"] != prefix {
            return Err(String::from("the next round's pull uses its held branch and proved prefix"));
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
            let prefix = super::candidate(&second.panel)?;
            if !self.receipt(second.run, prefix, "success")? {
                return Ok(false);
            }
            let panel = self.landing(second.panel["task"].as_u64().ok_or(String::from("landing task"))?)?;
            if panel["phase"] != "landed"
                || !self.member_landed(&members(&panel)?[2], third)?
                || !self.blamed(&panel)?
            {
                return Ok(false);
            }
            for (landed, prepared) in members(&panel)?.iter().zip(members(&second.panel)?).skip(2) {
                if !self.member_landed(landed, prepared)? {
                    return Ok(false);
                }
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
            // A deleted branch is retained by Forgejo as the pull's archive ref.
            if pull["head"]["ref"] != second.panel["round"]["branch"]
                && pull["head"]["ref"] != format!("refs/pull/{number}/head")
            {
                return Err(String::from("the merged pull keeps its branch or its archive ref"));
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
            Step::One => self.probe(if self.story.ready().len() == 4 { 2 } else { 1 }, "failure")?,
            Step::Two => {
                if self.story.ready().len() == 4 {
                    self.probe(1, "failure")?;
                } else {
                    self.probe(2, "success")?;
                }
            }
            Step::Boundary => self.boundary()?,
            Step::Remainder => {
                if self.remainder()? {
                    // The repair stays held; no repair result is needed
                    // to witness the next round's turn. Terminate drains them.
                    self.stop(procs)?;
                }
            }
        }

        Ok(())
    }
}
