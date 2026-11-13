//! Startup vocabulary shared with the component (protocol/services.md, 4.2).
pub mod application;
pub mod checks;
mod shared;
pub use application::sign_in::url as sign_in_url;
pub use shared::{AGENT_WORDS, admission, admission_words};
