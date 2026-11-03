//! Party policy edits and snapshots (jig/docs/design/domain/authority.md, 6).

use crate::translate::{
    authority_to_people, grant_to_authority, grant_to_people, people_authority, requirement_to_authority,
    requirement_to_people,
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use skein_lib::List;

fn role_exists(policy: &authority::Policy, number: u32) -> bool {
    for role in &policy.roles {
        if role.number == number {
            return true;
        }
    }
    false
}

fn map_valid(policy: &authority::Policy, mappings: &[people::PermissionRole]) -> bool {
    for (index, mapping) in mappings.iter().enumerate() {
        if mapping.role > 3 && !role_exists(policy, mapping.role) {
            return false;
        }
        for earlier in mappings.get(..index).expect("enumerated mapping is in bounds") {
            if earlier.connector == mapping.connector && earlier.permission == mapping.permission {
                return false;
            }
        }
    }
    true
}

/// Apply one mutable edit, keeping the ceiling and deployment rules.
pub(crate) fn apply(
    policy: &mut authority::Policy,
    permissions: &mut Box<[people::PermissionRole]>,
    templates: &mut Box<[people::TemplateOverride]>,
    change: people::PolicyChange,
    requirements_limit: u32,
    roles_limit: u32,
) -> Option<()> {
    match change {
        people::PolicyChange::Templates { overrides } => *templates = overrides,
        people::PolicyChange::Projections { grants } => policy.projections = grants_to_authority(grants)?,
        people::PolicyChange::ProjectSpend { period_spend } => policy.period_spend = period_spend,
        people::PolicyChange::RoleSpend { role, period_spend } => {
            let mut found = false;
            for value in &mut policy.roles {
                if value.number == role {
                    value.period_spend = period_spend;
                    found = true;
                }
            }
            if !found {
                return None;
            }
        }
        people::PolicyChange::Role(edit) => {
            if edit.number <= 3 && !role_exists(policy, edit.number) {
                return None;
            }
            let mut inherited = policy.ceiling.models;
            for role in &policy.roles {
                if role.number == edit.number {
                    inherited = role.authority.models;
                }
            }
            let mut given = edit.authority;
            if given.models.is_none() {
                given.models = Some(people::Models::from_parts(inherited.bits(), inherited.is_all()));
            }
            let value = people_authority(given)?;
            let mut found = false;
            for role in &mut policy.roles {
                if role.number == edit.number {
                    role.authority = value.clone();
                    role.period_spend = edit.period_spend;
                    role.requests = authority::Requests(edit.requests);
                    role.decides = authority::Proposals(edit.decides);
                    found = true;
                }
            }
            if !found {
                let mut roles = List::with_capacity(roles_limit);
                for role in &policy.roles {
                    roles.push(role.clone()).ok()?;
                }
                roles
                    .push(authority::Role {
                        number: edit.number,
                        authority: value,
                        period_spend: edit.period_spend,
                        requests: authority::Requests(edit.requests),
                        decides: authority::Proposals(edit.decides),
                    })
                    .ok()?;
                policy.roles = roles.into_boxed();
            }
        }
        people::PolicyChange::Requirements { requirements } => {
            if u32::try_from(requirements.len()).ok()? > requirements_limit {
                return None;
            }
            let mut translated = List::with_capacity(requirements_limit);
            for requirement in requirements {
                translated.push(requirement_to_authority(requirement)).ok()?;
            }
            policy.requirements = translated.into_boxed();
        }
        people::PolicyChange::Permissions { mappings } => *permissions = mappings,
    }
    if !map_valid(policy, permissions) {
        return None;
    }
    Some(())
}

/// Capture the whole policy after a validated policy event.
#[must_use]
pub fn snapshot(
    policy: &authority::Policy,
    permissions: &[people::PermissionRole],
    templates: &[people::TemplateOverride],
) -> Option<people::PolicyValue> {
    let mut roles = List::with_capacity(u32::try_from(policy.roles.len()).ok()?);
    for role in &policy.roles {
        roles
            .push(people::PolicyRole {
                number: role.number,
                authority: authority_to_people(&role.authority)?,
                period_spend: role.period_spend,
                requests: role.requests.0,
                decides: role.decides.0,
            })
            .ok()?;
    }
    let mut requirements = List::with_capacity(u32::try_from(policy.requirements.len()).ok()?);
    for requirement in &policy.requirements {
        requirements.push(requirement_to_people(requirement)).ok()?;
    }
    Some(people::PolicyValue {
        ceiling: authority_to_people(&policy.ceiling)?,
        escalation_role: policy.escalation_role,
        period_spend: policy.period_spend,
        projections: grants_to_people(&policy.projections)?,
        roles: roles.into_boxed(),
        requirements: requirements.into_boxed(),
        permissions: Box::from(permissions),
        templates: Box::from(templates),
    })
}

fn restore_value(
    policy: &mut authority::Policy,
    permissions: &mut Box<[people::PermissionRole]>,
    templates: &mut Box<[people::TemplateOverride]>,
    value: people::PolicyValue,
    limit: u32,
) -> Option<()> {
    policy.ceiling = people_authority(value.ceiling)?;
    policy.escalation_role = value.escalation_role;
    policy.period_spend = value.period_spend;
    policy.projections = grants_to_authority(value.projections)?;
    let mut roles = List::with_capacity(u32::try_from(value.roles.len()).ok()?);
    for role in value.roles {
        roles
            .push(authority::Role {
                number: role.number,
                authority: people_authority(role.authority)?,
                period_spend: role.period_spend,
                requests: authority::Requests(role.requests),
                decides: authority::Proposals(role.decides),
            })
            .ok()?;
    }
    policy.roles = roles.into_boxed();
    if u32::try_from(value.requirements.len()).ok()? > limit {
        return None;
    }
    let mut requirements = List::with_capacity(limit);
    for requirement in value.requirements {
        requirements.push(requirement_to_authority(requirement)).ok()?;
    }
    policy.requirements = requirements.into_boxed();
    *permissions = value.permissions;
    *templates = value.templates;
    Some(())
}

use crate::{Core, Limits};
use skein_lib::Queue;

impl Core {
    /// Apply one authenticated mutable policy edit in the core's authority and party vocabularies.
    pub(crate) fn change_policy(
        &mut self,
        limits: &Limits,
        person: u64,
        project: u32,
        change: people::PolicyChange,
    ) -> Result<(people::PolicyValue, crate::funding::Resize), people::Refusal> {
        self.role_allowed(person, project, limits.tasks.tree_tasks)?;
        let Some(mut policy) = self.authority.policy(project).cloned() else {
            return Err(people::Refusal::Unknown);
        };
        match &change {
            people::PolicyChange::Role(role) => {
                if let Some(models) = role.authority.models {
                    let models = authority::Models::from_parts(models.bits(), models.is_all());
                    if !models.at_most(policy.ceiling.models) || !models.at_most(self.authority.rules().ceiling.models)
                    {
                        return Err(people::Refusal::Authority { proposable: None, lacking: people::Lack::Models });
                    }
                }
            }
            people::PolicyChange::Templates { .. }
            | people::PolicyChange::Projections { .. }
            | people::PolicyChange::RoleSpend { .. }
            | people::PolicyChange::ProjectSpend { .. }
            | people::PolicyChange::Requirements { .. }
            | people::PolicyChange::Permissions { .. } => {}
        }
        let mut permissions = match self.permission_roles.get(&project) {
            Some(mappings) => mappings.clone(),
            None => Box::new([]),
        };
        if let people::PolicyChange::Templates { overrides } = &change {
            if overrides.len() > skein_lib::length::usize_of(limits.templates) {
                return Err(people::Refusal::Limit { which: people::LimitKind::Policy });
            }
            for (index, entry) in overrides.iter().enumerate() {
                let mut known = false;
                for setting in &self.current.offered() {
                    let number = match setting {
                        crate::settings::Setting::Template(number) => *number,
                        crate::settings::Setting::Charter(_)
                        | crate::settings::Setting::Policy(_)
                        | crate::settings::Setting::Rules
                        | crate::settings::Setting::ProjectPolicy { .. } => continue,
                        jig_domain_people::Setting::Application { .. } => {
                            unreachable!("an application's setting: apps-core")
                        }
                    };
                    let template = self.current.template(number).expect("offered template");
                    known |= template.number == entry.template;
                }
                if !known {
                    return Err(people::Refusal::Unknown);
                }
                for previous in overrides.iter().take(index) {
                    if previous.template == entry.template {
                        return Err(people::Refusal::Limit { which: people::LimitKind::Duplicate });
                    }
                }
            }
        }
        let mut templates = match self.template_overrides.get(&project) {
            Some(entries) => entries.clone(),
            None => Box::new([]),
        };
        if apply(
            &mut policy,
            &mut permissions,
            &mut templates,
            change,
            self.authority.limits().requirements,
            self.authority.limits().roles,
        )
        .is_none()
        {
            return Err(people::Refusal::Unknown);
        }
        if !crate::settings::templates::valid(&self.current, &self.settings, &templates) {
            return Err(people::Refusal::Authority { proposable: None, lacking: people::Lack::Shape });
        }
        let (snapshot, resize) = self.check_policy_replacement(
            limits,
            project,
            &crate::ConfiguredPolicy {
                policy: policy.clone(),
                permissions: permissions.clone(),
                templates: templates.clone(),
            },
            self.authority.rules(),
        )?;
        self.commit_policy(project, policy, permissions, templates);
        Ok((snapshot, resize))
    }

    /// The current project policy, whole, independently of revisioned settings (engine.md, 17.2).
    pub(crate) fn configured_policy(&self, project: u32) -> Option<crate::ConfiguredPolicy> {
        Some(crate::ConfiguredPolicy {
            policy: self.authority.policy(project)?.clone(),
            permissions: self.permission_roles.get(&project).cloned().unwrap_or_default(),
            templates: self.template_overrides.get(&project).cloned().unwrap_or_default(),
        })
    }

    /// Owner edits and whole settings replacements share their byte, ceiling and funding checks.
    pub(crate) fn check_policy_replacement(
        &self,
        limits: &Limits,
        project: u32,
        value: &crate::ConfiguredPolicy,
        rules: &authority::Rules,
    ) -> Result<(people::PolicyValue, crate::funding::Resize), people::Refusal> {
        let policy = &value.policy;
        if !map_valid(policy, &value.permissions) {
            return Err(people::Refusal::Unknown);
        }
        let Some(snapshot) = snapshot(policy, &value.permissions, &value.templates) else {
            return Err(people::Refusal::Limit { which: people::LimitKind::Policy });
        };
        if match people::policy_bytes(&snapshot) {
            Some(bytes) => bytes > limits.policy_bytes,
            None => true,
        } {
            return Err(people::Refusal::Limit { which: people::LimitKind::Policy });
        }
        let before = self.authority.policy(project).ok_or(people::Refusal::Unknown)?;
        if let Some(ceiling) = authority::policy_ceiling(rules, before, policy) {
            return Err(ceiling_refusal(ceiling));
        }
        if authority::policy_refusal(rules, self.authority.limits(), policy).is_some() {
            return Err(people::Refusal::Authority { proposable: None, lacking: people::Lack::Shape });
        }
        let resize = self.resize(project, before, policy, limits)?;
        Ok((snapshot, resize))
    }

    pub(crate) fn commit_policy(
        &mut self,
        project: u32,
        policy: authority::Policy,
        permissions: Box<[people::PermissionRole]>,
        templates: Box<[people::TemplateOverride]>,
    ) {
        let mut facts = Queue::uncapped();
        crate::routing::step::step_authority(self, authority::Event::Policy { project, policy }, &mut facts);
        match facts.pop().expect("policy update terminal") {
            authority::PolicyFact::Changed { .. } => {
                assert!(self.permission_roles.insert(project, permissions).is_ok(), "admitted permission policy");
                assert!(self.template_overrides.insert(project, templates).is_ok(), "admitted template policy");
            }
            authority::PolicyFact::Refused { .. } => unreachable!("policy checked before resize"),
            authority::PolicyFact::Added { .. } | authority::PolicyFact::Dropped { .. } => {
                unreachable!("existing project policy replacement")
            }
        }
    }

    /// Add a project from its committed row alone before admitting party requests.
    pub(crate) fn restore_policy(
        &mut self,
        limits: &Limits,
        project: u32,
        name: Box<str>,
        value: people::PolicyValue,
    ) -> bool {
        self.start_input();
        {
            if !self.start_policy_room(project) {
                return false;
            }
        }
        match people::policy_bytes(&value) {
            Some(bytes) if bytes <= limits.policy_bytes => {}
            Some(_) | None => {
                self.refuse_restore(crate::start::Failure::ProjectPolicy { project });
                return false;
            }
        }
        let Some(ceiling) = people_authority(value.ceiling.clone()) else {
            {
                self.refuse_restore(crate::start::Failure::ProjectPolicy { project });
            }
            return false;
        };
        let mut policy = authority::Policy {
            ceiling,
            escalation_role: None,
            projections: Box::new([]),
            period_spend: 0,
            roles: Box::new([]),
            requirements: Box::new([]),
        };
        let mut permissions: Box<[people::PermissionRole]> = Box::new([]);
        if value.templates.len() > skein_lib::length::usize_of(limits.templates) {
            {
                self.refuse_restore(crate::start::Failure::ProjectPolicy { project });
            }
            return false;
        }
        let mut templates: Box<[people::TemplateOverride]> = Box::new([]);
        {
            if restore_value(&mut policy, &mut permissions, &mut templates, value, self.authority.limits().requirements)
                .is_none()
            {
                self.refuse_restore(crate::start::Failure::ProjectPolicy { project });
                return false;
            }
        }
        let mut facts = Queue::uncapped();
        crate::routing::step::step_authority(self, authority::Event::Restore { project, policy }, &mut facts);
        match facts.pop().expect("policy restore terminal") {
            authority::PolicyFact::Added { .. } => {
                self.projects.push(project).expect("preflighted project room");
                self.permission_roles.insert(project, permissions).expect("preflighted permission room");
                self.template_overrides.insert(project, templates).expect("preflighted template room");
                self.names.insert(project, name).expect("preflighted name room");
                true
            }
            authority::PolicyFact::Refused { .. } => {
                {
                    self.refuse_restore(crate::start::Failure::ProjectPolicy { project });
                }
                false
            }
            authority::PolicyFact::Changed { .. } | authority::PolicyFact::Dropped { .. } => {
                unreachable!("absent project policy restoration")
            }
        }
    }

    fn start_policy_room(&mut self, project: u32) -> bool {
        assert!(
            self.authority.policy(project).is_none(),
            "duplicate {:?}",
            crate::Key::People(people::Key::Policy(project))
        );
        for (held, limit) in [
            (self.projects.len(), self.projects.capacity()),
            (self.names.len(), self.names.capacity()),
            (self.permission_roles.len(), self.permission_roles.capacity()),
            (self.template_overrides.len(), self.template_overrides.capacity()),
        ] {
            if !self.start_room(
                crate::start::LiveSet::Projects,
                crate::Key::People(people::Key::Policy(project)),
                held,
                limit,
            ) {
                return false;
            }
        }
        true
    }
}

#[must_use]
pub fn grants_to_authority(grants: Box<[people::Grant]>) -> Option<Box<[authority::Grant]>> {
    let mut result = List::with_capacity(u32::try_from(grants.len()).ok()?);
    for grant in grants {
        result.push(grant_to_authority(grant)).ok()?;
    }
    Some(result.into_boxed())
}
fn grants_to_people(grants: &[authority::Grant]) -> Option<Box<[people::Grant]>> {
    let mut result = List::with_capacity(u32::try_from(grants.len()).ok()?);
    for grant in grants {
        result.push(grant_to_people(grant)).ok()?;
    }
    Some(result.into_boxed())
}

fn ceiling_dimension(value: authority::CeilingDimension) -> people::CeilingDimension {
    match value {
        authority::CeilingDimension::PeriodSpend => people::CeilingDimension::PeriodSpend,
        authority::CeilingDimension::Spend => people::CeilingDimension::Spend,
        authority::CeilingDimension::Tools => people::CeilingDimension::Tools,
        authority::CeilingDimension::Tasks => people::CeilingDimension::Tasks,
        authority::CeilingDimension::Depth => people::CeilingDimension::Depth,
        authority::CeilingDimension::Executors => people::CeilingDimension::Executors,
        authority::CeilingDimension::Grants => people::CeilingDimension::Grants,
        authority::CeilingDimension::Deadline => people::CeilingDimension::Deadline,
        authority::CeilingDimension::Notes => people::CeilingDimension::Notes,
    }
}

fn ceiling_refusal(ceiling: authority::Ceiling) -> people::Refusal {
    people::Refusal::Authority {
        proposable: None,
        lacking: people::Lack::Ceiling {
            holder: match ceiling.holder {
                authority::CeilingHolder::Rules => people::CeilingHolder::Rules,
                authority::CeilingHolder::Policy => people::CeilingHolder::Policy,
                authority::CeilingHolder::Role(role) => people::CeilingHolder::Role(role),
            },
            dimension: ceiling_dimension(ceiling.dimension),
            value: ceiling.value,
            asked: ceiling.asked,
        },
    }
}
