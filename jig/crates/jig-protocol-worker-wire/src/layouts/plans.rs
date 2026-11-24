//! Per-wire opening checks and text cache (jig/docs/design/protocol/channel/layouts.md, 3).
use super::{MAX_TEXTS, Nested, Pass, Peer, Theirs, Value, table, words};
use crate::{Root, wire};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Mode, Plan, Why, Workspace},
    schema::{self, DeclIx, Named, Schema, Shape},
};
use skein_lib::{Decimal, List, Queue, bytes};

const PIECES: &str = smith_channel::pieces::wire::FAMILY;
const WIRE_TEXT: usize = 65_536;

/// The engine's parsed texts, roots, and the plans derived so far for each wire.
#[derive(Debug)]
pub struct Plans {
    wires: List<EngineWire>,
}

#[derive(Debug)]
struct EngineWire {
    name: Box<str>,
    layout: [u8; 32],
    schema: Schema,
    needs: List<Need>,
    known: Queue<Cached>,
    #[cfg(test)]
    derivations: u32,
}

#[derive(Clone, Copy, Debug)]
struct Need {
    declaration: &'static str,
    value: Value,
    mode: Mode,
}

#[derive(Clone, Debug)]
struct Derived {
    frames: Option<table::Frames>,
    passes: List<Pass>,
}

#[derive(Debug)]
struct Cached {
    layout: [u8; 32],
    text: Box<[u8]>,
    derived: Derived,
}

impl Plans {
    /// Parses the engine's texts in order and checks its application's roots.
    pub fn new(texts: &[&'static str], roots: &List<Root>) -> Result<Plans, Box<[u8]>> {
        let mut wires = List::uncapped();
        for text in texts {
            let schema = match schema::parse(text.as_bytes()) {
                Ok(schema) => schema,
                Err(error) => {
                    return Err(words::join(
                        &[
                            b"the engine's wire text does not parse at line ",
                            Decimal::of(u64::from(error.line)).as_bytes(),
                        ],
                        256,
                    ));
                }
            };
            let name = table::text(schema.spelled(schema.family()));
            for earlier in &wires {
                let earlier: &EngineWire = earlier;
                if earlier.name.as_ref() == name {
                    return Err(words::wire(name, &[b": stated twice"]));
                }
            }
            let layout = schema.fingerprints().family().digest();
            let mut needs = List::uncapped();
            if name == wire::FAMILY {
                for mode in [Mode::Read, Mode::Keep] {
                    let (_, declaration) = table::direction(mode);
                    add_need(name, &schema, &mut needs, Need { declaration, value: Value::Frame, mode })?;
                }
            }
            for (nested, mode) in [
                (Nested::Charter, Mode::Keep),
                (Nested::Result, Mode::Read),
                (Nested::Turn, Mode::Read),
                (Nested::Turn, Mode::Keep),
            ] {
                let (family, declaration) = nested.site();
                if name == family {
                    add_need(name, &schema, &mut needs, Need { declaration, value: Value::Part(nested), mode })?;
                }
            }
            wires
                .push(EngineWire {
                    name: Box::from(name),
                    layout,
                    schema,
                    needs,
                    known: Queue::with_capacity(MAX_TEXTS),
                    #[cfg(test)]
                    derivations: 0,
                })
                .expect("one engine wire per text");
        }
        for root in roots {
            let mut found = false;
            for wire in wires.iter_mut() {
                if wire.name.as_ref() == root.wire {
                    let nested = Nested::Carried(root.carried);
                    let mode = if nested.crosses(Mode::Read) { Mode::Read } else { Mode::Keep };
                    add_need(
                        &wire.name,
                        &wire.schema,
                        &mut wire.needs,
                        Need { declaration: root.declaration, value: Value::Part(nested), mode },
                    )?;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(words::wire(root.wire, &[b": the engine's root names an unspoken wire"]));
            }
        }
        Ok(Plans { wires })
    }

    /// Checks a worker's wires in the engine's order and fits the channel's conversions.
    pub fn judge(&mut self, theirs: &[Theirs<'_>]) -> Result<Peer, Box<[u8]>> {
        for (index, wire) in theirs.iter().enumerate() {
            for earlier in theirs.iter().take(index) {
                if earlier.name == wire.name {
                    return Err(words::wire(wire.name, &[b": stated twice"]));
                }
            }
        }
        let mut peer =
            Peer { same: true, frames: None, send_bounds: None, plans: List::uncapped(), work: Workspace::new() };
        for wire in self.wires.iter_mut() {
            let Some(stated) = stated(&wire.name, theirs) else {
                return Err(words::wire(&wire.name, &[b": the worker states none"]));
            };
            if wire.layout == stated.layout {
                for need in &wire.needs {
                    peer.plans
                        .push(Pass {
                            value: need.value,
                            mode: need.mode,
                            wire: wire.name.clone(),
                            source: Box::from(need.declaration),
                            plan: None,
                        })
                        .expect("one pass per root and direction");
                }
                continue;
            }
            if wire.name.as_ref() == PIECES {
                return Err(words::wire(
                    &wire.name,
                    &[
                        b": the worker's layout ",
                        &words::hex(&stated.layout),
                        b" is not the engine's ",
                        &words::hex(&wire.layout),
                        b", and pieces cross as written",
                    ],
                ));
            }
            let derived = wire.prepare(stated, &mut peer.work)?;
            peer.same = false;
            if let Some(frames) = derived.frames {
                peer.frames = Some(frames);
            }
            for pass in derived.passes.into_boxed() {
                peer.plans.push(pass).expect("one pass per root and direction");
            }
        }
        Ok(peer)
    }

    #[cfg(test)]
    pub(crate) fn derivations(&self, name: &str) -> u32 {
        for wire in &self.wires {
            if wire.name.as_ref() == name {
                return wire.derivations;
            }
        }
        unreachable!("test wire is spoken")
    }
}

fn add_need(name: &str, schema: &Schema, needs: &mut List<Need>, need: Need) -> Result<(), Box<[u8]>> {
    let Some(root) = schema.find(need.declaration.as_bytes()) else {
        return Err(words::no_plan(name, need.declaration, need.mode, Why::Root, need.declaration.as_bytes(), None));
    };
    if need.value == Value::Frame {
        match schema.declaration(root).shape {
            Shape::Enum { .. } => {}
            Shape::Record { .. } => {
                return Err(words::no_plan(
                    name,
                    need.declaration,
                    need.mode,
                    Why::Reshaped,
                    need.declaration.as_bytes(),
                    None,
                ));
            }
        }
    }
    needs.push(need).expect("one engine need per root and direction");
    Ok(())
}

fn stated<'a>(name: &str, theirs: &[Theirs<'a>]) -> Option<Theirs<'a>> {
    for stated in theirs {
        if stated.name == name {
            return Some(*stated);
        }
    }
    None
}

impl EngineWire {
    fn prepare(&mut self, stated: Theirs<'_>, work: &mut Workspace) -> Result<Derived, Box<[u8]>> {
        // The key is the text, but every opening must state its fingerprint.
        for cached in &self.known {
            if cached.text.as_ref() == stated.text && cached.layout != stated.layout {
                return Err(self.bad_layout(stated.layout));
            }
        }
        // Rotate the bounded queue once, preserving every other text's age.
        let mut hit = None;
        for _ in 0..self.known.len() {
            let cached = self.known.pop().expect("one entry per turn of the queue");
            if cached.text.as_ref() == stated.text {
                hit = Some(cached);
            } else {
                self.known.push(cached);
            }
        }
        let cached = match hit {
            Some(cached) => cached,
            None => {
                let schema = self.parse(stated)?;
                let derived = self.derive(schema)?;
                if self.known.room() == 0 {
                    let _oldest = self.known.pop();
                }
                Cached { layout: stated.layout, text: bytes::copy_of(stated.text), derived }
            }
        };
        let result = match self.fit(&cached.derived, work) {
            Ok(()) => Ok(cached.derived.clone()),
            Err(error) => Err(error),
        };
        self.known.push(cached);
        result
    }

    fn parse(&self, worker: Theirs<'_>) -> Result<Schema, Box<[u8]>> {
        let schema = if worker.text.len() > WIRE_TEXT {
            return Err(self.bad_text(1));
        } else {
            match schema::parse(worker.text) {
                Ok(schema) => schema,
                Err(error) => return Err(self.bad_text(error.line)),
            }
        };
        let family = schema.spelled(schema.family());
        if family != self.name.as_bytes() {
            return Err(words::wire(&self.name, &[b": the worker's text is of ", family]));
        }
        if schema.fingerprints().family().digest() != worker.layout {
            return Err(self.bad_layout(worker.layout));
        }
        Ok(schema)
    }

    fn bad_layout(&self, layout: [u8; 32]) -> Box<[u8]> {
        words::wire(&self.name, &[b": the worker's text is not its layout ", &words::hex(&layout)])
    }

    fn bad_text(&self, line: u32) -> Box<[u8]> {
        words::wire(
            &self.name,
            &[b": the worker's text does not parse at line ", Decimal::of(u64::from(line)).as_bytes()],
        )
    }

    fn derive(&mut self, worker: Schema) -> Result<Derived, Box<[u8]>> {
        let mut passes = List::with_capacity(self.needs.len());
        for need in &self.needs {
            #[cfg(test)]
            {
                self.derivations = self.derivations.checked_add(1).expect("test derivations fit");
            }
            passes.push(self.between(&worker, *need)?).expect("one pass per need");
        }
        let frames = if self.name.as_ref() == wire::FAMILY {
            let read = Peer::find(&passes, Value::Frame, Mode::Read).expect("jig receives frames");
            Some(table::Frames::new(&self.schema, worker, read.plan.as_ref().expect("worker from another layout"))?)
        } else {
            None
        };
        Ok(Derived { frames, passes })
    }

    fn between(&self, worker: &Schema, need: Need) -> Result<Pass, Box<[u8]>> {
        let engine = self.schema.find(need.declaration.as_bytes()).expect("engine roots are checked at construction");
        let Some(worker_root) = other_root(&self.schema, engine, worker) else {
            return Err(words::no_plan(
                &self.name,
                need.declaration,
                need.mode,
                Why::Root,
                need.declaration.as_bytes(),
                None,
            ));
        };
        let engine = schema::Root { decl: engine, variant: None };
        let worker_root = schema::Root { decl: worker_root, variant: None };
        let (source, old, target, new) = match need.mode {
            Mode::Read => (worker, worker_root, &self.schema, engine),
            Mode::Keep => (&self.schema, engine, worker, worker_root),
        };
        let plan = match Plan::between(need.mode, source, old, &[], target, new, &[]) {
            Ok(plan) => plan,
            Err(error) => {
                let text = if error.old { source } else { target };
                #[expect(clippy::manual_map, reason = "step code uses no closures")]
                let part = if let Some(part) = error.part { Some(text.spelled(part)) } else { None };
                return Err(words::no_plan(
                    &self.name,
                    need.declaration,
                    need.mode,
                    error.why,
                    text.spelled(error.decl),
                    part,
                ));
            }
        };
        Ok(Pass {
            value: need.value,
            mode: need.mode,
            wire: self.name.clone(),
            source: Box::from(table::text(source.spelled(source.declaration(old.decl).name))),
            plan: Some(plan),
        })
    }

    fn fit(&self, converted: &Derived, work: &mut Workspace) -> Result<(), Box<[u8]>> {
        for (need, pass) in self.needs.iter().zip(&converted.passes) {
            if !work.fit(pass.plan.as_ref().expect("worker from another layout")) {
                return Err(words::no_plan(
                    &self.name,
                    need.declaration,
                    need.mode,
                    Why::Deep,
                    pass.source.as_bytes(),
                    None,
                ));
            }
        }
        Ok(())
    }
}

// Locate the other root before asking the planner to confirm the pair.
pub(super) fn other_root(own: &Schema, root: DeclIx, peer: &Schema) -> Option<DeclIx> {
    let name = own.spelled(own.declaration(root).name);
    if let Some(at) = peer.find(name) {
        return Some(at);
    }
    for &was in own.was(Named::Declaration(root)) {
        if let Some(at) = peer.find(own.spelled(was)) {
            return Some(at);
        }
    }
    for index in 0..peer.declarations() {
        let at = DeclIx::new(index);
        for &was in peer.was(Named::Declaration(at)) {
            if peer.spelled(was) == name {
                return Some(at);
            }
        }
    }
    None
}
