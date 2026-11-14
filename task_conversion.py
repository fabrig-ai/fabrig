from pathlib import Path
import re
R=Path('hearth/crates/hearth-client-protocol-records/src')
# Keep only printer operations, replacing every inherent method with a concrete free function.
def end(s,start):
    # Braces in byte strings occur balanced in these blocks.
    n=0
    for i in range(start,len(s)):
        if s[i]=='{': n+=1
        elif s[i]=='}':
            n-=1
            if n==0:return i+1
    raise ValueError(start)
def snake(s):return re.sub(r'(?<!^)(?=[A-Z])','_',s).lower()
def convert(s):
    # Drop copied definitions and their attached docs/derives.
    s=re.sub(r'(?m)(?:^///[^\n]*\n)*^#\[derive[^\n]*\n(?:pub (?:enum|struct) [^{]+)\{',lambda m: m.group(),s)
    for m in list(re.finditer(r'(?m)(?:^///[^\n]*\n)*^#\[derive[^\n]*\n(?:pub (?:enum|struct) [^{]+)\{',s))[::-1]:
        stop=end(s,m.end()-1);s=s[:m.start()]+s[stop:]
    for m in list(re.finditer(r'impl ([\w:]+) \{',s))[::-1]:
        typ=m[1].removeprefix('super::'); stop=end(s,m.end()-1)
        body=s[m.end():stop-1]
        body='\n'.join(line[4:] if line.startswith('    ') else line for line in body.splitlines())
        body=body.replace('Self::',typ+'::')
        body=re.sub(r'\bself\b','item',body)
        body=re.sub(r'(?:pub\([^)]*\) )?(const )?fn (\w+)\((&item|item)([,\)])',lambda a: 'pub(super) '+(a[1] or '')+'fn '+snake(typ)+'_'+a[2]+'(item: '+('&' if a[3]=='&item' else '')+typ+a[4],body)
        s=s[:m.start()]+body+'\n'+s[stop:]
    return s
files=[R/'errors.rs',*sorted((R/'errors').glob('*.rs'))]
for p in files:
    s=convert(p.read_text())
    # Types are imported privately, not re-exported as another public vocabulary.
    s=re.sub(r'^pub use .*;\n','',s,flags=re.M)
    p.write_text(s)
# Main printer imports, all secondary operations private to this module tree.
p=R/'errors.rs';s=p.read_text()
s=s.replace('use alloc::boxed::Box;','use alloc::boxed::Box;\nuse hearth_domain_render_vocabulary::{errors::{Error, Place, Kind, Class, Decode, Field, Limit, Declared, Object, Expected, Required, Takes, Schema, Format, TypeName, Repository, Tool, Command, Ended, Syntax, Composition, Host, Collision, Id, Subject, Root, ExportFault, Exported, Cause, Resolution, Reserved, Rule, Ids, Owns, Why}, revision::declaration::Map};\nuse collisions::collision_text;\nuse collisions::collision_json;\nuse composition::{composition_text, composition_json};\nuse ids::{ids_text, ids_json};\nuse repository::{repository_text, repository_json, tool_text, tool_json};\nuse reserved::{reserved_text, reserved_json};\nuse resolution::{resolution_text, resolution_json};\nuse schema::{schema_text, schema_json};\nuse syntax::{syntax_text, syntax_json};\nuse validation::{fault_kind_message, fault_kind_name, required_name};\nmod store;\nuse store::{store_error_text, store_error_json};')
s=s.replace('pub mod syntax;','mod syntax;')
s=s.replace('pub(super) fn error_', 'pub(crate) fn ')
s=s.replace('side.name()', 'crate::diff::side(side)')
# Dispatch each variant explicitly, including the exported error's causes.
for variant,typ in [('Exported','exported'),('Ids','ids'),('Reserved','reserved'),('Collision','collision'),('Resolution','resolution'),('Store','store_error'),('Decode','decode'),('Repository','repository'),('Tool','tool'),('Composition','composition'),('Schema','schema')]:
    for enum in ['Kind','Cause']:
        s=re.sub(r'('+enum+r'::'+variant+r'\((\w+)\) => )(\2)\.(text|json)\(([^\n]*)\)',lambda m:m[1]+typ+'_'+m[4]+'('+m[2]+', '+m[5]+')',s)
        # Multiline JSON dispatch within Cause/Kind arm.
        s=re.sub(r'('+enum+r'::'+variant+r'\((\w+)\) => \{.*?)(\2)\.json\(json\)',lambda m:m[1]+typ+'_json('+m[2]+', json)',s,flags=re.S)
for v in ['at','also']: 
    for meth in ['text','json']:s=s.replace(f'{v}.{meth}(', f'place_{meth}({v}, ')
s=s.replace('kind.message()', 'fault_kind_message(*kind)').replace('kind.name()', 'fault_kind_name(*kind)').replace('value.message()', 'fault_kind_message(*value)').replace('value.name()', 'fault_kind_name(*value)')
s=s.replace('syntax.text(lines)', 'syntax_text(syntax, lines)').replace('syntax.json(json)', 'syntax_json(syntax, json)')
s=s.replace('declared.text(lines)', 'declared_text(*declared, lines)').replace('declared.json(json)', 'declared_json(*declared, json)')
for var,typ,deref in [('field','field',True),('name','field',True),('of','object',False),('expected','expected',False),('key','required',False),('map','map',False),('takes','takes',False)]:
    s=s.replace(f'{var}.name()',f'{typ}_name({"*" if deref else ""}{var})')
p.write_text(s)
# Secondary modules only import their concrete value types.
imports={
 'collisions':'Collision, Subject, Id', 'composition':'Composition, Host', 'ids':'Ids, Owns, Why',
 'repository':'Repository, Tool, Command, Ended','reserved':'Reserved, Rule','resolution':'Resolution',
 'schema':'Format, TypeName','syntax':'Syntax','type_name':'TypeName'}
for mod,types in imports.items():
 p=R/'errors'/f'{mod}.rs';s=p.read_text();s='use hearth_domain_render_vocabulary::errors::{'+types+'};\n'+s
 if mod=='syntax':s='use hearth_domain_render_vocabulary::{errors::syntax::Expected, evaluate::Filter};\n'+s
 if mod=='schema':s='use super::type_name::{type_name_article, type_name_name};\n'+s
 for var,typ,byref in [('subject','subject',False),('id','id',False),('root','root',False),('format','format',False),('expected','expected',False)]:
  # Match ergonomics lend enum fields, while helpers' own parameters are values.
  if mod=='collisions':
   s=s.replace(f'{var}.name()',f'{typ}_name(*{var})').replace(f'{var}.prefix(lines)',f'{typ}_prefix(*{var}, lines)')
  elif mod=='schema':
   s=s.replace('format.name()', 'format_name(*format)').replace('root.name()', 'root_name(root)')
   s=s.replace('expected.article()', 'type_name_article(expected)').replace('expected.name()', 'type_name_name(expected)')
  elif mod=='syntax':s=s.replace('expected.name()', 'expected_name(*expected)')
 if mod=='composition':s=s.replace('item.name()', 'composition_name(item)')
 if mod=='ids':s=s.replace('item.of.name()','owns_name(item.of)').replace('item.why.name()','why_name(item.why)')
 if mod=='repository':s=s.replace('item.command.text()', 'command_text(item.command)').replace('item.command.name()', 'command_name(item.command)').replace('lines.put(oid)', 'lines.put(&oid.0)').replace('b"oid", oid','b"oid", &oid.0')
 if mod=='reserved':s=s.replace('item.rule.name()', 'rule_name(item.rule)')
 if mod=='schema':
  s=s.replace('lines.put(first);','lines.put(first.key());').replace('lines.put(second);','lines.put(second.key());')
  s=s.replace('b"first", first','b"first", first.key()').replace('b"second", second','b"second", second.key()')
 if mod=='syntax':
  s=s.replace('Arity { filter, expected, given }','Arity { filter, given }')
  s=s.replace('lines.quote(filter)', 'lines.quote(filter_name(*filter))').replace('u64::from(*expected)', 'u64::from(filter.arity())').replace('*expected == 1','filter.arity() == 1').replace('b"filter", filter','b"filter", filter_name(*filter)')
 p.write_text(s)
p=R/'errors/validation.rs';s=p.read_text().replace('Evaluation','FaultKind');s='use hearth_domain_render_vocabulary::{evaluate::FaultKind, errors::Required};\n'+s;p.write_text(s)
# Move the existing store printer to the vocabulary, excluding the client's additional Layers limit.
s=(R/'progress.rs').read_text();a=s.index('#[derive',s.index('impl Strays'));b=s.index('#[derive',s.index('impl StoreError'))
store=convert(s[a:b]);store='use hearth_domain_render_vocabulary::errors::{StoreError, StoreFault, StoreLimit, Step};\nuse super::{Lines, Encoder, field};\n'+store
store=re.sub(r'^\s*StoreLimit::Layers =>[^\n]*\n','\n',store,flags=re.M).replace('step.name()', 'step_name(step)')
(R/'errors/store.rs').write_text(store)
# Keep the store's extended full answer as its own client record, not another render error.
full='''/// A full store can reach its layer limit (hearth/docs/design/client/README.md, 3.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StoreFull {
    pub store: Box<[u8]>,
    pub limit: StoreLimit,
    pub most: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreLimit {
    Resolutions,
    Layers,
    CacheFiles,
}
impl StoreFull {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"the store ");
        lines.put(&self.store);
        lines.put(b" is full: ");
        lines.decimal(u64::from(self.most));
        lines.put(match self.limit {
            StoreLimit::Resolutions => b" resolutions",
            StoreLimit::Layers => b" layers",
            StoreLimit::CacheFiles => b" cache files",
        });
        lines.put(b"\\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"error");
        field(json, b"kind", b"store");
        field(json, b"store", &self.store);
        field(json, b"limit", match self.limit {
            StoreLimit::Resolutions => b"resolutions",
            StoreLimit::Layers => b"layers",
            StoreLimit::CacheFiles => b"cache_files",
        });
        json.key(b"most");
        json.unsigned(u64::from(self.most));
        field(json, b"class", b"operational");
    }
}

'''
(R/'progress.rs').write_text(s[:a]+full+s[b:])
# Move filter's projection, unmodified.
builder=Path('hearth/crates/hearth-protocol-builder/src/translate/errors.rs');s=builder.read_text();a=s.index('fn filter_name');b=end(s,s.index('{',a));filter_fn=s[a:b].replace('render::evaluate::Filter','hearth_domain_render_vocabulary::evaluate::Filter').replace('    use hearth_domain_render_vocabulary::evaluate::Filter;\n','')
p=R/'errors/syntax.rs';p.write_text(p.read_text()+'\n'+filter_fn+'\n')
# Retain only configuration and template inputs in the builder.
keep=s[:s.index('#[must_use]\npub fn error')]+s[s.index('#[must_use]\npub fn declared('):s.index('#[must_use]\npub fn declared_to_records')]+s[s.index('#[must_use]\npub fn syntax('):s.index('#[must_use]\npub fn syntax_to_records')]
keep=keep.replace('mod ids;\n','').replace('mod reserved;\n','').replace('mod collisions;\n','').replace('    config, records,','    config,').replace('Exhaustive config faults and client error records','Exhaustive config and template faults')
builder.write_text(keep)
