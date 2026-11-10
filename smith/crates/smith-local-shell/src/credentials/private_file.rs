//! Private credential directories and startup store answers (shell/credentials.md, 2).
//! The shell makes and checks the directory; skein owns the durable write and closes.
use skein_io::file;
use skein_lib::Duration;
use skein_shell::Clock;
use std::{os::unix::fs::DirBuilderExt, path::Path};

pub(super) fn store(path: &Path, bytes: Box<[u8]>, expected: file::Expect, entropy: &[u8; 32]) -> Result<bool, String> {
    let clock = Clock::new();
    let parent = path.parent().ok_or("private file needs a parent")?;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)
        .map_err(|_| "cannot make private directory")?;
    smith_agent_shell::program::private_directory(parent).map_err(|_| "unreadable private directory")?;
    let grandparent = parent.parent().ok_or("private directory needs a parent")?;
    let file_name = path.file_name().ok_or("private file needs a name")?.as_encoded_bytes();
    let directory_name = parent.file_name().ok_or("private directory needs a name")?.as_encoded_bytes();
    let root = skein_shell::open_root(grandparent).map_err(|_| "cannot open private directory")?;
    let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: skein_shell::STORE_OPERATIONS })
        .map_err(|_| "private file kernel unavailable")?;
    let store = skein_shell::StartupStore {
        root,
        directory: skein_shell::StoreDirectory::Private { name: directory_name, user: skein_shell::effective_user() },
        name: file_name,
        bytes,
        expected,
        timeout: Duration::from_secs(30),
        seed: u64::from_be_bytes(entropy.get(..8).expect("entropy prefix").try_into().expect("eight bytes")),
    };
    match skein_shell::store_file(&mut kernel, &clock, store) {
        Ok(skein_shell::Stored::Kept) => Ok(true),
        Ok(skein_shell::Stored::Conflict) => Ok(false),
        Err(skein_shell::Unstored::TooLarge) => Err("private file exceeds bound".into()),
        Err(skein_shell::Unstored::Unsafe(_) | skein_shell::Unstored::Kernel(_)) => {
            Err("private file operation failed".into())
        }
    }
}
