//! A composition's document and typed figures (configuration/limit-settings.md, 8).
use serde_json::Value;
use temper_service::limits;

/// Only the document serializes; the figures cross the in-process start typed.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct Configuration {
    pub document: Value,
    #[serde(skip)]
    pub jig: jig_protocol_config::Limits,
    #[serde(skip)]
    pub limits: limits::Limits,
    #[serde(skip)]
    workspaces_given: bool,
    #[serde(skip)]
    operations_given: bool,
    #[serde(skip)]
    jig_given: Vec<String>,
}
impl Configuration {
    #[must_use]
    pub fn stock(document: Value) -> Self {
        let stock = limits::stock();
        let (jig, limits) = limits::base(stock.live_goals, stock.tool_payload, temper_service_core::derived::STOCK)
            .expect("stock limits");
        let mut world =
            Self { document, jig, limits, workspaces_given: false, operations_given: false, jig_given: Vec::new() };
        world.derive_places();
        world
    }
    /// Import a jig world's figures before its document becomes temper's.
    #[must_use]
    pub fn from_jig(document: Value) -> Self {
        let base = Self::stock(Value::Null);
        Self::from_jig_over(document, &base.jig, &base.limits)
    }
    #[must_use]
    pub fn from_jig_over(mut document: Value, base: &jig_protocol_config::Limits, limits: &limits::Limits) -> Self {
        let mut jig = *base;
        let mut own = *limits;
        let mut jig_given = Vec::new();
        if let Some(engine) = document.get_mut("engine").and_then(Value::as_object_mut) {
            if let Some(value) = engine.remove("limits") {
                for (name, value) in value["overrides"].as_object().expect("jig fixture overrides") {
                    // Watch rooms are derived from streams in temper.
                    if name == "views.watchers" || name == "views.watchers_per_party" {
                        continue;
                    }
                    jig_protocol_config::overrides::apply(
                        &mut jig,
                        name.as_bytes(),
                        value.as_u64().expect("jig fixture figure"),
                    )
                    .expect("jig fixture limit");
                    jig_given.push(name.clone());
                }
            }
            if let Some(memory) = engine.remove("memory_bytes") {
                own.root.transcript_memory = memory.as_u64().expect("jig fixture budget");
            }
        }
        let mut workspaces_given = false;
        let mut operations_given = false;
        if document.get("engine").is_some()
            && let Some(application) = document.get_mut("application").and_then(Value::as_object_mut)
            && let Some(value) = application.remove("limits")
        {
            for (name, value) in value["overrides"].as_object().expect("temper fixture overrides") {
                workspaces_given |= name == "checkout.workspaces";
                operations_given |= name == "git_component.operations";
                limits::apply(&mut own, name.as_bytes(), value.as_u64().expect("temper fixture figure"))
                    .expect("temper fixture limit");
            }
        }
        if let Some(workers) = document.get_mut("workers").and_then(Value::as_array_mut) {
            for worker in workers {
                for member in ["memory_bytes", "limits", "agent"] {
                    worker.as_object_mut().expect("fixture worker").remove(member);
                }
            }
        }
        let mut world = Self { document, jig, limits: own, workspaces_given, operations_given, jig_given };
        world.resolve_jig();
        world.derive_places();
        world
    }
    #[must_use]
    pub fn jig_limits_document(&self) -> Value {
        let mut values = serde_json::Map::new();
        for name in &self.jig_given {
            values.insert(name.clone(), Value::from(self.jig_value(name)));
        }
        serde_json::json!({"overrides":values})
    }
    #[must_use]
    pub fn jig_value(&self, name: &str) -> u64 {
        (jig_protocol_config::names::find(name.as_bytes()).expect("jig figure").value)(&self.jig)
    }
    #[must_use]
    pub fn temper_value(&self, name: &str) -> u64 {
        (limits::find(name.as_bytes()).expect("temper figure").value)(&self.limits)
    }
    pub fn jig_over(&mut self, name: impl AsRef<str>, value: impl TryInto<u64>) {
        let value = value.try_into().unwrap_or_else(|_| panic!("world's jig figure"));
        let name = name.as_ref();
        jig_protocol_config::overrides::apply(&mut self.jig, name.as_bytes(), value).expect("world's jig limit");
        if !self.jig_given.iter().any(|given| given == name) {
            self.jig_given.push(name.to_owned());
        }
        self.resolve_jig();
    }
    fn resolve_jig(&mut self) {
        self.jig_given.sort();
        let rows: Vec<_> = self
            .jig_given
            .iter()
            .map(|name| jig_domain_core::settings::Override {
                name: name.as_bytes().into(),
                value: (jig_protocol_config::names::find(name.as_bytes()).expect("known world figure").value)(
                    &self.jig,
                ),
                reason: b"world figure".as_slice().into(),
            })
            .collect();
        if let Ok(jig) = jig_protocol_config::overrides::with_overrides(&self.jig, &rows) {
            self.jig = jig;
        }
    }
    pub fn temper_over(&mut self, name: impl AsRef<str>, value: impl TryInto<u64>) {
        let value = value.try_into().unwrap_or_else(|_| panic!("world's temper figure"));
        let name = name.as_ref();
        self.workspaces_given |= name == "checkout.workspaces";
        self.operations_given |= name == "git_component.operations";
        limits::apply(&mut self.limits, name.as_bytes(), value).expect("world's temper limit");
    }
    /// A hosted world's engine supplies the same hub bounds as its current agent.
    pub fn agent_bounds(&mut self, agent: &temper_service::settings::Agent) {
        self.jig_over("hub.accounts", agent.host_limits.accounts);
        self.jig_over("hub.run_calls", agent.host_limits.calls);
        self.jig_over("hub.event_bytes", agent.bounds.message_bytes);
        super::turn_bounds::apply(self, agent.limits.bodies.turn_body);
    }
    /// Reset carrier figures to the stock derivation before a world's agent-specific turn.
    pub fn use_stock_carriers(&mut self) {
        let sources = temper_service_core::derived::STOCK.with_payload(self.limits.tool_payload);
        temper_service_core::derived::engine(&mut self.jig, sources).expect("world carriers");
        limits::derive(&mut self.limits, sources).expect("world carriers");
    }
    /// A world's unoverridden places follow its worker's slots, as at a start.
    pub fn derive_places(&mut self) {
        if let Some(slots) = self.document.pointer("/workers/0/slots").and_then(Value::as_u64) {
            if !self.workspaces_given {
                self.limits.checkout.workspaces = u32::try_from(slots).expect("worker slots");
            }
            if !self.operations_given {
                self.limits.git_component.operations = self.limits.checkout.workspaces;
            }
        }
    }
}
impl core::ops::Deref for Configuration {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.document
    }
}
impl core::ops::DerefMut for Configuration {
    fn deref_mut(&mut self) -> &mut Value {
        &mut self.document
    }
}

/// The retained figures of story A, supplied typed beside its document.
#[must_use]
pub fn story_a(document: Value) -> Configuration {
    let mut world = Configuration::stock(document);
    world.limits.root.transcript_memory = 1_152_921_504_606_846_976_u64;
    world.jig_over("authority.executors", 22_u64);
    world.jig_over("authority.grants", 16_u64);
    world.jig_over("authority.projects", 2_u64);
    world.jig_over("authority.segments", 8_u64);
    world.jig_over("brief.briefs", 1_u64);
    world.jig_over("brief.sections", 8_u64);
    world.jig_over("core.call_answer_bytes", 2048_u64);
    world.jig_over("core.load_bytes", 8192_u64);
    world.jig_over("core.load_slots", 8_u64);
    world.jig_over("core.policy_bytes", 8192_u64);
    world.jig_over("core.resume_bytes", 998_048_u64);
    world.jig_over("core.run_bytes", 32_768_u64);
    world.jig_over("core.templates", 7_u64);
    world.jig_over("documents.chips", 1_u64);
    world.jig_over("documents.lines", 12_u64);
    world.jig_over("documents.rows", 2_u64);
    world.jig_over("documents.text_bytes", 128_u64);
    world.jig_over("fleet.attempts", 1_u64);
    world.jig_over("fleet.calls", 1_u64);
    world.jig_over("fleet.turns", 8_u64);
    world.jig_over("hub.transcript_bytes", 3_992_192_u64);
    world.jig_over("hub.turn_bytes", 998_044_u64);
    world.jig_over("hub.turn_queue_bytes", 7_984_352_u64);
    world.jig_over("notes.entries_per_scope", 2_u64);
    world.jig_over("notes.party_entries", 2_u64);
    world.jig_over("notes.recalled", 2_u64);
    world.jig_over("notes.scopes", 32_u64);
    world.jig_over("people.amendment_bytes", 1218_u64);
    world.jig_over("people.holdings", 2_u64);
    world.jig_over("people.inbox_tasks", 8_u64);
    world.jig_over("people.operations", 2_u64);
    world.jig_over("people.pending", 5_u64);
    world.jig_over("people.people", 1_u64);
    world.jig_over("people.projects", 2_u64);
    world.jig_over("people.sign_ins", 2_u64);
    world.jig_over("people.waiters", 2_u64);
    world.jig_over("people.words", 512_u64);
    world.jig_over("tasks.authority_bytes", 75_u64);
    world.jig_over("tasks.authority_grants", 15_u64);
    world.jig_over("tasks.batch", 1_u64);
    world.jig_over("tasks.charter_documents", 0_u64);
    world.jig_over("tasks.charters", 16_u64);
    world.jig_over("tasks.delegates", 1_u64);
    world.jig_over("tasks.dependencies", 2_u64);
    world.jig_over("tasks.executor_kinds", 22_u64);
    world.jig_over("tasks.exemptions", 2_u64);
    world.jig_over("tasks.funders", 4_u64);
    world.jig_over("tasks.hold_bytes", 256_u64);
    world.jig_over("tasks.message_bytes", 512_u64);
    world.jig_over("tasks.project_goals", 1_u64);
    world.jig_over("tasks.project_tasks", 4_u64);
    world.jig_over("tasks.parameters", 5_u64);
    world.jig_over("tasks.contract_choices", 2_u64);
    world.jig_over("tasks.result_bytes", 128_u64);
    world.jig_over("tasks.spec_bytes", 187_u64);
    world.jig_over("tasks.tasks", 4_u64);
    world.jig_over("tasks.tree_tasks", 6_u64);
    world.jig_over("views.runs", 1_u64);
    world.jig_over("views.text_bytes", 1024_u64);
    world.jig_over("tasks.inbox_bytes", 4096_u64);
    world.jig_over("tasks.run_inbox", 8_u64);
    world.temper_over("checkout.message_bytes", 4096_u64);
    world.temper_over("checkout.name_bytes", 256_u64);
    world.temper_over("checkout.repositories", 2_u64);
    world.temper_over("forge.name_bytes", 64_u64);
    world.temper_over("forge.resources_per_task", 3_u64);
    world.temper_over("git_component.stdout_bytes", 131_072_u64);
    world.temper_over("root.journal.transcript_bytes", 998_048_u64);
    world.temper_over("root.loads.bytes", 998_292_u64);
    world.temper_over("root.loads.read_bytes", 1_996_088_u64);
    world.temper_over("root.loads.reply_bytes", 998_292_u64);
    world.temper_over("root.loads.transcript_bytes", 998_044_u64);
    world.temper_over("forge.client.answer_bytes", 65_536_u64);
    world.temper_over("forge.client.op_bytes", 512_u64);
    world.temper_over("forge_component.body_bytes", 16_384_u64);
    world.agent_bounds(&super::agent());
    world.derive_places();
    world
}
