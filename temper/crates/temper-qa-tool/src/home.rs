//! Exclusive whole-document transactions (qa/ledger.md, 3).
use crate::{Configuration, Ledger, Refusal, refused};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
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
    pub fn open(directory: &Path) -> Result<Self, Refusal> {
        let bytes = fs::read(directory.join("qa.json")).map_err(|why| refused(format!("qa.json: {why}")))?;
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

    /// The lock stays held until the read operation returns.
    pub fn read<T>(&self, operation: impl FnOnce(&Ledger) -> Result<T, Refusal>) -> Result<T, Refusal> {
        let lock = self.lock()?;
        let result = operation(&self.load()?);
        drop(lock);
        result
    }

    /// A refusal discards even local mutations, and never opens the temporary document.
    pub fn transact<T>(&self, operation: impl FnOnce(&mut Ledger) -> Result<T, Refusal>) -> Result<T, Refusal> {
        let lock = self.lock()?;
        let mut ledger = self.load()?;
        let result = operation(&mut ledger)?;
        let bytes = ledger.encode()?;
        let temporary = self.directory.join("ledger.tmp");
        let write = || -> std::io::Result<()> {
            let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &self.ledger)?;
            Ok(())
        };
        write().map_err(|why| refused(format!("ledger.json: {why}")))?;
        drop(lock);
        Ok(result)
    }
    fn lock(&self) -> Result<File, Refusal> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(&self.ledger_lock)
            .map_err(|why| refused(format!("ledger.lock: {why}")))?;
        file.lock().map_err(|why| refused(format!("ledger.lock: {why}")))?;
        Ok(file)
    }
    fn load(&self) -> Result<Ledger, Refusal> {
        match fs::read(&self.ledger) {
            Ok(bytes) => Ledger::decode(&bytes),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(Ledger::default()),
            Err(why) => Err(refused(format!("ledger.json: {why}"))),
        }
    }
}
