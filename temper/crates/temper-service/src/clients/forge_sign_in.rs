//! Kept forges' browser sign-in facts (temper/docs/design/protocol/openid-provider.md, 3 and 7).
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Map, Writer};
use temper_engine_domain_forge::Mode;

/// Why an entry serves no meta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unserved {
    NotKept,
    NotForgejo,
    Origin { origin: Box<[u8]> },
    NoRoom,
    NoSource,
}

#[derive(Clone, Debug)]
struct Entry {
    forge: u16,
    client: Box<[u8]>,
    redirect_uri: Box<[u8]>,
}

/// The entries joined to their clients' redirects, and what each was last named for.
#[derive(Debug)]
pub(crate) struct Entries {
    rows: List<Entry>,
    owned: Option<Entry>,
    box_owned: bool,
    source: Option<u16>,
    public: Option<temper_forge_setup::setup::Origin>,
    limit: usize,
    named: Map<u16, Unserved>,
    waiting: bool,
}
impl Entries {
    pub(crate) fn new(
        rows: &[crate::startup::application::sign_in::ForgeSignIn],
        clients: &[jig_protocol_config::clients::ProviderClient],
        public: Option<&temper_forge_setup::setup::Origin>,
        owned: Option<&temper_forge_setup::machine::OwnedMachine>,
        limit: u32,
    ) -> Self {
        let capacity = u32::try_from(rows.len()).expect("document bounds entries");
        let mut entries = List::with_capacity(capacity);
        for row in rows {
            let mut redirect_uri = None;
            for client in clients {
                if row.client == client.id {
                    redirect_uri = Some(client.redirect_uri.clone());
                    break;
                }
            }
            entries
                .push(Entry {
                    forge: row.forge,
                    client: row.client.clone(),
                    redirect_uri: redirect_uri.expect("checked sign-in client"),
                })
                .expect("one joined entry per row");
        }
        let mut owned_entry = None;
        if let Some(owned) = owned
            && let Some(id) = &owned.sign_in
        {
            for client in clients {
                if client.id == *id {
                    owned_entry =
                        Some(Entry { forge: 0, client: client.id.clone(), redirect_uri: client.redirect_uri.clone() });
                }
            }
        }
        Self {
            rows: entries,
            owned: owned_entry,
            box_owned: owned.is_some(),
            source: None,
            public: public.cloned(),
            limit: usize::try_from(limit).expect("bundle limit fits usize"),
            named: Map::with_capacity(capacity),
            waiting: false,
        }
    }
    pub(crate) fn source(&mut self, forge: u16, added: bool) {
        if added {
            self.source = Some(forge);
        } else if self.source == Some(forge) {
            self.source = None;
        }
    }
    fn rows(&self, forges: &crate::forges::Forges) -> List<Entry> {
        let capacity = self.rows.len().checked_add(forges.len()).expect("bounded forge entries");
        let mut rows = List::with_capacity(capacity);
        for row in &self.rows {
            rows.push(row.clone()).expect("configured entries counted");
        }
        if let Some(owned) = &self.owned {
            for forge in forges.iter() {
                if forge.mode == Mode::Owned {
                    let mut row = owned.clone();
                    row.forge = forge.number;
                    let mut present = false;
                    for entry in &rows {
                        if entry.forge == row.forge {
                            present = true;
                            break;
                        }
                    }
                    if !present {
                        rows.push(row).expect("kept forges counted");
                    }
                }
            }
        }
        rows
    }
    pub(crate) fn waiting(&self) -> bool {
        self.waiting
    }
}

impl super::Clients {
    pub(crate) fn configure_forge_sign_in(&mut self, settings: &crate::settings::Engine) {
        let rows = match &settings.sign_in {
            Some(sign_in) => sign_in.forges.as_ref(),
            None => &[],
        };
        let clients = match &settings.jig.clients.provider {
            Some(provider) => provider.clients.as_ref(),
            None => &[],
        };
        let public = match &settings.machine.owned {
            Some(owned) => owned.public.as_ref(),
            None => None,
        };
        let owned = match &settings.machine.owned {
            Some(owned) if self.deployment.owned || owned.sign_in.is_some() => Some(owned),
            Some(_) | None => None,
        };
        self.forge_sign_in = Entries::new(rows, clients, public, owned, settings.limits.clients.bundle_bytes);
    }
}

fn attribute(byte: &u8) -> &[u8] {
    match byte {
        b'&' => b"&amp;",
        b'"' => b"&quot;",
        b'\'' => b"&#39;",
        b'<' => b"&lt;",
        b'>' => b"&gt;",
        _ => core::slice::from_ref(byte),
    }
}

/// The attribute-escaped bytes, shared by measurement and writing.
fn meta(entry: &Entry, origin: &[u8], mode: &[u8]) -> Box<[u8]> {
    let number = Decimal::of(u64::from(entry.forge));
    let parts = [b"<meta name=\"temper-forge-sign-in\" content=\"".as_slice(), number.as_bytes(), b" "];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded meta");
    }
    for value in [entry.client.as_ref(), origin] {
        for byte in value {
            length = length.checked_add(attribute(byte).len()).expect("bounded meta");
        }
        length = length.checked_add(1).expect("meta separator");
    }
    length = length.checked_add(mode.len()).expect("meta mode").checked_add(2).expect("meta close");
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("measured meta");
    }
    for value in [entry.client.as_ref(), origin] {
        for byte in value {
            out.put(attribute(byte)).expect("measured attribute");
        }
        out.put(b" ").expect("measured separator");
    }
    out.put(mode).expect("measured mode");
    out.put(b"\">").expect("measured close");
    out.finish()
}

fn serving(entries: &Entries, row: &Entry, forges: &crate::forges::Forges) -> Result<Box<[u8]>, Unserved> {
    let forge = forges.get(row.forge).ok_or(Unserved::NotKept)?;
    match forge.kind {
        crate::settings::Kind::Forgejo => {}
        crate::settings::Kind::GitHub => return Err(Unserved::NotForgejo),
    }
    let (public, mode) = match forge.mode {
        Mode::Owned => (entries.public.as_ref(), b"owned".as_slice()),
        Mode::Managed | Mode::Connected | Mode::Fake => (None, b"other".as_slice()),
    };
    let origin = crate::startup::application::forge_origin(&forge.git, forge.web.as_deref(), public);
    if row.redirect_uri.strip_prefix(origin.as_ref()) != Some(b"/user/oauth2/temper/callback".as_slice()) {
        return Err(Unserved::Origin { origin });
    }
    if entries.box_owned && forge.mode == Mode::Owned && entries.source != Some(row.forge) {
        return Err(Unserved::NoSource);
    }
    Ok(meta(row, &origin, mode))
}

/// Each entry's verdict and the metas' bytes, in the document's order.
#[expect(clippy::type_complexity, reason = "the design returns metas and ordered entry verdicts together")]
fn decide(
    entries: &Entries,
    forges: &crate::forges::Forges,
    page: usize,
    files: usize,
) -> (Box<[u8]>, List<(u16, Box<[u8]>, Unserved)>) {
    let rows = entries.rows(forges);
    let capacity = rows.len().checked_add(forges.len()).expect("bounded facts");
    let mut metas = List::with_capacity(capacity);
    let mut unserved = List::with_capacity(rows.len());
    let mut length = 0_usize;
    for row in &rows {
        let answer = match serving(entries, row, forges) {
            Ok(meta) => match total(page, files, length, meta.len()) {
                Some(total) if total <= entries.limit => Ok(meta),
                Some(_) | None => Err(Unserved::NoRoom),
            },
            Err(why) => Err(why),
        };
        match answer {
            Ok(meta) => {
                length = length.checked_add(meta.len()).expect("metas within bundle limit");
                metas.push(meta).expect("at most one meta per entry");
            }
            Err(why) => unserved.push((row.forge, row.client.clone(), why)).expect("one verdict per entry"),
        }
    }
    if entries.box_owned {
        for forge in forges.iter() {
            if forge.mode == Mode::Owned && (entries.owned.is_none() || entries.source != Some(forge.number)) {
                let fact = off(forge.number);
                if let Some(total) = total(page, files, length, fact.len())
                    && total <= entries.limit
                {
                    length = length.checked_add(fact.len()).expect("bounded fact");
                    metas.push(fact).expect("kept forges counted");
                }
            }
        }
    }
    let mut out = Writer::new(length);
    for meta in metas.into_boxed() {
        out.put(&meta).expect("measured metas");
    }
    (out.finish(), unserved)
}

fn off(forge: u16) -> Box<[u8]> {
    let number = Decimal::of(u64::from(forge));
    let parts = [b"<meta name=\"temper-forge-sign-in-off\" content=\"".as_slice(), number.as_bytes(), b"\">"];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded off fact");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("measured off fact");
    }
    out.finish()
}
fn total(page: usize, files: usize, metas: usize, meta: usize) -> Option<usize> {
    page.checked_add(files)?.checked_add(metas)?.checked_add(meta)
}

/// Decide again, set the page's metas, and name each entry newly not serving.
pub(crate) fn refresh(engine: &mut crate::engine::Engine) {
    if !engine.forge_records.done()
        || (engine.clients.forge_sign_in.rows.is_empty() && !engine.clients.forge_sign_in.box_owned)
    {
        return;
    }
    let (page, files) = match &engine.clients.bundle {
        Some(bundle) => {
            let mut files = 0_usize;
            for (_, file) in &bundle.files {
                files = files.checked_add(file.bytes.len()).expect("bounded bundle files");
            }
            (bundle.page.len().checked_sub(bundle.forge_meta_bytes).expect("page includes its metas"), files)
        }
        None => (0, 0),
    };
    let (metas, unserved) = decide(&engine.clients.forge_sign_in, &engine.workers.forges, page, files);
    let entries = &mut engine.clients.forge_sign_in;
    let mut named =
        Map::with_capacity(entries.rows.len().checked_add(engine.workers.forges.len()).expect("bounded entries"));
    entries.waiting = false;
    for (number, client, why) in unserved.into_boxed() {
        if entries.named.get(&number) != Some(&why)
            && engine.notes.try_push(crate::Note::ForgeSignIn { number, client, why: why.clone() }).is_err()
        {
            entries.waiting = true;
            if let Some(previous) = entries.named.get(&number) {
                named.insert(number, previous.clone()).expect("one verdict per entry");
            }
            continue;
        }
        named.insert(number, why).expect("one verdict per entry");
    }
    entries.named = named;
    engine.clients.pending_forge_metas = Some(metas);
    engine.clients.publish_forge_metas();
}
