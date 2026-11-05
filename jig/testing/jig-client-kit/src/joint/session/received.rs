use super::{ClientId, Event, Session, Time, Wall};
impl Session {
    /// Record consumption of the authentication notification by the shell.
    pub fn received(&mut self, client: ClientId, now: Time, wall: Wall, event: &Event) {
        self.stopping().expect("whole original stopped routes");
        if matches!(event, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } }) {
            self.reloaded.remove(&client);
        }
        let _ = (now, wall);
    }
}
