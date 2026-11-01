//! Serve's playground read, key handover and fake setup host over the engine world's doors
//! (temper/docs/design/draft/playground-reset.md, 6; domain/testing.md, 6.3).
use super::{ClientId, Joint, Request, RootEvent, engine};
use skein_lib::Duration;
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain_forge as forge;

impl Joint {
    #[must_use]
    pub fn playground(&self) -> Option<forge::Playground> {
        self.engine.root.playground()
    }

    pub(super) fn playground_read(&mut self, who: ClientId, request: &Request) -> bool {
        let Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
        else {
            return false;
        };
        let answer = if record.family == wire::FORGE
            && record.version == wire::CLIENT_VERSION
            && wire::decode_reading(&record.bytes, &w::CEILINGS)
                .is_some_and(|reading| *reading.read() == w::Read::Playground)
        {
            self.playground_config.as_ref().map(|config| {
                let row = self.playground();
                let mut cues =
                    skein_lib::List::with_capacity(u32::try_from(self.playground_cues.len()).expect("configured cues"));
                for cue in &self.playground_cues {
                    cues.push(cue.clone()).expect("configured cue");
                }
                let answer = w::PlaygroundAnswer::new(
                    &w::CEILINGS,
                    w::PlaygroundAnswerParts {
                        project: row.and_then(|row| row.project).unwrap_or(0),
                        forge: row.map_or(0, |row| row.forge),
                        state: match row.map(|row| row.state) {
                            None => w::PlaygroundState::NotStarted,
                            Some(forge::PlaygroundState::Running) => w::PlaygroundState::Running,
                            Some(forge::PlaygroundState::Resetting(_)) => w::PlaygroundState::Resetting,
                        },
                        provider: config.provider,
                        owner: config.adoption.owner.clone(),
                        name: config.adoption.name.clone(),
                        cues,
                    },
                )
                .expect("configured playground fits the wire");
                wire::playground_answer(&answer)
            })
        } else {
            None
        };
        self.callbacks.push((
            who,
            RootEvent::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::ReadDone {
                        read: *read,
                        result: jig_client::Readout::Family { family: record.family, record: answer },
                    },
                },
            }
            .into(),
        ));
        true
    }

    pub(super) fn playground_key(&mut self, event: &mut engine::Event) {
        if let engine::Event::Party(engine::PartyInput::Application {
            request: engine::Application::Forge(forge::Ask::StartPlayground),
            carried,
            ..
        }) = event
            && self.playground_config.is_some()
        {
            if let Some(unused) = carried.take() {
                self.transport.unused(unused.number);
            }
            if !self.playground_key_available {
                return;
            }
            // Serve reads the configured scripted key; the world supplies its fake file's bytes.
            let value = skein_lib::Secret::new(b"playground-scripted-key".as_slice().into());
            let digest = skein_lib::sha256::digest(value.expose());
            *carried = Some(jig_domain_core::Carried { number: self.transport.carry(value), digest });
        }
    }

    pub(super) fn playground_host(&mut self, request: &engine::Request) {
        if let engine::Request::SetUpGroup { forge, .. } = request
            && self.records().iter().any(|record| record.number == *forge && record.playground)
        {
            self.engine.forge = Some(
                temper_engine_domain_world::forge_stand::ForgeStand::with(&[(
                    b"playground",
                    b"widget",
                    crate::fixtures::PLAYGROUND_REPOSITORY,
                )])
                .at_forge(*forge),
            );
            if !self.hold_playground_connection {
                self.connect_playground();
            }
        }
    }

    /// The fake setup host finished connecting; adoption remains the engine's work.
    pub fn connect_playground(&mut self) {
        self.hold_playground_connection = false;
        let row = self.playground().expect("playground made before setup");
        self.engine.send(engine::Event::Credential {
            number: u32::from(row.forge),
            generation: 1,
            valid: Duration::from_secs(3600),
        });
        self.engine.send(engine::Event::Forge(Box::new(forge::Event::Connection {
            forge: row.forge,
            state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
        })));
    }
}
