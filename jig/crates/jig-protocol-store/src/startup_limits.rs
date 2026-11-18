//! Runtime codecs installed after the setting reads and before restore
//! (protocol/store/README.md, 8; an application's limit settings).
use crate::{Get, Phase, Request, Row, Store, sizes};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{self as core, settings};
use skein_lib::Token;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InstallRefusal {
    NotOpening,
    Store(sizes::SizeRefusal),
}

impl Store {
    /// Read only the two limit settings before installing runtime codecs. The
    /// application supplies its own setting's key, relative to its tag. These
    /// reads do not begin restore, and never inspect the secrets store.
    pub fn limit_settings(&mut self, application_key: Box<[u8]>) -> Result<(Option<Row>, Option<Row>), InstallRefusal> {
        self.before_restore()?;
        if let Some(key) = &self.startup_setting
            && *key != application_key
        {
            self.runtime_started = true;
            return Err(InstallRefusal::NotOpening);
        }
        self.startup_setting = Some(application_key.clone());
        let key = core::Key::Core(core::CoreKey::Settings(settings::Key::Changed(settings::Setting::Limits)));
        let jig = crate::gets::row(self, Token::new(0), Get::Jig(key)).or(Err(InstallRefusal::NotOpening))?;
        let application = crate::gets::row(self, Token::new(0), Get::Application(application_key))
            .or(Err(InstallRefusal::NotOpening))?;
        Ok((jig, application))
    }

    /// Replace runtime core/codec ceilings without moving any KV or driver
    /// envelope. No restore, non-setting read, switch or commit may precede it.
    pub fn install_limits(
        &mut self,
        core: &core::Limits,
        application: sizes::Application,
        transcript_bytes: u32,
    ) -> Result<(), InstallRefusal> {
        self.before_restore()?;
        let limits = match runtime_limits(&self.limits, core, application, transcript_bytes) {
            Ok(limits) => limits,
            Err(why) => return Err(InstallRefusal::Store(why)),
        };
        let codec = runtime_codec(core, transcript_bytes).expect("checked runtime codecs");
        self.transcripts.reads = skein_lib::Map::with_capacity(limits.reads);
        self.limits = limits;
        self.codec_limits = codec;
        self.application = application;
        Ok(())
    }

    fn before_restore(&self) -> Result<(), InstallRefusal> {
        match self.phase {
            Phase::Ready { .. } if !self.runtime_started => Ok(()),
            Phase::Ready { .. } | Phase::Closed | Phase::Stopped => Err(InstallRefusal::NotOpening),
        }
    }
}

/// The pre-restore codec check, also used by an application's pure resolver.
/// Its result changes only runtime core/read ceilings, never a KV envelope.
pub fn runtime_limits(
    stock: &crate::Limits,
    core: &core::Limits,
    application: sizes::Application,
    transcript_bytes: u32,
) -> Result<crate::Limits, sizes::SizeRefusal> {
    let codec = runtime_codec(core, transcript_bytes)?;
    sizes::check_commits(core, &application, &stock.commits, &stock.store, stock.orphan_erases)?;
    let frame = u32::try_from(crate::frame_bound(&codec).ok_or(sizes::SizeRefusal::Ceiling)?)
        .or(Err(sizes::SizeRefusal::Ceiling))?;
    if frame > stock.store.page.bytes {
        return Err(sizes::SizeRefusal::Page { needs: frame, has: stock.store.page.bytes });
    }
    let mut limits = *stock;
    limits.core = *core;
    limits.reads = core.load_slots.checked_add(core.tasks.tasks).ok_or(sizes::SizeRefusal::Ceiling)?;
    if !crate::valid_with_codec(&limits, &codec) {
        return Err(sizes::SizeRefusal::Ceiling);
    }
    Ok(limits)
}

fn runtime_codec(core: &core::Limits, transcript_bytes: u32) -> Result<crate::codec::View, sizes::SizeRefusal> {
    let mut codec = crate::codec::limits(core).ok_or(sizes::SizeRefusal::Ceiling)?;
    if transcript_bytes > codec.core.turn_record_transcript {
        return Err(sizes::SizeRefusal::Ceiling);
    }
    codec.core.turn_record_transcript = transcript_bytes;
    Ok(codec)
}

pub(crate) fn observe(store: &mut Store, request: &Request) {
    if store.runtime_started || store.phase == Phase::Closed {
        return;
    }
    let runtime = match request {
        Request::Open { .. }
        | Request::Rehearse { .. }
        | Request::Back { .. }
        | Request::Site { .. }
        | Request::Job { .. }
        | Request::Close { .. }
        | Request::Get {
            key: Get::Jig(core::Key::Core(core::CoreKey::Settings(settings::Key::Changed(settings::Setting::Limits)))),
            ..
        } => false,
        Request::Get { .. }
        | Request::Restore { .. }
        | Request::Load { .. }
        | Request::Read { .. }
        | Request::Transcript { .. }
        | Request::Secrets { .. }
        | Request::Switch { .. }
        | Request::Commit { .. } => true,
    };
    store.runtime_started |= runtime;
}
