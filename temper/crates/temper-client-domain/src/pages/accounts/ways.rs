//! Offered ways, in provider order (client/README.md, 3.5 and 5.5).
use jig_client::contract as c;
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    SignIn { provider: u32, plan: bool },
    Program { provider: u32 },
    Key { provider: u32 },
}
impl Way {
    #[must_use]
    pub const fn provider(self) -> u32 {
        match self {
            Self::SignIn { provider, .. } | Self::Program { provider } | Self::Key { provider } => provider,
        }
    }
}
/// A cut catalogue still offers the facts it holds. At most three ways per provider.
#[must_use]
pub fn ways(providers: &List<c::ProviderOffer>, demo: Option<u32>, holder: c::AccountHolder) -> List<Way> {
    let mut ways = List::with_capacity(providers.len().checked_mul(3).expect("received providers fit"));
    let mut after = None;
    // Borrow the catalogue in number order without copying its names or program lists.
    for _ in 0..providers.len() {
        let mut next: Option<&c::ProviderOffer> = None;
        for offer in providers {
            let passed = match after {
                Some(number) => offer.provider <= number,
                None => false,
            };
            if passed || Some(offer.provider) == demo {
                continue;
            }
            let earlier = match next {
                Some(row) => offer.provider < row.provider,
                None => true,
            };
            if earlier {
                next = Some(offer);
            }
        }
        let Some(offer) = next else { break };
        let provider = offer.provider;
        if offer.sign_in && holder == c::AccountHolder::Own {
            ways.push(Way::SignIn { provider, plan: offer.signed_in_plan }).expect("three per provider");
        }
        if !offer.programs.is_empty() && holder == c::AccountHolder::Own {
            ways.push(Way::Program { provider }).expect("three per provider");
        }
        if offer.key {
            ways.push(Way::Key { provider }).expect("three per provider");
        }
        after = Some(provider);
    }
    ways
}
