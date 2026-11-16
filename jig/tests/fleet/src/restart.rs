//! Restart-hold safety judged from boundary events, not the fleet's state.
use jig_domain_fleet::{Event, Request};
use skein_lib::{Duration, Time, Token};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub(crate) struct Holds {
    word: Option<Duration>,
    loaded: bool,
    discharged: bool,
    held: BTreeMap<(Token, Token), (Time, Option<Token>)>,
    listed: BTreeMap<(Token, Token), Token>,
    heard: BTreeSet<Token>,
    pub(crate) adopted: u64,
    pub(crate) lost: u64,
    pub(crate) placements: u64,
}
impl Holds {
    pub(crate) fn restart(&mut self) {
        self.word = None;
        self.loaded = false;
        self.discharged = false;
        self.held.clear();
        self.listed.clear();
        self.heard.clear();
    }
    pub(crate) fn event(&mut self, now: Time, grace: Duration, event: &Event) {
        match event {
            Event::Restarting { wait } => self.word = Some(*wait),
            Event::Hello { channel, hello, .. } => {
                for hosting in &hello.hosting {
                    let key = (hosting.run, hosting.attempt);
                    self.listed.insert(key, *channel);
                    if let Some((_, on)) = self.held.get_mut(&key) {
                        *on = Some(*channel);
                    }
                }
            }
            Event::Logins { channel, .. } => {
                self.heard.insert(*channel);
                self.held.retain(|_, (_, on)| *on != Some(*channel));
            }
            Event::Adopt { run, attempt, .. } | Event::Cross { run, attempt, .. } => {
                if !self.loaded
                    && let Some(wait) = self.word
                {
                    let key = (*run, *attempt);
                    let on = self.listed.get(&key).copied();
                    if !on.is_some_and(|channel| self.heard.contains(&channel)) {
                        self.held.insert(key, (now.saturating_add(wait).saturating_add(grace), on));
                        self.adopted += 1;
                    }
                }
            }
            Event::Lost { channel } => {
                self.listed.retain(|_, on| on != channel);
                for (_, on) in self.held.values_mut() {
                    if *on == Some(*channel) {
                        *on = None;
                    }
                }
            }
            Event::Loaded => self.loaded = true,
            _ => {}
        }
    }
    pub(crate) fn request(&mut self, now: Time, request: &Request) -> Result<(), String> {
        match request {
            Request::Lost { run, attempt, .. } => {
                if let Some((end, _)) = self.held.remove(&(*run, *attempt)) {
                    if now < end {
                        return Err(format!("held claim {run:?}/{attempt:?} lost at {now:?}, before {end:?}"));
                    }
                    self.lost += 1;
                }
            }
            Request::Assign { run, attempt, .. } | Request::AssignJob { run, attempt, .. } => {
                self.placements += 1;
                for (&(held_run, held_attempt), &(end, _)) in &self.held {
                    if held_run == *run && held_attempt != *attempt && now < end {
                        return Err(format!(
                            "next attempt placed while {held_run:?}/{held_attempt:?} held until {end:?}"
                        ));
                    }
                }
            }
            Request::Answered { run, attempt, .. }
            | Request::Acknowledge { run, attempt, .. }
            | Request::Refused { run, attempt, .. } => {
                self.held.remove(&(*run, *attempt));
            }
            Request::Discharged => {
                if !self.loaded || self.discharged {
                    return Err("discharged once and after Loaded".into());
                }
                if self.held.values().any(|&(end, _)| now < end) {
                    return Err("discharged with claims held".into());
                }
                self.discharged = true;
            }
            _ => {}
        }
        Ok(())
    }
}
