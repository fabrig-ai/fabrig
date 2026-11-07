//! The configured LLM providers whose accounts this child holds (accounts.md, 3.1 and 3.4;
//! configuration.md, 3). Numbered apart from identity providers (people.md, 3).
use alloc::boxed::Box;

/// Maximum UTF-8 bytes in a configured provider name.
pub const PROVIDER_NAME: u32 = 256;
/// Maximum bytes in a public manage-usage HTTPS address.
pub const MANAGE_USAGE: u32 = 2_048;

/// Every configured LLM provider; empty when the configuration names none.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Providers {
    pub providers: Box<[Provider]>,
}

impl Providers {
    /// No provider configured.
    #[must_use]
    pub fn none() -> Self {
        Self { providers: Box::new([]) }
    }
    /// The provider of this number.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
    pub fn get(&self, number: u32) -> Option<&Provider> {
        for provider in &self.providers {
            if provider.number == number {
                return Some(provider);
            }
        }
        None
    }
}

/// One LLM provider: its number and the sources it offers.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Provider {
    pub number: u32,
    pub name: Box<[u8]>,
    pub signed_in_plan: bool,
    pub manage_usage: Option<Box<[u8]>>,
    pub offers: Offers,
    pub list: Option<ProviderList>,
}

/// Which names of a provider's list are models for this work (jig/docs/design/domain/accounts.md, 3.7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProviderList {
    pub keep: Box<[Box<[u8]>]>,
    pub drop: Box<[Box<[u8]>]>,
}

/// The sources a provider offers its accounts. Offering none of them is the `none` source:
/// nothing held, as for a local model (accounts.md, 3.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Offers {
    /// A sign-in, with the provider's public registration, kept by configuration for the driver.
    pub sign_in: bool,
    pub key: bool,
    /// Program names, such as `claude_code` for Claude Code.
    pub programs: Box<[Box<[u8]>]>,
}

impl Offers {
    /// Offering no source: the provider's one account is the deployment's `none` account.
    #[must_use]
    pub fn none(&self) -> bool {
        !self.sign_in && !self.key && self.programs.is_empty()
    }
}

impl Provider {
    /// Program sources remain plans independently of the signed-in source's terms.
    #[must_use]
    pub const fn plan(&self, kind: crate::Kind) -> bool {
        match kind {
            crate::Kind::SignIn => self.signed_in_plan,
            crate::Kind::Program => true,
            crate::Kind::Key | crate::Kind::None => false,
        }
    }
}
