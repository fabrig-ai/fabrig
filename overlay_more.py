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
