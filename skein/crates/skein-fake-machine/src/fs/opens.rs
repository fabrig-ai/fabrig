//! Existing and created file opens beneath a root.
use super::{Body, How, Machine, Opened, READ, Refusal, UMASK, WRITE, fail};
use alloc::vec::Vec;
impl Machine {
    pub(crate) fn open_sibling_file(&mut self, directory: Opened, name: &[u8]) -> Result<Opened, Refusal> {
        let root = self.handle(directory).node;
        let source = self.resolve_path(root, name, false, true, true)?.node.ok_or(Refusal::NotFound)?;
        let target = match &self.node_state(source).body {
            Body::Link(target) => {
                if target.len() > skein_io::kernel::LONGEST_NAME || !skein_io::kernel::is_name(target) {
                    return Err(Refusal::Loop);
                }
                let sibling = self
                    .resolve_path(root, target, false, true, true)
                    .map_err(|_| Refusal::Loop)?
                    .node
                    .ok_or(Refusal::Loop)?;
                match self.node_state(sibling).body {
                    Body::File(_) => {}
                    Body::Link(_) | Body::Directory { .. } | Body::Special(_) => return Err(Refusal::Loop),
                }
                Some(target.clone())
            }
            Body::File(_) | Body::Directory { .. } | Body::Special(_) => None,
        };
        self.open(directory, target.as_deref().unwrap_or(name), How::PathNoFollow)
    }

    /// Opens `path`, resolved beneath the directory `root` has open, as
    /// `how` says.
    pub fn open(&mut self, root: Opened, path: &[u8], how: How) -> Result<Opened, Refusal> {
        let root = self.handle(root).node;
        let node = match how {
            How::PathNoFollow
            | How::Read
            | How::ReadNoFollow
            | How::Write
            | How::WriteNoFollow
            | How::Directory
            | How::DirectoryNoFollow => {
                let path_only = matches!(how, How::PathNoFollow);
                let no_follow =
                    path_only || matches!(how, How::ReadNoFollow | How::WriteNoFollow | How::DirectoryNoFollow);
                let found = self.resolve_path(root, path, false, no_follow, path_only)?;
                let node = found.node.ok_or(Refusal::NotFound)?;
                if (found.slash || matches!(how, How::Directory | How::DirectoryNoFollow)) && !self.is_directory(node) {
                    return Err(Refusal::NotADirectory);
                }
                if !path_only && !self.may(node, READ) {
                    return Err(Refusal::Permission);
                }
                if matches!(how, How::Write | How::WriteNoFollow) {
                    if self.is_directory(node) {
                        return Err(Refusal::IsADirectory);
                    }
                    if !self.may(node, WRITE) {
                        return Err(Refusal::Permission);
                    }
                }
                // Opened without blocking, then refused for what it is.
                if !path_only && let Body::Special(_) = self.node_state(node).body {
                    return Err(Refusal::NotAFile);
                }
                node
            }
            How::Append { mode } => {
                let found = self.resolve(root, path, false, false)?;
                if found.slash {
                    return Err(Refusal::IsADirectory);
                }
                match found.node {
                    Some(node) => {
                        match &self.node_state(node).body {
                            Body::Directory { .. } => return Err(Refusal::IsADirectory),
                            Body::Special(_) => return Err(Refusal::NotAFile),
                            Body::File(_) => {}
                            Body::Link(_) => fail("resolution follows links"),
                        }
                        if !self.may(node, WRITE) {
                            return Err(Refusal::Permission);
                        }
                        node
                    }
                    None => {
                        let name = found.name.ok_or(Refusal::NotFound)?;
                        if !self.may(found.parent, WRITE) {
                            return Err(Refusal::Permission);
                        }
                        self.make(found.parent, name, Body::File(Vec::new().into()), mode & !UMASK)
                    }
                }
            }
            How::Create { mode } | How::CreateNoFollow { mode } => {
                let found = self.resolve(root, path, true, matches!(how, How::CreateNoFollow { .. }))?;
                // A path that ends at a directory of no name of its own
                // (`.`, `a/..`) is taken: O_EXCL answers before O_CREAT.
                let Some(name) = found.name else {
                    return Err(Refusal::Exists);
                };
                if found.node.is_some() {
                    return Err(Refusal::Exists);
                }
                if !self.may(found.parent, WRITE) {
                    return Err(Refusal::Permission);
                }
                self.make(found.parent, name, Body::File(Vec::new().into()), mode & !UMASK)
            }
        };
        Ok(self.issue(node, how))
    }
}
