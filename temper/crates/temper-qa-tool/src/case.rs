//! A triage case file (qa/triage.md, 4; qa/ledger.md, 4.3 and 5).
use crate::{Refusal, Session, document, refused};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseInput {
    pub surface: String,
    pub page: String,
    pub setup: String,
    pub adds: String,
    pub steps: Vec<Step>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub act: String,
    pub expect: String,
    pub see: Option<String>,
}
impl CaseInput {
    /// Syntax errors name the input; semantic errors use section 5's words.
    pub fn decode(bytes: &[u8], file: &str) -> Result<Self, Refusal> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|why| refused(format!("case refused: {file}: {why}")))?;
        Self::from_value(&value)
    }
    fn from_value(value: &Value) -> Result<Self, Refusal> {
        let decode = || {
            let object = document::object(value, &["surface", "page", "setup", "adds", "steps"])?;
            let surface = field(object, "surface", "no surface")?;
            check_surface(&surface)?;
            let page = field(object, "page", "no page")?;
            let setup = field(object, "setup", "no setup")?;
            let adds = object.get("adds").map(document::string).transpose()?.unwrap_or_default();
            let values = object.get("steps").ok_or_else(|| refused("no step"))?;
            let values = document::array(values)?;
            if values.is_empty() {
                return Err(refused("no step"));
            }
            let mut steps = Vec::new();
            for (index, value) in values.iter().enumerate() {
                let number = index + 1;
                let object = document::object(value, &["do", "expect", "see"])?;
                steps.push(Step {
                    act: field(object, "do", &format!("step {number} says nothing to do"))?,
                    expect: field(object, "expect", &format!("step {number} has no expectation"))?,
                    see: object.get("see").map(document::string).transpose()?,
                });
            }
            Ok(Self { surface, page, setup, adds, steps })
        };
        decode().map_err(|why: Refusal| refused(format!("case refused: {why}")))
    }
    pub fn check(&self, session: &Session) -> Result<(), Refusal> {
        let check = || {
            if self.surface.trim().is_empty() {
                return Err(refused("no surface"));
            }
            check_surface(&self.surface)?;
            if self.page.trim().is_empty() {
                return Err(refused("no page"));
            }
            let pages = session.pages.get(&self.surface).map_or(&[][..], Vec::as_slice);
            if !pages.contains(&self.page) {
                return Err(refused(format!("{} is no {} page; pages: {}", self.page, self.surface, pages.join(", "))));
            }
            if self.setup.trim().is_empty() {
                return Err(refused("no setup"));
            }
            if !session.setups.contains(&self.setup) {
                return Err(refused(format!(
                    "setup {} is not in the catalogue; setups: {}",
                    self.setup,
                    session.setups.join(", ")
                )));
            }
            if self.steps.is_empty() {
                return Err(refused("no step"));
            }
            for (index, step) in self.steps.iter().enumerate() {
                let number = index + 1;
                if step.act.trim().is_empty() {
                    return Err(refused(format!("step {number} says nothing to do")));
                }
                if step.expect.trim().is_empty() {
                    return Err(refused(format!("step {number} has no expectation")));
                }
            }
            Ok(())
        };
        check().map_err(|why: Refusal| refused(format!("case refused: {why}")))
    }
}
fn field(object: &serde_json::Map<String, Value>, name: &str, absent: &str) -> Result<String, Refusal> {
    let value = object.get(name).and_then(Value::as_str).filter(|value| !value.trim().is_empty());
    value.map(String::from).ok_or_else(|| refused(absent))
}
fn check_surface(surface: &str) -> Result<(), Refusal> {
    if surface != "web" && surface != "tui" {
        return Err(refused(format!("surface {surface} is neither web nor tui")));
    }
    Ok(())
}
