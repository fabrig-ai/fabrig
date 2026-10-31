//! Durable store IO routing owned by the local host (protocol/hosts.md, section 5.3).
//! File operations are submitted serially and each domain terminal precedes later file work.
use super::{HostIo, store_limits};
use alloc::boxed::Box;
use skein_io::kernel;
use skein_lib::{Queue, Time, Token};
use smith_local_domain as local;
use smith_local_protocol as local_protocol;

#[derive(Debug)]
pub(super) struct StoreIo {
    pub(super) store: Option<local_protocol::Store>,
    pub(super) cleanup_warning: Option<kernel::Error>,
    pub(super) store_requests: Queue<local_protocol::StoreRequest>,
    pub(super) store_actions: Queue<local_protocol::Action>,
}

impl StoreIo {
    pub(super) fn new() -> Self {
        Self { store: None, cleanup_warning: None, store_requests: Queue::uncapped(), store_actions: Queue::uncapped() }
    }
}

impl HostIo {
    pub(super) fn store_actions_dispatched(&self) -> bool {
        self.store_io.store_actions.is_empty()
    }

    pub(crate) fn adopt_store(&mut self, root: kernel::Fd) -> Option<()> {
        let token = self.files.adopt_root(root)?;
        self.store_io.store = Some(local_protocol::Store::new(
            Token::new(u64::MAX - 1),
            token,
            root,
            store_limits(),
            self.endpoints.clone(),
        )?);
        Some(())
    }

    pub(crate) fn ephemeral(&mut self, parent: kernel::Fd, name: Box<[u8]>) -> Result<(), kernel::Fd> {
        let token = self.files.adopt_root(parent).ok_or(parent)?;
        self.store_io.store.as_mut().expect("adopted private store").ephemeral(token, parent, name);
        Ok(())
    }

    pub(crate) fn cleanup_warning(&mut self) -> Option<kernel::Error> {
        self.store_io.cleanup_warning.take()
    }

    #[cfg(test)]
    pub(crate) fn store_requests(&mut self) -> &mut Queue<local_protocol::StoreRequest> {
        &mut self.store_io.store_requests
    }

    pub(crate) fn store_request(&mut self, request: local_protocol::StoreRequest) {
        self.store_io.store_requests.push(request);
    }

    pub(super) fn store_pass(&mut self, now: Time, local_events: &mut Queue<local::Event>) {
        for _ in 0_u8..3_u8 {
            let Some(action) = self.store_io.store_actions.pop() else { break };
            match action {
                local_protocol::Action::Io { request, deadline } => self.file_requests.push((request, deadline)),
                local_protocol::Action::Domain(event) => local_events.push(event),
                local_protocol::Action::Fault(_) => {}
                local_protocol::Action::CleanupWarning { error } => self.store_io.cleanup_warning = Some(error),
                local_protocol::Action::Closed => self.store_io.store = None,
            }
        }
        if self.store_io.store_actions.is_empty()
            && let Some(store) = &mut self.store_io.store
        {
            if self.closing.is_some() {
                store.close(now, &mut self.store_io.store_actions);
            } else if store.takes()
                && let Some(request) = self.store_io.store_requests.pop()
            {
                store.request(request, now, &mut self.store_io.store_actions);
            }
        }
    }
}
