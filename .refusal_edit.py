from pathlib import Path
base=Path('temper/crates/temper-engine-domain-forge/src')
p=base/'refusal.rs'
s=p.read_text(); pos=s.index('/// A refused forge bound')
s=s[:pos]+'''pub(crate) fn resource_limit(
    out: &mut skein_lib::Queue<crate::Request>,
    task: u64,
    operation: &'static [u8],
    resource: Option<crate::Name>,
    member: &'static str,
    configured: u32,
    needed: u64,
) {
    out.push(crate::Request::ResourceRefused {
        task,
        operation,
        resource,
        cause: RefusalCause::Limit { member, configured: u64::from(configured), needed },
    });
}

'''+s[pos:];p.write_text(s)
def replace(path,old,new):
 s=path.read_text(); assert s.count(old)==1,(path,old,s.count(old));path.write_text(s.replace(old,new))
p=base/'design.rs'
replace(p,'emit(out, Request::Refused { task });\n        return false;', '''crate::refusal::resource_limit(out, task, b"claim design", names.first().cloned(),
            "forge.resources_per_task", env.limits.resources_per_task, skein_lib::len_u64(names.len()));
        return false;''')
replace(p,'''if !replacing && hold.design_writers.len() >= skein_lib::usize_of(env.limits.holds) {
            emit(out, Request::Refused { task });''','''if !replacing && hold.design_writers.len() >= skein_lib::usize_of(env.limits.holds) {
            crate::refusal::resource_limit(out, task, b"claim design", Some(name.clone()),
                "forge.holds", env.limits.holds, skein_lib::len_u64(hold.design_writers.len()).saturating_add(1));''')
p=base/'domain.rs'
for needle in ['''if !present && list.push(skein_forge_client::Watch { resource, participating }).is_err()''','''if !present && list.push(skein_forge_client::Watch { resource, participating: false }).is_err()''']:
 replace(p,needle+''' {
                emit(out, Request::Refused { task });''',needle+''' {
                crate::refusal::resource_limit(out, task, b"unwatch resources", Some(name.clone()),
                    "forge.client.resources", env.limits.client.resources, u64::from(list.len()).saturating_add(1));''')
replace(p,'''    if !holdable
        || !valid_name(d, &env.limits, &resource)
        || (!d.holds.contains_key(&resource) && d.holds.len() == env.limits.holds)
    {''','''    if let Err((resource, cause)) = admission::validate(d, &env.limits, &resource) {
        emit(out, Request::ResourceRefused { task, operation: b"hold resource", resource, cause });
        return;
    }
    if !holdable {''')
replace(p,'''    if let Some(existing) = d.holds.get(&resource) {''','''    if !d.holds.contains_key(&resource) && d.holds.len() == env.limits.holds {
        crate::refusal::resource_limit(out, task, b"hold resource", Some(resource),
            "forge.holds", env.limits.holds, u64::from(d.holds.len()).saturating_add(1));
        return;
    }
    if let Some(existing) = d.holds.get(&resource) {''')
replace(p,'''if writes.len() > skein_lib::usize_of(env.limits.resources_per_task) {
        emit(out, Request::Refused { task });''','''if writes.len() > skein_lib::usize_of(env.limits.resources_per_task) {
        crate::refusal::resource_limit(out, task, b"claim resources", writes.first().cloned(),
            "forge.resources_per_task", env.limits.resources_per_task, skein_lib::len_u64(writes.len()));''')
replace(p,'''    if repository.role == Role::Context
        || !effect_permitted(repository.kinds, &entry.effect.write)
        || (!d.entries.contains_key(&entry.number) && d.entries.len() == env.limits.entries)
    {''','''    if repository.role == Role::Context || !effect_permitted(repository.kinds, &entry.effect.write) {''')
replace(p,'''    let release = Some(entry.number);
    keep_entry''','''    if d.entries.len() == env.limits.entries {
        crate::refusal::resource_limit(out, task, b"enqueue effect", None,
            "forge.entries", env.limits.entries, u64::from(d.entries.len()).saturating_add(1));
        return;
    }
    let release = Some(entry.number);
    keep_entry''')
replace(p,'''if gates.len() > skein_lib::usize_of(env.limits.change_policy.gates) {
        emit(out, Request::Refused { task });''','''if gates.len() > skein_lib::usize_of(env.limits.change_policy.gates) {
        crate::refusal::resource_limit(out, task, b"step change", None,
            "forge.change_policy.gates", env.limits.change_policy.gates, skein_lib::len_u64(gates.len()));''')
replace(p,'''if d.steps.len() == env.limits.changes {
        emit(out, Request::Refused { task });''','''if d.steps.len() == env.limits.changes {
        crate::refusal::resource_limit(out, task, b"step change", None,
            "forge.changes", env.limits.changes, u64::from(d.steps.len()).saturating_add(1));''')
replace(p,'''if d.entries.contains_key(&pending.entry) || d.entries.len() == env.limits.entries {
        emit(out, Request::Refused { task: row.task });
        return;
    }''','''if d.entries.contains_key(&pending.entry) {
        emit(out, Request::Refused { task: row.task });
        return;
    }
    if d.entries.len() == env.limits.entries {
        crate::refusal::resource_limit(out, row.task, b"emit change effect", None,
            "forge.entries", env.limits.entries, u64::from(d.entries.len()).saturating_add(1));
        return;
    }''')
replace(p,'''if !record_review(row, env, number, head, (status, failed, cancelled), words) {
                emit(out, Request::Refused { task });''','''if let Err(cause) = record_review(row, env, number, head, (status, failed, cancelled), words) {
                emit(out, Request::ResourceRefused { task, operation: b"record gate review", resource: None, cause });''')
s=p.read_text();start=s.index('fn record_review(');end=s.index('\nfn change_terminal',start);r=s[start:end];r=r.replace(') -> bool {',') -> Result<(), crate::RefusalCause> {').replace('return true;', 'return Ok(());').replace('''if !failed && verdicts.push(report).is_err() {
        return false;
    }''','''if !failed && verdicts.push(report).is_err() {
        return Err(crate::RefusalCause::Limit { member: "forge.change_policy.gates",
            configured: u64::from(env.limits.change_policy.gates), needed: u64::from(verdicts.len()).saturating_add(1) });
    }''').replace('''return false;''','''return Err(crate::RefusalCause::Limit { member: "forge.client.answer_bytes",
            configured: u64::from(env.limits.client.answer_bytes), needed: text.minimum_room() });''').replace('''    true\n}''','''    Ok(())\n}''');s=s[:start]+r+s[end:];p.write_text(s)
p=base/'domain/admission.rs';replace(p,'type Failed =','pub(super) type Failed =');replace(p,'fn validate(d:', 'pub(super) fn validate(d:')
# Keep the typed union error all the way to the caller.
s=p.read_text();start=s.index('pub(super) fn landing_subscription_problem(');end=s.index('\nfn add(',start)
s=s[:start]+'''pub(super) fn landing_subscription_problem(
    d: &Domain,
    limits: &Limits,
    repository: skein_forge::api::Repository,
    branch: &[u8],
) -> Result<(), Failed> {
    let resource = Resource { repository, what: skein_forge_client::What::Branch(Box::from(branch)) };
    let Some(name) = super::from_client(&resource, limits) else {
        return Err((None, RefusalCause::InvalidName));
    };
    let mut watches = Map::uncapped();
    for (_, names) in &d.names {
        add(d, limits, names, &mut watches)?;
    }
    add(d, limits, crate::tiers::watched(d).as_slice(), &mut watches)?;
    add(d, limits, core::slice::from_ref(&name), &mut watches)?;
    finish(watches, limits, Some(name))?;
    Ok(())
}
''' + s[end:];p.write_text(s)
p=base/'result.rs';replace(p,'''    /// Fit from the end, marking every omitted byte.''','''    /// Least room for either the whole text or a mark counting every omitted byte.
    pub(crate) fn minimum_room(&self) -> u64 {
        let length = skein_lib::len_u64(self.bytes.len());
        let lost = self.cut.saturating_add(length);
        let marker = 2_u64.saturating_add(skein_lib::len_u64(Decimal::of(lost).as_bytes().len()))
            .saturating_add(skein_lib::len_u64(b" bytes cut]\\n".len()));
        if self.cut == 0 { length.min(marker) } else { marker }
    }

    /// Fit from the end, marking every omitted byte.''')
