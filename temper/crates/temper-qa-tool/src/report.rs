//! The pass report, built only from records (qa/passes.md, 8).
use crate::{Home, Ledger, Refusal, refused};
use serde_json::{Value, json};

impl Ledger {
    pub fn report(&self, number: u64) -> Result<Value, Refusal> {
        let pass = self
            .passes
            .iter()
            .find(|pass| pass.number == number)
            .ok_or_else(|| refused(format!("no pass {number}")))?;
        let cases = pass.cases.iter().map(|number| {
            let case = self.cases.iter().find(|case| case.number == *number).expect("pass case");
            let mut value = json!({"case":case.number,"surface":case.surface,"page":case.page,"landings":case.landings(),
                "expected":case.asks.iter().flat_map(|ask| ask.steps.iter().map(|step| &step.expect)).collect::<Vec<_>>()});
            if let Some(outcome) = case.outcomes.last() {
                value["outcome"] = json!(outcome.result.name());
                value["notes"] = json!(outcome.notes);
                value["findings"] = json!(outcome.findings);
            }
            value
        }).collect::<Vec<_>>();
        let findings = self.findings.iter().filter(|finding| finding.pass == number).map(|finding| {
            let mut value = json!({"finding":finding.number,"title":finding.title,"severity":finding.severity,"area":finding.area,
                "landings":finding.landings,"case":finding.case,"setup":finding.setup,"reproduction":finding.lines,
                "expected":finding.expected,"seen":finding.seen,"evidence":finding.evidence,"state":finding.state.name(),"goals":finding.goals});
            if let Some(linked) = finding.to { value["linked"] = json!(linked); }
            value
        }).collect::<Vec<_>>();
        Ok(
            json!({"pass":number,"place":pass.place,"build":pass.build,"state":pass.state.name(),"cases":cases,"findings":findings,
            "notes":self.notes.iter().filter(|note| note.pass == number).map(|note| &note.words).collect::<Vec<_>>()}),
        )
    }
}
impl Home {
    /// Reads and writes under the ledger's read lock; the loop calls this after ending a session.
    pub fn write_pass_report(&self, number: u64) -> Result<(), Refusal> {
        self.read(|ledger| {
            let report = ledger.report(number)?;
            let directory = self.directory.join("passes").join(number.to_string());
            let bytes = serde_json::to_vec_pretty(&report).map_err(|why| refused(format!("report: {why}")))?;
            let write = || -> std::io::Result<()> {
                std::fs::create_dir_all(&directory)?;
                std::fs::write(directory.join("report.tmp"), bytes)?;
                std::fs::rename(directory.join("report.tmp"), directory.join("report.json"))
            };
            write().map_err(|why| refused(format!("passes/{number}/report.json: {why}")))
        })
    }
}
