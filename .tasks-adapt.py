from pathlib import Path
p=Path('jig/crates/jig-protocol-store-records/src/codec/tasks.rs')
s=p.read_text()+'''
/// Translation context: the family's limits and the row's model-name remap.
/// Carried values have no remap (skein/docs/design/data/codec/maps.md, 4).
pub struct Context {
    limits: wire::Limits,
    moved: Option<[u64; 64]>,
}
impl Context {
    pub(crate) fn new(limits: &wire::Limits, moved: Option<&[u64; 64]>) -> Self {
        Self { limits: *limits, moved: moved.copied() }
    }
}

pub fn tools_to_record(value: &domain::Tools) -> Result<u64, skein_codec::Reason> { Ok(value.0) }
pub fn tools_to_domain(value: u64) -> Result<domain::Tools, skein_codec::Reason> { Ok(domain::Tools(value)) }
pub fn scopes_to_record(value: &domain::Scopes) -> Result<u8, skein_codec::Reason> { Ok(value.0) }
pub fn scopes_to_domain(value: u8) -> Result<domain::Scopes, skein_codec::Reason> { Ok(domain::Scopes(value)) }
pub fn newest_to_record(value: &domain::Newest) -> Result<u64, skein_codec::Reason> { Ok(value.value()) }
pub fn newest_to_domain(value: u64) -> Result<domain::Newest, skein_codec::Reason> { Ok(domain::Newest::of(value)) }
pub fn reasons_to_record(value: &domain::Reasons) -> Result<u32, skein_codec::Reason> { Ok(value.0) }
pub fn reasons_to_domain(value: u32) -> Result<domain::Reasons, skein_codec::Reason> {
    let value = domain::Reasons(value);
    if value.is_known() { Ok(value) } else { Err(skein_codec::Reason::Bound) }
}
'''
# adapters initially retain existing by-hand signatures (externally named pairs stay).
for decl,typ,mod,enc,dec,moved,pure in [
 ('Name','Name','value','name','read_name',False,False),
 ('Spec','Spec','value','spec','read_spec',False,False),
 ('TaskResult','TaskResult','phase','result','read_result',False,False),
 ('Authority','Authority','value','authority','read_authority',True,False),
 ('ProposalAction','ProposalAction','proposals','action','read_action',True,False),
 ('ResultKind','ResultKind','activity','result_kind','read_result_kind',False,False),
 ('Why','Why','value','why','read_why',False,True),
 ('RecurringTemplate','RecurringTemplate','phase','template','read_template',True,False),
 ]:
 import re
 stem=re.sub(r'(?<!^)(?=[A-Z])','_',decl).lower()
 arg='*value' if decl=='Why' else 'value'
 extra=', context.moved.as_ref()' if moved else ''
 s+=f'\npub fn {stem}_to_record(value: &domain::{typ}, context: &Context) -> Result<wire::{decl}, wire::Problem> {{\n    {mod}::{enc}({arg}, &context.limits{extra})\n}}\n'
 call=f'{mod}::{dec}(record'+('' if pure else ', &context.limits')+')'
 if pure:call=f'Ok({call})'
 s+=f'pub fn {stem}_to_domain(record: wire::{decl}, context: &Context) -> Result<domain::{typ}, wire::Problem> {{\n    {call}\n}}\n'
s+='''
pub fn part_path_to_record(value: &skein_lib::List<Box<[u8]>>, context: &Context) -> Result<wire::PartPath, wire::Problem> {
    let limits = &context.limits;
    let segments = pieces::part_segments(value.as_slice(), limits, limits.part_path_segments, wire::paths::PART_PATH_SEGMENTS)?;
    let mut bytes = 0_usize;
    for segment in value { bytes = bytes.saturating_add(segment.len()); }
    if bytes > skein_lib::length::usize_of(limits.part_segment_bytes) {
        return Err(wire::paths::PART_PATH_SEGMENTS.problem(skein_codec::Reason::Bound));
    }
    Ok(wire::PartPath { segments })
}
pub fn part_path_to_domain(record: wire::PartPath, context: &Context) -> Result<skein_lib::List<Box<[u8]>>, wire::Problem> {
    let limits = &context.limits;
    let segments = pieces::read_part_segments(record.segments, limits, wire::paths::PART_PATH_SEGMENTS)?;
    let mut members = skein_lib::List::with_capacity(u32::try_from(segments.len()).expect("bounded path"));
    let mut bytes = 0_usize;
    for segment in segments {
        bytes = bytes.saturating_add(segment.len());
        members.push(segment).expect("same number of segments");
    }
    if bytes > skein_lib::length::usize_of(limits.part_segment_bytes) {
        return Err(wire::paths::PART_PATH_SEGMENTS.problem(skein_codec::Reason::Bound));
    }
    Ok(members)
}
pub fn result_followups_delegates_to_record(value: &[domain::New], context: &Context) -> Result<wire::ResultFollowupsDelegates, wire::Problem> {
    let limits = &context.limits;
    Ok(wire::ResultFollowupsDelegates { value: pieces::creations(value, limits, limits.result_followups_delegates_value, wire::paths::RESULT_FOLLOWUPS_DELEGATES_VALUE, context.moved.as_ref())? })
}
pub fn result_followups_delegates_to_domain(record: wire::ResultFollowupsDelegates, context: &Context) -> Result<Box<[domain::New]>, wire::Problem> {
    pieces::read_creations(record.value, &context.limits, wire::paths::RESULT_FOLLOWUPS_DELEGATES_VALUE)
}
'''
p.write_text(s)
# Extract the owner-shaped PersonAction rather than constructing a temporary domain row.
p=Path('jig/crates/jig-protocol-store-records/src/codec/tasks/proposals.rs');s=p.read_text()
start=s.index('    let action = match action {',s.index('pub fn person_proposal('))
end=s.index('    Ok(wire::PersonProposal {',start)
enc=s[start:end].replace('    let action = match action {','    Ok(match value {',1).rstrip();assert enc.endswith('};');enc=enc[:-2]+'})\n'
s=s[:start]+'    let action = super::person_action_to_record(action, &super::Context::new(limits, moved))?;\n'+s[end:]
start=s.index('    let action = match action {',s.index('pub fn read_person_proposal('));end=s.index('    Ok(domain::PersonProposal',start)
dec=s[start:end].replace('    let action = match action {','    Ok(match value {',1).rstrip();assert dec.endswith('};');dec=dec[:-2]+'})\n'
s=s[:start]+'    let action = super::person_action_to_domain(action, &super::Context::new(limits, None))?;\n'+s[end:]
s+='\npub fn person_action_to_record(value: &domain::PersonAction, context: &super::Context) -> Result<wire::PersonAction, wire::Problem> {\n    let limits = &context.limits;\n    let moved = context.moved.as_ref();\n'+enc+'}\n'
s+='pub fn person_action_to_domain(value: wire::PersonAction, context: &super::Context) -> Result<domain::PersonAction, wire::Problem> {\n    let limits = &context.limits;\n'+dec+'}\n'
p.write_text(s)
p=Path('jig/crates/jig-protocol-store-records/src/codec/tasks.rs');s=p.read_text()+'\npub use proposals::{person_action_to_record, person_action_to_domain};\n';p.write_text(s)
