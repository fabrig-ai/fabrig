//! The domain's lasting state (skein/docs/design/testing/fakes/forge.md, 2.7).
//!
//! Big-endian counts prefix byte strings and collections. Tagged options and
//! enums have explicit discriminants. Repository records are in slot order;
//! indexes and handles are rebuilt. Timers store durations from the last pass,
//! and the restored world's deadline origin is zero. Reply rights, deliveries,
//! rate windows, observations and tallies do not cross this boundary.
mod codec;
mod fields;
mod operations;
mod repositories;
mod validate;

use crate::{
    Config, Domain,
    domain::{Alarm, Call, Landing, State},
    git::Object,
    store::Repository,
};
use alloc::boxed::Box;
use codec::{Sink, bytes, count, optional_u64, require, time, u32, u64};
use skein_lib::{Id, Reader, Rng, Writer};

/// Why an image cannot be taken or restored. No partially restored domain escapes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// A call still owns its reply right.
    Busy,
    /// The named shape or reference is not an image of a domain.
    Malformed(&'static str),
    /// The named collection or payload exceeds the supplied configuration.
    Limits(&'static str),
}

impl Domain {
    /// Takes the lasting state, unless a call still awaits its reply.
    pub fn image(&self) -> Result<Box<[u8]>, Refusal> {
        for (_, call) in &self.calls {
            match &call.state {
                State::Waiting { .. } | State::Copying { .. } => return Err(Refusal::Busy),
                State::Landing(_) | State::Closed => {}
            }
        }
        let mut measure = Sink::Measure(0);
        put(&mut measure, self);
        let size = match measure {
            Sink::Measure(size) => size,
            Sink::Write(_) => unreachable!("measurement sink"),
        };
        if u32::try_from(size).is_err() {
            return Err(Refusal::Limits("image bytes"));
        }
        let mut out = Sink::Write(Writer::new(size));
        put(&mut out, self);
        match out {
            Sink::Write(writer) => Ok(writer.finish()),
            Sink::Measure(_) => unreachable!("writing sink"),
        }
    }

    /// Restores lasting state under `config`, with no calls awaiting replies,
    /// hooks, rate windows or undrained observations. Deadlines start at zero;
    /// forge timestamps have the saved reading as their floor.
    pub fn from_image(config: &Config, bytes: &[u8]) -> Result<Domain, Refusal> {
        validate::config(config)?;
        if u32::try_from(bytes.len()).is_err() {
            return Err(Refusal::Malformed("image length"));
        }
        let mut input = Reader::new(bytes);
        require(input.bytes(8) == Some(b"FORGEIMG".as_slice()), "image format")?;
        let mut domain = Domain::new(config, 0);
        let limits = &config.limits;
        for _ in 0..count(&mut input, limits.repositories, "repositories")? {
            let repository = repositories::get(&mut input, limits)?;
            let gone = repository.gone;
            let name = repository.name.clone();
            let id = domain.repositories.insert(repository).expect("bounded repositories");
            if !gone {
                require(
                    domain.names.insert(name, id).expect("one name per repository").is_none(),
                    "duplicate repository name",
                )?;
            }
        }
        for _ in 0..count(&mut input, limits.commits, "commits")? {
            let id = u64(&mut input)?;
            let object = get_object(&mut input, limits)?;
            require(domain.commits.insert(id, object).expect("bounded store").is_none(), "duplicate commit")?;
        }
        domain.made = u64(&mut input)?;
        domain.comments = u64(&mut input)?;
        domain.reviews = u64(&mut input)?;
        domain.runs = u64(&mut input)?;
        domain.jobs = u64(&mut input)?;
        domain.clock = time(&mut input)?;
        domain.floor = domain.clock;
        domain.rng = Rng::new(u64(&mut input)?);
        validate::domain(&domain, limits)?;
        for _ in 0..count(&mut input, crate::limits::timers(limits).ok_or(Refusal::Limits("timers"))?, "timers")? {
            get_alarm(&mut input, &mut domain, limits)?;
        }
        if !input.is_empty() {
            require(input.bytes(8) == Some(b"MANUAL01".as_slice()), "trailing bytes")?;
            let rows = count(&mut input, limits.repositories, "manual merge settings")?;
            require(rows == domain.repositories.len(), "manual merge settings")?;
            for index in 0..rows {
                let settings = crate::api::ManualMerges {
                    allow_manual_merge: codec::flag(&mut input)?,
                    autodetect_manual_merge: codec::flag(&mut input)?,
                };
                let (id, _) = domain.repositories.iter().nth(skein_lib::usize_of(index)).expect("bounded repository");
                domain.repositories.get_mut(id).expect("retained repository").manual_merges = settings;
            }
        }
        require(input.is_empty(), "trailing bytes")?;
        Ok(domain)
    }
}
fn put(out: &mut Sink, domain: &Domain) {
    out.put(b"FORGEIMG");
    out.u32(domain.repositories.len());
    for (_, r) in &domain.repositories {
        repositories::put(out, r);
    }
    out.u32(domain.commits.len());
    for (id, object) in &domain.commits {
        out.u64(*id);
        put_object(out, object);
    }
    for counter in [domain.made, domain.comments, domain.reviews, domain.runs, domain.jobs] {
        out.u64(counter);
    }
    out.u64(domain.clock.as_nanos());
    out.u64(domain.rng.state());
    let mut n = 0u32;
    for (alarm, _) in domain.timers.iter() {
        if retained(alarm) {
            n = n.checked_add(1).expect("bounded timers");
        }
    }
    out.u32(n);
    for (alarm, at) in domain.timers.iter() {
        if retained(alarm) {
            out.u64(at.saturating_since(domain.now).as_nanos());
            put_alarm(out, domain, alarm);
        }
    }
    let mut manual = false;
    for (_, repository) in &domain.repositories {
        if repository.manual_merges != crate::api::ManualMerges::OFF {
            manual = true;
        }
    }
    if manual {
        out.put(b"MANUAL01");
        out.u32(domain.repositories.len());
        for (_, repository) in &domain.repositories {
            out.flag(repository.manual_merges.allow_manual_merge);
            out.flag(repository.manual_merges.autodetect_manual_merge);
        }
    }
}
fn retained(alarm: Alarm) -> bool {
    match alarm {
        Alarm::Hook(_) => false,
        Alarm::Call(_) | Alarm::Check { .. } | Alarm::Rerun { .. } | Alarm::Workflow { .. } | Alarm::Queue(_) => true,
    }
}
fn put_object(out: &mut Sink, object: &Object) {
    out.optional_u64(object.parent);
    out.optional_u64(object.merge_parent);
    out.u32(object.tree.len());
    for (name, content) in &object.tree {
        out.bytes(name);
        out.bytes(content);
    }
    out.bytes(&object.message);
    fields::put_optional_signature(out, object.author.as_ref());
    fields::put_optional_signature(out, object.committer.as_ref());
}
fn get_object(input: &mut Reader<'_>, limits: &crate::Limits) -> Result<Object, Refusal> {
    let parent = optional_u64(input)?;
    let merge_parent = optional_u64(input)?;
    let mut tree = crate::Tree::with_capacity(limits.files);
    for _ in 0..count(input, limits.files, "files")? {
        let path = bytes(input, limits.name_bytes, "file path")?;
        let content = bytes(input, limits.content_bytes, "file content")?;
        require(tree.insert(path, content).expect("bounded files").is_none(), "duplicate path")?;
    }
    let message =
        bytes(input, limits.title_bytes.saturating_add(limits.body_bytes).saturating_add(2), "commit message")?;
    let author = fields::get_optional_signature(input, limits)?;
    let committer = fields::get_optional_signature(input, limits)?;
    Ok(Object { parent, merge_parent, tree, message, author, committer })
}
fn put_alarm(out: &mut Sink, domain: &Domain, alarm: Alarm) {
    match alarm {
        Alarm::Call(id) => {
            out.u8(0);
            match &domain.calls.get(id).expect("timer's call").state {
                State::Landing(landing) => {
                    out.u64(landing.user);
                    out.bytes(&landing.repository);
                    operations::put(out, &landing.op);
                }
                State::Waiting { .. } | State::Copying { .. } | State::Closed => {
                    unreachable!("only answered late landings can be imaged")
                }
            }
        }
        Alarm::Check { repository, commit, context } => {
            out.u8(1);
            out.u32(slot(domain, repository));
            out.u64(commit);
            out.u32(context);
        }
        Alarm::Rerun { repository, commit, context } => {
            out.u8(2);
            out.u32(slot(domain, repository));
            out.u64(commit);
            out.u32(context);
        }
        Alarm::Workflow { repository, run, job } => {
            out.u8(3);
            out.u32(slot(domain, repository));
            out.u64(run);
            out.u64(job);
        }
        Alarm::Queue(id) => {
            out.u8(4);
            out.u32(slot(domain, id));
        }
        Alarm::Hook(_) => unreachable!("in-flight hooks are dropped"),
    }
}
fn slot(domain: &Domain, id: Id<Repository>) -> u32 {
    for (slot, (candidate, _)) in domain.repositories.iter().enumerate() {
        if candidate == id {
            return u32::try_from(slot).expect("bounded repository slots");
        }
    }
    unreachable!("timer holds a retained repository")
}
fn repository(input: &mut Reader<'_>, domain: &Domain) -> Result<Id<Repository>, Refusal> {
    let slot = u32(input)?;
    for (index, (id, _)) in domain.repositories.iter().enumerate() {
        if u32::try_from(index).expect("bounded slots") == slot {
            return Ok(id);
        }
    }
    Err(Refusal::Malformed("timer repository"))
}
fn get_alarm(input: &mut Reader<'_>, domain: &mut Domain, limits: &crate::Limits) -> Result<(), Refusal> {
    let at = time(input)?;
    let tag = codec::u8(input)?;
    let alarm = match tag {
        0 => {
            if domain.calls.is_full() {
                return Err(Refusal::Limits("late landings"));
            }
            let user = u64(input)?;
            let repository = bytes(input, limits.name_bytes, "landing repository")?;
            let op = operations::get(input, limits)?;
            let id = domain
                .calls
                .insert(Call { state: State::Landing(Landing { user, repository, op }) })
                .expect("bounded landings");
            Alarm::Call(id)
        }
        1 | 2 => {
            let rerun = tag == 2;
            let repository = repository(input, domain)?;
            let commit = u64(input)?;
            let context = u32(input)?;
            validate::check_alarm(domain, repository, commit, context)?;
            if rerun {
                Alarm::Rerun { repository, commit, context }
            } else {
                Alarm::Check { repository, commit, context }
            }
        }
        3 => {
            let repository = repository(input, domain)?;
            let run = u64(input)?;
            let job = u64(input)?;
            validate::workflow_alarm(domain, repository, run, job)?;
            Alarm::Workflow { repository, run, job }
        }
        4 => Alarm::Queue(repository(input, domain)?),
        _tag => return Err(Refusal::Malformed("timer kind")),
    };
    for (key, _) in domain.timers.iter() {
        require(key != alarm, "duplicate timer")?;
    }
    domain.timers.arm(alarm, at).or(Err(Refusal::Limits("timers")))
}
