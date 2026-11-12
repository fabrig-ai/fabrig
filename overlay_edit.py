from pathlib import Path
root=Path('temper/crates/temper-settings-form/src')
fields={'rest':'bool','model':'SettingModel','alternatives':'List<SettingModel>','inspect':'bool','modify':'bool','shell':'bool','agents':'bool','tools':'u64','hosts':'SettingHosts','waiting':'Duration','resume':'bool','turns':'u32','time':'Duration','reserve_turns':'u32','reserve_time':'Duration','call_timeout':'Duration','spend':'u64','documents':'List<CharterDocument>'}
def change(p,old,new):
 s=p.read_text(); assert old in s,(p,old); p.write_text(s.replace(old,new))
p=root/'view.rs'
s='''/// Literal text and optional replacements (client/ctl/overlays.md, 1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Overlay {
'''+''.join(f'    pub {f}: Option<{t}>,\n' for f,t in fields.items())+'''    pub instructions: Text,
    pub parts: List<CharterPart>,
    /// Unresolved directory names; empty after assembly.
    pub charters: List<Box<[u8]>>,
}
'''
change(p,'/// The store names in bit order',s+'/// The store names in bit order')
change(p,'pub struct SettingDocument {','pub struct SettingDocument {\n    pub overlay: Option<Box<Overlay>>,')
change(p,'pub enum SettingFollows {','pub enum SettingFollows {\n    Overlay,')
change(p,'pub enum SettingValue {','pub enum SettingValue {\n    Overlay(Box<Overlay>),')
change(Path('temper/crates/temper-settings-form/Cargo.toml'),'skein-lib.workspace = true','temper-charter-set.workspace = true\nskein-lib.workspace = true')
# overlay decoder uses existing member forms
p=root/'settings_file_values.rs'
s='''pub(super) fn overlay(d: &Decoder<'_>, at: u32, number: u32) -> Result<c::Overlay, Error> {
    let fields = d.object(at, &['''+','.join('b"'+x+'"' for x in [*fields,'instructions','parts','charters'])+'''])?;
    if optional(&fields, b"charters").is_some() && (optional(&fields, b"instructions").is_some() || optional(&fields, b"parts").is_some()) {
        return Err(error(b"overlay", b"charters or instructions and parts, not both"));
    }
    let mut charters = List::uncapped();
    if let Some(at) = optional(&fields, b"charters") {
        if number != 5 && number != 2 { return Err(error(b"overlay.charters", b"sets are charters 5 and 2's")); }
        let positions = d.array(at, d.doc.len(), b"overlay.charters")?;
        if positions.is_empty() { return Err(error(b"overlay.charters", b"nonempty list required")); }
        for at in &positions {
            let name = d.string(*at)?;
            if name.is_empty() { return Err(error(b"overlay.charters", b"directory must not be empty")); }
            charters.push(name.into()).expect("bounded names");
        }
    }
    let mut parts = charter_parts(d, optional(&fields, b"parts"))?;
    if optional(&fields, b"resume").is_some_and(|at| d.boolean(at).unwrap_or(false)) && !parts.is_empty() {
        return Err(error(b"overlay.parts", b"a charter that resumes takes no parts"));
    }
    Ok(c::Overlay {
        charters,
        instructions: match optional(&fields, b"instructions") {
            Some(at) => d.words(at, d.limits.charter_instructions, b"overlay.instructions")?,
            None => c::Text::new(Box::from(b"".as_slice()), d.limits.charter_instructions).expect("empty text"),
        },
        parts: core::mem::replace(&mut parts, List::with_capacity(0)),
'''
for f,t in fields.items():
 if t=='bool': expr='d.boolean(at)?'
 elif t=='u64': expr='d.number(at)?'
 elif t=='u32': expr=f'u32::try_from(d.number(at)?).or(Err(error(b"overlay.{f}", b"integer bound")))?'
 elif t=='Duration': expr='Duration::from_nanos(d.number(at)?)'
 elif f=='model': expr='setting_model(d, at)?'
 elif f=='hosts': expr='setting_hosts(d, at)?'
 elif f=='documents': expr='documents(d, at)?'
 elif f=='alternatives': expr='''{ let positions = d.array(at, d.limits.charter_alternatives, b"overlay.alternatives")?;
 let mut values = List::with_capacity(positions.len());
 for at in &positions { values.push(setting_model(d, *at)?).expect("bounded alternatives"); } values }'''
 s+=f'        {f}: match optional(&fields, b"{f}") {{ Some(at) => Some({expr}), None => None }},\n'
s+='    })\n}\n'
change(p,'pub(super) fn charter(',s+'pub(super) fn charter(')
p=root/'file.rs'
change(p,'let fields = d.object(at, &[b"setting", b"set", b"edit"])?;','let fields = d.object(at, &[b"setting", b"set", b"edit", b"overlay"])?;')
change(p,'    let edit = match (optional(&fields, b"set"), optional(&fields, b"edit")) {','''    if let Some(at) = optional(&fields, b"overlay") {
        if optional(&fields, b"set").is_some() || optional(&fields, b"edit").is_some() {
            return Err(error(b"item", b"exactly one of set, edit or overlay required"));
        }
        let c::Setting::Charter(n) = setting else { return Err(error(b"overlay", b"charter required")); };
        let overlay = values::overlay(d, at, n.number)?;
        return Ok(c::SettingItem { setting, edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Overlay(Box::new(overlay)) }) });
    }
    let edit = match (optional(&fields, b"set"), optional(&fields, b"edit")) {''')
# encode overlay with existing value member encoders
p=root/'values.rs'
s='pub fn encode_overlay(value: &c::Overlay, out: &mut Encoder) {\n    out.object_start();\n'
for f,t in fields.items():
 s+=f'    if let Some(value) = &value.{f} {{\n        out.key(b"{f}");\n'
 if t=='bool': expr='out.boolean(*value);'
 elif t in ['u64','u32']: expr='out.unsigned(u64::from(*value));' if t=='u32' else 'out.unsigned(*value);'
 elif t=='Duration': expr='out.unsigned(value.as_nanos());'
 elif f=='model': expr='encode_setting_model(value, out);'
 elif f=='hosts': expr='encode_setting_hosts(value, out);'
 else:
  enc='encode_document' if f=='documents' else 'encode_setting_model'
  expr=f'out.array_start(); for value in value {{ {enc}(value, out); }} out.array_end();'
 s+='        '+expr+'\n    }\n'
s+='''    if !value.instructions.bytes().is_empty() { out.key(b"instructions"); encode_text(&value.instructions, out); }
    if !value.parts.is_empty() { out.key(b"parts"); encode_parts(&value.parts, out); }
    out.object_end();
}
fn encode_parts(parts: &skein_lib::List<c::CharterPart>, out: &mut Encoder) {
    out.array_start();
    for part in parts {
        out.object_start(); out.key(b"on"); out.array_start();
        for (index, name) in c::REASON_NAMES.iter().enumerate() {
            let bit = 1_u32.checked_shl(u32::try_from(index).expect("reason index")).expect("reason bit");
            if part.on & bit != 0 { out.string(name); }
        }
        out.array_end(); out.key(b"instructions"); encode_text(&part.instructions, out); out.object_end();
    }
    out.array_end();
}
'''
change(p,'pub fn encode_charter(',s+'pub fn encode_charter(')
# adapter auto-conversion of options
p=Path('temper/crates/temper-client-tools/src/jig/settings_adapter.rs')
s=''
for prefix,src,dst in [('form','c','n'),('client','n','c')]:
 s+=f'pub fn {prefix}_overlay(v: &{src}::Overlay) -> {dst}::Overlay {{\n    {dst}::Overlay {{\n'
 if prefix=='form': s+='        charters: List::with_capacity(0),\n'
 for f,t in fields.items():
  if f in ['model','hosts']: expr=f'match &v.{f} {{ Some(v) => Some({prefix}_setting_{f}(v)), None => None }}'
  elif f in ['alternatives','documents']:
   cv=f'{prefix}_setting_model(item)' if f=='alternatives' else f'{dst}::CharterDocument {{ connector: item.connector, name: item.name.clone(), path: item.path.clone(), heading: item.heading.clone() }}'
   expr=f'match &v.{f} {{ Some(v) => {{ let mut out = List::with_capacity(v.len()); for item in v {{ out.push({cv}).expect("same count"); }} Some(out) }}, None => None }}'
  else: expr=f'v.{f}'
  s+=f'        {f}: {expr},\n'
 s+=f'''        instructions: {prefix}_text(&v.instructions),
        parts: {{ let mut out = List::with_capacity(v.parts.len()); for p in &v.parts {{ out.push({dst}::CharterPart {{ on: p.on, instructions: {prefix}_text(&p.instructions) }}).expect("same count"); }} out }},
    }}
}}
'''
change(p,'pub fn form_charter(',s+'pub fn form_charter(')
change(p,'overlay: None,','overlay: v.overlay.as_ref().map(|v| Box::new(form_overlay(v))),')
change(p,'c::SettingFollows::Default =>','c::SettingFollows::Overlay => n::SettingFollows::Overlay,\n        c::SettingFollows::Default =>')
change(p,'c::SettingValue::Model(x) =>','c::SettingValue::Overlay(x) => n::SettingValue::Overlay(Box::new(form_overlay(x))),\n        c::SettingValue::Model(x) =>')
change(p,'n::SettingValue::Model(x) =>','n::SettingValue::Overlay(x) => c::SettingValue::Overlay(Box::new(client_overlay(x))),\n        n::SettingValue::Model(x) =>')
# neutral wire conversions
p=root/'wire.rs'
s=''
for direction in ['restore','wire']:
 restoring=direction=='restore'; src='w' if restoring else 'c'; dst='c' if restoring else 'w'; lim='Limits' if restoring else 'w::Limits'
 s+=f'pub fn {direction}_overlay(value: &{src}::Overlay, limits: &{lim}) -> Result<{dst}::Overlay, w::Problem> {{\n'
 s+=f'    let mut parts = List::with_capacity(value.parts{ "()" if restoring else ""}.len());\n'
 s+=f'    for item in {"value.parts()" if restoring else "&value.parts"} {{\n'
 expr='c::CharterPart { on: item.on(), instructions: settings_text(item.instructions(), w::CEILINGS.charter_part_instructions)? }' if restoring else 'w::CharterPart::new(limits, w::CharterPartParts { on: item.on, instructions: bytes::copy_of(item.instructions.bytes()) })?'
 s+=f'        parts.push({expr}).expect("same count");\n    }}\n'
 s+= '    Ok(c::Overlay {\n        charters: List::with_capacity(0),\n' if restoring else '    w::Overlay::new(limits, w::OverlayParts {\n'
 s+='        parts,\n        instructions: '+('settings_text(value.instructions(), w::CEILINGS.charter_instructions)?' if restoring else 'bytes::copy_of(value.instructions.bytes())')+',\n'
 for f,t in fields.items():
  access=f'value.{f}()' if restoring else f'&value.{f}'
  if f in ['model','hosts']:
   fun=f'settings_{direction}_setting_{f}'
   expr=f'match {access} {{ Some(v) => Some({fun}(v, limits){"?" if f=="model" else ""}), None => None }}'
  elif f in ['alternatives','documents']:
   if f=='alternatives': conv=f'settings_{direction}_setting_model(item, limits)?'
   elif restoring: conv='c::CharterDocument { connector: item.connector(), name: item.name().clone(), path: item.path().into(), heading: item.heading().clone() }'
   else: conv='w::CharterDocument::new(limits, w::CharterDocumentParts { connector: item.connector, name: item.name.clone(), path: item.path.clone(), heading: item.heading.clone() })?'
   expr=f'match {access} {{ Some(v) => {{ let mut out = List::with_capacity(v.len()); for item in v {{ out.push({conv}).expect("same count"); }} Some(out) }}, None => None }}'
  else: expr=f'*value.{f}()' if restoring else f'value.{f}'
  s+=f'        {f}: {expr},\n'
 s+='    })\n}\n'
change(p,'pub(crate) fn settings_restore_charter(',s+'pub(crate) fn settings_restore_charter(')
change(p,'w::SettingValue::Charter(item) =>','w::SettingValue::Overlay(item) => c::SettingValue::Overlay(Box::new(restore_overlay(item, limits)?)),\n        w::SettingValue::Charter(item) =>')
change(p,'c::SettingValue::Charter(item) =>','c::SettingValue::Overlay(item) => w::SettingValue::Overlay(Box::new(wire_overlay(item, limits)?)),\n        c::SettingValue::Charter(item) =>')
change(p,'        setting: settings_restore_setting(value.setting(), limits),\n        revision: value.revision(),\n        value:', '        overlay: match value.overlay() { Some(v) => Some(Box::new(restore_overlay(v, limits)?)), None => None },\n        setting: settings_restore_setting(value.setting(), limits),\n        revision: value.revision(),\n        value:')
change(p,'w::SettingFollows::Default =>','w::SettingFollows::Overlay => c::SettingFollows::Overlay,\n        w::SettingFollows::Default =>')
p=root/'proposal.rs'; change(p,'view::SettingValue::Model(_)','view::SettingValue::Overlay(_)\n        | view::SettingValue::Model(_)')
p=root/'render.rs'
change(p,'c::SettingValue::Model(v) =>','c::SettingValue::Overlay(v) => values::encode_overlay(v, out),\n        c::SettingValue::Model(v) =>')
change(p,'c::SettingValue::Model(_)','c::SettingValue::Overlay(_)\n        | c::SettingValue::Model(_)')
change(p,'            if let Some(state) = &document.application {','            if let Some(overlay) = &document.overlay { out.key(b"overlay"); values::encode_overlay(overlay, out); }\n            if let Some(state) = &document.application {')
change(p,'c::SettingFollows::Default => {','c::SettingFollows::Overlay => out.put(b"overlay"),\n            c::SettingFollows::Default => {')
change(p,'c::SettingFollows::Default => b"default",','c::SettingFollows::Overlay => b"overlay",\n            c::SettingFollows::Default => b"default",')
p=Path('temper/crates/temper-client-tools/src/jig/render/document/settings.rs')
change(p,'c::SettingFollows::Default => {','c::SettingFollows::Overlay => out.put(b"overlay"),\n            c::SettingFollows::Default => {')
