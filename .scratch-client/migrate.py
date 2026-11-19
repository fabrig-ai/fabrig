import re, pathlib, subprocess, json, sys
ROOT=pathlib.Path('.')
base='temper/crates/temper-client-protocol-wire/src/generated/'
meta={}
for f in ['client_wire.rs','public.rs']:
 s=subprocess.check_output(['git','show','HEAD:'+base+f],text=True)
 types={}
 for m in re.finditer(r'impl (\w+) \{(.*?)(?=\nimpl |\n(?:#\[|pub (?:struct|enum))|\Z)',s,re.S):
  types.setdefault(m[1],{}).update({x[1]:x[2] for x in re.finditer(r'pub (?:const )?fn (\w+)\(&self\) -> ([^\n{]+) \{',m[2])})
 # explicit constants can map via normalized spelling
 constants=re.findall(r'pub const (\w+): Path',pathlib.Path(base+f).read_text())
 paths={p:next((c for c in constants if c.replace('_','').lower()==p.replace('TemperLimits','LimitOverrides').lower()),None) for p in re.findall(r'Self::(\w+) => "',s)}
 meta[f]={'types':types,'paths':paths}
pathlib.Path('.scratch-client/meta.json').write_text(json.dumps(meta))
