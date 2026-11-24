//! Exclusive whole-document transactions (qa/ledger.md, 3).
use crate::{
    Configuration, Ledger, Refusal,
    faces::{ClockFace, FilesFace, LockMode, Mode, WriteMode, WriteOptions},
    refused,
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct Home {
    pub directory: PathBuf,
    pub configuration: Configuration,
    pub ledger: PathBuf,
    pub ledger_lock: PathBuf,
    pub run_lock: PathBuf,
    pub builds: PathBuf,
    pub sessions: PathBuf,
}
impl Home {
    /// The deployment creates the home; opening never creates it.
    pub fn open(directory: &Path, files: &mut impl FilesFace) -> Result<Self, Refusal> {
        let bytes = files.read(&directory.join("qa.json")).map_err(|why| refused(format!("qa.json: {why}")))?;
        let configuration = Configuration::decode(&bytes, directory)?;
        Ok(Self {
            directory: directory.into(),
            ledger: directory.join("ledger.json"),
            ledger_lock: directory.join("ledger.lock"),
            run_lock: directory.join("run.lock"),
            builds: configuration.builds.clone(),
            sessions: directory.join("sessions"),
            configuration,
        })
    }

    #[must_use]
    pub fn session_directory(&self, number: u64) -> PathBuf {
        self.sessions.join(number.to_string())
    }

    #[must_use]
    pub fn triage_tree(&self) -> PathBuf {
        self.directory.join("trees/triage")
    }

    /// Places use their one-based configuration order under trees/.
    #[must_use]
    pub fn pass_tree(&self, place: &str) -> PathBuf {
        let index = self.configuration.places.iter().position(|name| name == place).expect("configured place");
        self.directory.join("trees").join(index.saturating_add(1).to_string())
    }

    /// The lock stays held until the read operation returns.
    pub fn read<T>(
        &self,
        files: &mut impl FilesFace,
        operation: impl FnOnce(&Ledger) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let lock = self.lock(files, LockMode::Wait)?.expect("waiting lock is held on success");
        self.read_locked(files, lock, |ledger, _| operation(ledger))
    }

    /// The holder and its clients wait for the same lock through their clock.
    pub(crate) fn read_polling<T, F: FilesFace>(
        &self,
        files: &mut F,
        clock: &mut impl ClockFace,
        tick: Duration,
        operation: impl FnOnce(&Ledger, &mut F) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        loop {
            if let Some(lock) = self.lock(files, LockMode::Try)? {
                return self.read_locked(files, lock, operation);
            }
            clock.sleep(tick);
        }
    }

    fn read_locked<T, F: FilesFace>(
        &self,
        files: &mut F,
        lock: F::Lock,
        operation: impl FnOnce(&Ledger, &mut F) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let result = operation(&self.load(files)?, files);
        drop(lock);
        result
    }

    /// A refusal discards even local mutations, and never opens the temporary document.
    pub fn transact<T, F: FilesFace>(
        &self,
        files: &mut F,
        operation: impl FnOnce(&mut Ledger, &mut F) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let result = self.transact_loop(files, operation)?;
        // A saved command remains successful even if its best-effort wake fails.
        let _ = crate::events::poke(&self.directory, files);
        Ok(result)
    }

    /// The loop already observes its own saves in the current tick.
    pub(crate) fn transact_loop<T, F: FilesFace>(
        &self,
        files: &mut F,
        operation: impl FnOnce(&mut Ledger, &mut F) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let (_lock, mut ledger) = self.begin(files)?;
        let result = operation(&mut ledger, files)?;
        self.save(files, &ledger)?;
        Ok(result)
    }

    // The loop also needs the files face during its transaction. The owned lock
    // keeps it held while the loop operates on the loaded ledger.
    pub(crate) fn begin<F: FilesFace>(&self, files: &mut F) -> Result<(F::Lock, Ledger), Refusal> {
        let lock = self.lock(files, LockMode::Wait)?.expect("waiting lock is held on success");
        let ledger = self.load(files)?;
        Ok((lock, ledger))
    }

    pub(crate) fn save(&self, files: &mut impl FilesFace, ledger: &Ledger) -> Result<(), Refusal> {
        let bytes = ledger.encode()?;
        let temporary = self.directory.join("ledger.tmp");
        let options = WriteOptions { way: WriteMode::Replace, mode: Mode::Create(0o600), read: false };
        files
            .write(&temporary, &bytes, options, true)
            .and_then(|()| files.rename(&temporary, &self.ledger))
            .map_err(|why| refused(format!("ledger.json: {why}")))
    }
    fn lock<F: FilesFace>(&self, files: &mut F, mode: LockMode) -> Result<Option<F::Lock>, Refusal> {
        files.lock(&self.ledger_lock, mode, false).map_err(|why| refused(format!("ledger.lock: {why}")))
    }
    fn load(&self, files: &mut impl FilesFace) -> Result<Ledger, Refusal> {
        match files.read(&self.ledger) {
            Ok(bytes) => Ledger::decode(&bytes),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(Ledger::default()),
            Err(why) => Err(refused(format!("ledger.json: {why}"))),
        }
    }
}
