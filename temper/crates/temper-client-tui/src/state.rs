//! The TUI's own state beside the client root (temper/docs/design/client/README.md, 6.14 and 7.1).
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the TUI holds independent sign-in, paste and credential facts")]
pub struct Tui {
    address: Box<[u8]>,
    field: Box<[u8]>,
    signing: Signing,
    browser: Option<Browser>,
    pasted: bool,
    signed_out: bool,
    unkept: bool,
    linked: bool,
    openings: u64,
    revision: u64,
    written: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Browser {
    Opening,
    Opened,
    None,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Signing {
    Idle,
    Refused(Refusal),
    Asked,
    Answered { lost: bool },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Refusal {
    NotALink,
    OtherAddress { address: Box<[u8]> },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Told {
    Typed { value: Box<[u8]> },
    Refused { refusal: Refusal },
    Asked,
    Browse,
    Opened { opened: bool },
    Answered { lost: bool },
    Link { link: jig_client::Link },
    Kept { written: bool },
}
impl Tui {
    #[must_use]
    pub fn new(address: &[u8]) -> Tui {
        Tui {
            address: copy_of(address),
            field: copy_of(b""),
            signing: Signing::Idle,
            browser: None,
            pasted: false,
            signed_out: false,
            unkept: false,
            linked: false,
            openings: 1,
            revision: 0,
            written: 0,
        }
    }
    #[must_use]
    pub fn address(&self) -> &[u8] {
        &self.address
    }
    #[must_use]
    pub fn field(&self) -> &[u8] {
        &self.field
    }
    #[must_use]
    pub const fn signing(&self) -> &Signing {
        &self.signing
    }
    #[must_use]
    pub const fn browser(&self) -> Option<Browser> {
        self.browser
    }
    #[must_use]
    pub const fn pasted(&self) -> bool {
        self.pasted
    }
    #[must_use]
    pub const fn signed_out(&self) -> bool {
        self.signed_out
    }
    #[must_use]
    pub const fn unkept(&self) -> bool {
        self.unkept
    }
    #[must_use]
    pub const fn openings(&self) -> u64 {
        self.openings
    }
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    pub(crate) const fn written(&self) -> u64 {
        self.written
    }
    fn empty(&mut self) {
        self.field = copy_of(b"");
        self.written = self.written.saturating_add(1);
    }
    pub fn step(&mut self, told: Told) {
        let changed = match told {
            Told::Typed { value } => {
                let signing = match self.signing {
                    Signing::Refused(..) | Signing::Answered { .. } | Signing::Idle => Signing::Idle,
                    Signing::Asked => Signing::Asked,
                };
                let changed = self.field != value || self.signing != signing;
                self.field = value;
                self.signing = signing;
                changed
            }
            Told::Refused { refusal } => {
                let signing = Signing::Refused(refusal);
                let changed = self.signing != signing;
                self.signing = signing;
                changed
            }
            Told::Asked => {
                let changed = self.signing != Signing::Asked || !self.field.is_empty() || !self.pasted;
                if changed {
                    self.empty();
                }
                self.signing = Signing::Asked;
                self.pasted = true;
                changed
            }
            Told::Browse => {
                let changed = self.browser != Some(Browser::Opening) || self.pasted;
                self.browser = Some(Browser::Opening);
                self.pasted = false;
                changed
            }
            Told::Opened { opened } => {
                let browser = Some(if opened { Browser::Opened } else { Browser::None });
                let changed = self.browser != browser;
                self.browser = browser;
                changed
            }
            Told::Answered { lost } => match self.signing {
                Signing::Asked => {
                    self.signing = Signing::Answered { lost };
                    true
                }
                Signing::Idle | Signing::Refused(..) | Signing::Answered { .. } => false,
            },
            Told::Link { link } => match link {
                jig_client::Link::SignedOut => {
                    if self.linked {
                        self.signing = Signing::Idle;
                        self.browser = None;
                        self.pasted = false;
                        self.empty();
                        self.linked = false;
                        self.signed_out = true;
                        self.openings = self.openings.saturating_add(1);
                        true
                    } else {
                        false
                    }
                }
                jig_client::Link::Starting
                | jig_client::Link::Live
                | jig_client::Link::Offline
                | jig_client::Link::OutOfDate => {
                    let changed =
                        self.signing != Signing::Idle || !self.field.is_empty() || !self.linked || self.signed_out;
                    if changed {
                        self.empty();
                    }
                    self.signing = Signing::Idle;
                    self.linked = true;
                    self.signed_out = false;
                    changed
                }
            },
            Told::Kept { written } => {
                let changed = self.unkept == written;
                self.unkept = !written;
                changed
            }
        };
        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }
}
