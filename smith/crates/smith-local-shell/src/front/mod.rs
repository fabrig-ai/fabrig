//! Product front ends render typed values, never read a destination's bytes.
pub mod headless;
pub mod interactive;
mod remote;

/// The selected product presentation, priced before the local host adopts resources.
#[expect(clippy::large_enum_variant, reason = "front variants stay inline and are priced before resource adoption")]
pub enum Front {
    /// One headless outcome and its output policy.
    Headless(headless::Headless),
    /// The line-mode typed live view.
    Interactive(interactive::Interactive),
}

impl Front {
    pub(crate) const fn headless(&self) -> Option<&headless::Headless> {
        match self {
            Self::Headless(front) => Some(front),
            Self::Interactive(_) => None,
        }
    }

    pub(crate) const fn headless_mut(&mut self) -> Option<&mut headless::Headless> {
        match self {
            Self::Headless(front) => Some(front),
            Self::Interactive(_) => None,
        }
    }

    pub(crate) const fn interactive(&self) -> Option<&interactive::Interactive> {
        match self {
            Self::Interactive(front) => Some(front),
            Self::Headless(_) => None,
        }
    }

    pub(crate) const fn interactive_mut(&mut self) -> Option<&mut interactive::Interactive> {
        match self {
            Self::Interactive(front) => Some(front),
            Self::Headless(_) => None,
        }
    }
}
