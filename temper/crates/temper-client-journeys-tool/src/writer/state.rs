//! The writer's offer, last finished run and cadence (temper/docs/design/client/journey-writer.md, 4).

use std::{os::unix::ffi::OsStrExt as _, path::Path};

use crate::{
    host::{Answer, Ask, Host, Program, Stream},
    looks::hash,
};

use super::{
    Checkout, Phase, Stop, changed, checked, child, chomp, decimal, diagnostic, fail, git_path, phase, refuse,
};

const MAIN: &[u8] = b"origin/main";
const HOUR: u64 = 3600;

pub(super) struct Run {
    pub(super) tip: String,
    pub(super) seconds: u64,
}

struct Offer {
    kind: Offered,
    run: Option<OfferedRun>,
}

struct OfferedRun {
    run: Run,
    commit: String,
}

enum Offered {
    Commit(String),
    Task(String),
}

impl Offer {
    fn read(bytes: &[u8]) -> Option<Self> {
        // Accept the earlier one-line form, including its CR and CRLF endings.
        let text = std::str::from_utf8(bytes).ok()?;
        let mut lines = text.strip_suffix('\r').unwrap_or(text).lines();
        let (kind, value) = lines.next()?.split_once(' ')?;
        let kind = match kind {
            "task" if decimal(value.as_bytes()) => Offered::Task(value.into()),
            "commit" if hash(value) => Offered::Commit(value.into()),
            _ => return None,
        };
        let mut offer = Self { kind, run: None };
        if let Some(line) = lines.next() {
            let mut fields = line.split(' ');
            let (Some("run"), Some(tip), Some(commit), Some(start), None) =
                (fields.next(), fields.next(), fields.next(), fields.next(), fields.next())
            else {
                return None;
            };
            if !hash(commit) || lines.next().is_some() || matches!(&offer.kind, Offered::Commit(kept) if kept != commit)
            {
                return None;
            }
            offer.run = Some(OfferedRun { run: Run::read(tip, start)?, commit: commit.into() });
        }
        Some(offer)
    }
}

impl Run {
    fn read(tip: &str, seconds: &str) -> Option<Self> {
        if !hash(tip) || !decimal(seconds.as_bytes()) {
            return None;
        }
        Some(Self { tip: tip.into(), seconds: seconds.parse().ok()? })
    }
}

impl Checkout {
    pub(super) fn plan(&self, host: &mut Host, offer_path: &Path) -> Result<Option<Run>, Stop> {
        let mut offer = self.read_offer(host, offer_path)?;
        let mut fetched = false;
        let mut landed = false;
        if let Some(offer) = &mut offer {
            match &offer.kind {
                Offered::Task(task) => {
                    if self.waiting_on_task(host, task.as_bytes())? {
                        return Ok(None);
                    }
                }
                Offered::Commit(commit) => {
                    let tip = if let Some(offered) = &offer.run {
                        offered.run.tip.as_bytes().into()
                    } else {
                        self.legacy_tip(host, commit.as_bytes(), offer_path)?
                    };
                    self.fetch(host)?;
                    fetched = true;
                    if !self.holds(host, commit.as_bytes(), &tip, offer.run.is_none())? {
                        host.out(format!("waiting on {commit}\n").as_bytes());
                        return Ok(None);
                    }
                    landed = true;
                }
            }
            if offer.run.is_none() {
                changed(host, Ask::Remove(self.offer.clone()))?;
            }
        }

        self.clean_checkout(host)?;
        if !fetched {
            self.fetch(host)?;
        }
        let tip = self.git(host, &[b"rev-parse", MAIN], Stream::Keep)?;
        let tip = String::from_utf8_lossy(chomp(&tip));
        if !hash(&tip) {
            return Err(fail(&[b"cannot read origin/main's tip"]));
        }
        let now = match host.ask(Ask::Clock) {
            Answer::Seconds(seconds) => seconds,
            Answer::Failed(why) => return Err(Stop::Host(why)),
            _ => unreachable!("host answers Clock with Seconds or Failed"),
        };
        let mut last = None;
        if let Some(OfferedRun { run, commit }) = offer.and_then(|offer| offer.run) {
            if landed || self.holds(host, commit.as_bytes(), run.tip.as_bytes(), false)? {
                self.finish_run(host, &run)?;
                last = Some(run);
            }
            changed(host, Ask::Remove(self.offer.clone()))?;
        }
        if let Some(last) = last.or_else(|| self.last_run(host))
            && !self.due(host, &last, &tip, now)?
        {
            return Ok(None);
        }
        Ok(Some(Run { tip: tip.into_owned(), seconds: now }))
    }

    fn read_offer(&self, host: &mut Host, shown: &Path) -> Result<Option<Offer>, Stop> {
        match host.ask(Ask::Exists(self.offer.clone())) {
            Answer::Is(false) => return Ok(None),
            Answer::Is(true) => (),
            Answer::Failed(why) => return Err(Stop::Host(why)),
            _ => unreachable!("host answers Exists with Is or Failed"),
        }
        if let Answer::Bytes(bytes) = host.ask(Ask::Read(self.offer.clone()))
            && let Some(offer) = Offer::read(&bytes)
        {
            return Ok(Some(offer));
        }
        Err(fail(&[b"cannot read kept offer in ", shown.as_os_str().as_bytes()]))
    }

    fn waiting_on_task(&self, host: &mut Host, task: &[u8]) -> Result<bool, Stop> {
        let doc = checked(host, self.here(Program::new(b"temper", &[b"ctl", b"task", b"show", task, b"--json"])))?;
        match phase(&doc) {
            Some(Phase::Ended) => Ok(false),
            Some(Phase::Live) => {
                host.out(&[b"waiting on change ".as_slice(), task, b"\n"].concat());
                Ok(true)
            }
            None => Err(fail(&[b"cannot read the task document's phase"])),
        }
    }

    fn legacy_tip(&self, host: &mut Host, commit: &[u8], offer: &Path) -> Result<Box<[u8]>, Stop> {
        let mut cat = self.here(Program::new(b"git", &[b"cat-file", b"-e", &[commit, b"^{commit}"].concat()]));
        cat.stderr = Stream::Keep;
        if checked(host, cat).is_err() {
            return Err(fail(&[b"kept commit ", commit, b" is missing (", offer.as_os_str().as_bytes(), b")"]));
        }
        let parent = self
            .git(host, &[b"rev-parse", &[commit, b"^"].concat()], Stream::Keep)
            .map_err(|_| fail(&[b"cannot read parent of kept commit ", commit]))?;
        Ok(chomp(&parent).into())
    }

    fn holds(&self, host: &mut Host, commit: &[u8], tip: &[u8], legacy: bool) -> Result<bool, Stop> {
        if self.ancestor(host, commit, MAIN)? {
            return Ok(true);
        }
        if legacy {
            // One-line offers recognize the earlier form, where Printed could be any message line.
            let messages = self.git(host, &[b"log", b"--format=%B", MAIN], Stream::Keep)?;
            let printed = [b"Printed: ".as_slice(), tip].concat();
            return Ok(messages.split(|&byte| byte == b'\n').any(|line| line == printed));
        }
        let trailers = self.git(
            host,
            &[b"log", b"--topo-order", b"--format=%(trailers:key=Printed,valueonly,unfold)", MAIN],
            Stream::Keep,
        )?;
        Ok(trailers.split(|&byte| byte == b'\n').any(|value| value.trim_ascii() == tip))
    }

    fn clean_checkout(&self, host: &mut Host) -> Result<(), Stop> {
        for state in [b"rebase-merge".as_slice(), b"rebase-apply", b"MERGE_HEAD", b"CHERRY_PICK_HEAD", b"sequencer"] {
            let path = git_path(host, &self.root, state)?;
            match host.ask(Ask::Exists(self.root.join(path))) {
                Answer::Is(true) => return Err(refuse(&[state, b" is in progress"])),
                Answer::Is(false) => (),
                Answer::Failed(why) => return Err(Stop::Host(why)),
                _ => unreachable!("host answers Exists with Is or Failed"),
            }
        }
        let tracked = self.git(host, &[b"status", b"--porcelain", b"--untracked-files=no"], Stream::Keep)?;
        if !chomp(&tracked).is_empty() {
            return Err(refuse(&[b"tracked files must be clean"]));
        }
        Ok(())
    }

    fn last_run(&self, host: &mut Host) -> Option<Run> {
        match host.ask(Ask::Read(self.finished.clone())) {
            Answer::Missing => (),
            Answer::Bytes(bytes) => {
                let run = std::str::from_utf8(&bytes).ok().and_then(|text| {
                    let mut lines = text.lines();
                    let (tip, seconds) = lines.next()?.split_once(' ')?;
                    if lines.next().is_some() {
                        return None;
                    }
                    Run::read(tip, seconds)
                });
                if run.is_some() {
                    return run;
                }
                diagnostic(
                    host,
                    &[
                        b"ignoring unreadable ".as_slice(),
                        self.finished.as_os_str().as_bytes(),
                        b": expected TIP SECONDS",
                    ]
                    .concat(),
                );
            }
            Answer::Failed(why) => diagnostic(host, &[b"ignoring unreadable ".as_slice(), why.as_bytes()].concat()),
            _ => unreachable!("host answers Read with Bytes, Missing or Failed"),
        }
        None
    }

    fn due(&self, host: &mut Host, last: &Run, tip: &str, now: u64) -> Result<bool, Stop> {
        if last.tip == tip {
            current(host, tip.as_bytes());
            return Ok(false);
        }
        if now.saturating_sub(last.seconds) >= HOUR || !self.ancestor(host, last.tip.as_bytes(), tip.as_bytes())? {
            return Ok(true);
        }
        let range = format!("{}..{tip}", last.tip);
        let keys =
            self.git(host, &[b"log", b"--format=%(trailers:key=Look-landed,keyonly)", range.as_bytes()], Stream::Keep)?;
        if !keys.trim_ascii().is_empty() {
            return Ok(true);
        }
        host.out(format!("waiting on the hour after {:.12}\n", last.tip).as_bytes());
        Ok(false)
    }

    fn ancestor(&self, host: &mut Host, older: &[u8], newer: &[u8]) -> Result<bool, Stop> {
        let program = self.here(Program::new(b"git", &[b"merge-base", b"--is-ancestor", older, newer]));
        match child(host, program)?.code {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(Stop::Child),
        }
    }

    pub(super) fn finish_run(&self, host: &mut Host, run: &Run) -> Result<(), Stop> {
        let record = format!("{} {}\n", run.tip, run.seconds);
        changed(host, Ask::Write(self.finished.clone(), record.into_bytes().into_boxed_slice()))
    }
}

pub(super) fn current(host: &mut Host, tip: &[u8]) {
    host.out(&[b"main's pages are ".as_slice(), tip, b"'s print\n"].concat());
}
