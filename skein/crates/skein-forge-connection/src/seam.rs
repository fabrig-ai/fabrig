//! Machine-world state access (skein/docs/design/forge/connection.md, 10).
//! State access requires naming this module and importing its traits.
pub use crate::connection::{Connection, Phase};
pub use crate::installations::{Accepted, Clock, Diff, GitHub, HeldToken, Read, Reading, Row, Selected, Tokens};
pub use crate::puller::State;
use crate::{Event, Limits};
use skein_lib::{Duration, Env, Queue, Secret, Time, Wall};

/// State inspected and seeded on the component by the machine world.
pub trait Access {
    fn test_github_bearer(&mut self, app: u64, key: &Secret, wall: Wall, deadline: Duration) -> Option<&Secret>;
    fn test_calls_ready(&mut self, access: crate::Access);
    #[must_use]
    fn test_github(&self) -> &GitHub;
    fn test_github_mut(&mut self) -> &mut GitHub;
    fn test_local(&mut self, env: &Env<Limits>, up: &mut Queue<Event>);
    fn test_last_connection(&mut self) -> &mut Connection;
    fn test_open_call(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>);
    #[must_use]
    fn test_github_handoffs(&self) -> u32;
}

/// Deadline access on a connection inspected by the machine world.
pub trait ConnectionAccess {
    #[must_use]
    fn test_deadline(&self) -> Option<Time>;
    fn test_until(&mut self, until: Option<Time>);
}
