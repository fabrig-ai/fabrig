//! Forge records and the machine's places (temper/docs/design/draft/playground-reset.md, 3–4).
use crate::{
    Bot, Configured, Domain, FirstIntent, ForgeRecord, ForgeState, Key, Kind, Machine, Mode, Owned, Place, Request,
    Stored,
};
use alloc::boxed::Box;
use skein_lib::{Map, Queue};

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

    /// Ended at removal's decision, or absent (temper/docs/design/draft/playground-reset.md, 4).
    #[must_use]
    pub fn forge_ended(&self, forge: u16) -> bool {
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
#[expect(clippy::too_many_arguments, reason = "shared forge-making surface carries the record fields")]
pub(crate) fn make(
    d: &mut Domain,
    kind: Kind,
    mode: Mode,
    place: Place,
    git_program: Option<Box<[u8]>>,
    bot: Bot,
    asked: bool,
    first: Option<FirstIntent>,
    out: &mut Queue<Request>,
) -> Option<u16> {
    if d.records.len() >= d.records.capacity() {
        return None;
    }
    let number = d.forge_number.checked_add(1)?;
    d.forge_number = number;
    out.push(Request::Save { record: Stored::ForgeNumber { last: number }, release: None, read_afresh: None });
    let record = ForgeRecord {
        number,
        kind,
        mode,
        state: ForgeState::Kept,
        place,
        git_program,
        read_only: false,
        bot,
        asked,
        first,
    };
    save(d, record.clone(), out);
    out.push(Request::ForgeKept(record));
    Some(number)
}

/// Save a changed record and keep its public configuration in step.
pub(crate) fn save(d: &mut Domain, record: ForgeRecord, out: &mut Queue<Request>) {
    d.identities.insert(record.number, u32::from(record.number)).expect("admitted forge credential");
    d.configured.insert(record.number, configured_of(&record)).expect("admitted configured forge");
    d.records.insert(record.number, record.clone()).expect("admitted forge record");
    out.push(Request::Save { record: Stored::Forge(record), release: None, read_afresh: None });
}

/// Erase a record whole; its number stays spent (section 4).
pub(crate) fn erase(d: &mut Domain, forge: u16, out: &mut Queue<Request>) {
    d.identities.remove(&forge);
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

/// Admit document forges until startup supplies only records.
pub(crate) fn document(
    limit: u32,
    forges: &[Configured],
) -> Result<(Map<u16, Configured>, u16), skein_forge::api::Error> {
    if forges.len() > usize::try_from(limit).expect("forge count fits usize") {
        return Err(skein_forge::api::Error::TooLarge);
    }
    // Document forges and records each enter at the kept-forges bound.
    let mut configured = Map::uncapped();
    let mut forge_number = 0;
    for forge in forges {
        forge_number = forge_number.max(forge.number);
        if forge.git.len() > usize::try_from(crate::PULL_ADDRESS_BYTES).expect("u32 fits usize") {
            return Err(skein_forge::api::Error::TooLarge);
        }
        if configured.insert(forge.number, forge.clone()).is_err() {
            return Err(skein_forge::api::Error::TooLarge);
        }
    }
    Ok((configured, forge_number))
}
