use super::*;
use temper_client_domain::forge;
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(crate) const DRAFT_PATH: &str = "docs/design/draft/catch-up.md";
pub(crate) const DRAFT_COMMIT: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
pub(crate) fn draft_root(verb: &str, extra: &[&str]) -> (Harness, u32) {
    let mut root = signed_in(PARTY);
    let words = extended(&["draft", verb, "--project", "shop", DRAFT_PATH, "--key", "draft"], extra);
    let call = root.submit(words.as_slice());
    (root, call)
}
pub(crate) fn landed_draft(ready: bool, chat: Option<u64>, goal: Option<u64>, at: u64) -> w::LandedDraft {
    w::LandedDraft::new(
        &w::CEILINGS,
        w::LandedDraftParts {
            forge: 1,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            path: DRAFT_PATH.as_bytes().into(),
            branch: b"main".as_slice().into(),
            commit: DRAFT_COMMIT.into(),
            title: b"Catch-up barrier".as_slice().into(),
            writer: PARTY,
            at,
            chat,
            goal,
            ready,
        },
    )
    .expect("draft row")
}
pub(crate) fn drafts_panel(rows: &[w::LandedDraft]) -> w::DraftsPanel {
    let mut drafts = List::with_capacity(u32::try_from(rows.len()).expect("few drafts"));
    for row in rows {
        drafts.push(row.clone()).expect("draft row");
    }
    w::DraftsPanel::new(&w::CEILINGS, w::DraftsPanelParts { project: PROJECT, drafts }).expect("drafts panel")
}
impl Harness {
    pub(crate) fn sent_draft(&self, call: u32) -> w::SetDraftParts {
        let (_, _, client::Ask::Family { scope, record }) = self.sends.get(&call).expect("draft sent") else {
            panic!("family request")
        };
        assert_eq!(*scope, c::Scope::Project(PROJECT));
        assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
        let bounds = forge::wire_limits(&self.limits.domain.forge).expect("forge bounds");
        let request = wire::decode_requesting(&record.bytes, &bounds).expect("draft request");
        let w::Request::SetDraft(request) = request.into_parts().request else { panic!("SetDraft") };
        request.into_parts()
    }
    pub(crate) fn deliver_drafts(&mut self, panel: &w::DraftsPanel) {
        let subject = forge::drafts::watch(PROJECT, &self.limits.domain.forge).expect("drafts watch");
        let token = self.watch_token(&subject);
        self.client(client::Event::Up {
            up: client::Up::FamilyDelivered {
                watch: token,
                record: client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::drafts_panel(panel),
                },
            },
        });
    }
}
