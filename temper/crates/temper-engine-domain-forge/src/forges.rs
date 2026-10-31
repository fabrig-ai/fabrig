//! Forge records and the machine's places (temper/docs/design/draft/playground-reset.md, 3–4).
use crate::{
    Bot, Configured, DemoState, Domain, FirstIntent, ForgeRecord, ForgeState, Key, Kind, Machine, Mode, Owned, Place,
    Request, Stored,
};
use alloc::boxed::Box;
use skein_lib::Queue;

impl Domain {
    /// Keep the machine's places for new forges.
    pub fn set_machine(&mut self, machine: Machine) {
        self.machine = Some(machine);
    }

    /// The kept record, whatever its state.
    #[must_use]
    pub fn record(&self, forge: u16) -> Option<&ForgeRecord> {
        self.records.get(&forge)
    }

    /// Ended at removal's decision, or absent; the demo's end also ends its forge (domain/forge.md, 4.1).
    #[must_use]
    pub fn forge_ended(&self, forge: u16) -> bool {
        if self.demo_state == Some(DemoState::Ended)
            && match &self.demo {
                Some(demo) => demo.adoption.forge == forge,
                None => false,
            }
        {
            return true;
        }
        match self.record(forge) {
            Some(record) => match record.state {
                ForgeState::Kept => false,
                ForgeState::Ended { .. } => true,
            },
            None => !self.configured.contains_key(&forge),
        }
    }

    /// Whether a real forge of any state is kept (section 6.3).
    #[must_use]
    pub fn real_forge_kept(&self) -> bool {
        for (_, record) in &self.records {
            if record.mode != Mode::Fake {
                return true;
            }
        }
        false
    }
}

/// Make a record numbered next; refuse at the kept-forges bound or when numbers are spent.
#[expect(clippy::too_many_arguments, reason = "the record fields and its decision output travel together")]
pub(crate) fn make(
    d: &mut Domain,
    kind: Kind,
    mode: Mode,
    place: Place,
    bot: Bot,
    asked: bool,
    first: Option<FirstIntent>,
    out: &mut Queue<Request>,
) -> Option<u16> {
    if d.records.len() >= d.records.capacity() || d.configured.len() >= d.configured.capacity() {
        return None;
    }
    let number = d.forge_number.checked_add(1)?;
    d.forge_number = number;
    out.push(Request::Save { record: Stored::ForgeNumber { last: number }, release: None, read_afresh: None });
    let record =
        ForgeRecord { number, kind, mode, state: ForgeState::Kept, place, read_only: false, bot, asked, first };
    save(d, record.clone(), out);
    out.push(Request::ForgeKept(record));
    Some(number)
}

/// Save a changed record and keep its public configuration in step.
pub(crate) fn save(d: &mut Domain, record: ForgeRecord, out: &mut Queue<Request>) {
    d.configured.insert(record.number, configured_of(&record)).expect("admitted configured forge");
    d.records.insert(record.number, record.clone()).expect("admitted forge record");
    out.push(Request::Save { record: Stored::Forge(record), release: None, read_afresh: None });
}

/// Erase a record whole; its number stays spent (section 4).
#[expect(dead_code, reason = "forge removal consumes this shared surface")]
pub(crate) fn erase(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    d.records.remove(&forge);
    d.configured.remove(&forge);
    out.push(Request::Erase { key: Key::Forge(forge) });
}

pub(crate) fn configured_of(record: &ForgeRecord) -> Configured {
    Configured {
        number: record.number,
        kind: record.kind,
        mode: record.mode,
        bot: record.bot.login.clone(),
        owned: match record.mode {
            Mode::Owned => Some(Owned { forgejo: Box::new([]), runner: None, password: Box::new([]) }),
            Mode::Connected | Mode::Managed | Mode::Fake => None,
        },
        git: record.place.git.clone(),
        web: record.place.web.clone(),
        asked: record.asked,
        admin_connect: record.mode == Mode::Managed,
        read_only: record.read_only,
    }
}
