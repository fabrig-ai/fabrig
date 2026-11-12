//! The walk's early stop and output policy, without io (client/recipe.md, 5.1).
use super::{Chain, Refusal, Source, WalkLine, hex};
use crate::drive::{Answer, Exit};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Action {
    Line(Box<[u8]>),
    Print(String),
    End,
    Finished(Exit),
}
pub enum Event {
    Answer(Answer),
    Ended(Exit),
}
pub struct Walk {
    name: String,
    hash: String,
    lines: VecDeque<WalkLine>,
    current: Option<Source>,
    stopped: Option<String>,
}
impl Walk {
    pub fn start(chain: &Chain, from: Option<&str>) -> Result<(Self, Vec<Action>), Refusal> {
        let mut walk = Self {
            name: chain.recipes.last().expect("chain is nonempty").name.clone(),
            hash: hex(chain.hash()),
            lines: chain.lines(from)?.into(),
            current: None,
            stopped: None,
        };
        let first = walk.next();
        Ok((walk, vec![first]))
    }
    pub fn step(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::Answer(answer) => {
                let source = self.current.take().expect("one answer per walk line");
                let mut actions: Vec<_> =
                    answer.print().lines().map(|line| Action::Print(format!("{source} {line}"))).collect();
                if let Some(why) = answer.failed {
                    let separator = if source == Source::Walk { " " } else { ": " };
                    self.stopped = Some(format!("stopped at {source}{separator}{}", why.replace(['\n', '\r'], " ")));
                    self.lines.clear();
                    actions.push(Action::End);
                } else {
                    actions.push(self.next());
                }
                actions
            }
            Event::Ended(mut exit) => {
                if exit.status == 0 {
                    if let Some(stopped) = &self.stopped {
                        exit.status = 5;
                        exit.shown.push(stopped.clone());
                    } else {
                        exit.shown.push(format!("walked {} {}", self.name, self.hash));
                    }
                }
                vec![Action::Finished(exit)]
            }
        }
    }
    fn next(&mut self) -> Action {
        match self.lines.pop_front() {
            Some(line) => {
                self.current = Some(line.source);
                Action::Line(line.line)
            }
            None => Action::End,
        }
    }
}
impl Chain {
    /// Give the whole chain's scripts to the session, even when starting from a copy.
    #[must_use]
    pub fn scripts(&self, setups: &Path) -> Vec<PathBuf> {
        self.recipes
            .iter()
            .filter(|recipe| recipe.scripts.is_some())
            .map(|recipe| setups.join(&recipe.name).join("scripts.json"))
            .collect()
    }
}
/// Check both directories before the drive session can start or prepare anything.
pub(super) fn check_scratch(from: Option<&str>, config: bool, data: bool) -> Result<(), String> {
    if from.is_some() {
        if !config || !data {
            return Err("--from requires config/ and data/ directories in scratch".into());
        }
    } else if config || data {
        return Err("fresh walk requires no config/ and no data/ in scratch".into());
    }
    Ok(())
}
