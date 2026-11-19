import pathlib,re,subprocess,sys
from rewrite import rewrite
allfiles=[]
for p in pathlib.Path('temper').rglob('*.rs'):
 if 'generated' in p.parts:continue
 s=p.read_text()
 if 'temper_client_protocol_wire' not in s:continue
 prefixes=['temper_client_protocol_wire::v1','temper_client_protocol_wire::public_v1']
 aliases=[]
 for m in re.finditer(r'use temper_client_protocol_wire::(v1|public_v1)(?: as (\w+))?;',s):aliases.append(m[2] or m[1])
 for m in re.finditer(r'use temper_client_protocol_wire::\{([^}]+)\};',s,re.S):
  aliases+=re.findall(r'(?:v1|public_v1) as (\w+)',m[1])
  if re.search(r'\bv1\b(?! as)',m[1]):aliases.append('v1')
 for a in aliases:
  # A competing import shadows this alias only in its own scope: leave it to compiler passes.
  if re.search(r'use (?:jig|temper_protocol_store|temper_protocol_worker|skein)[\w:]* (?:as )?'+a+r';',s):continue
  prefixes.append(a)
 new=rewrite(s,list(set(prefixes)))
 if new!=s:p.write_text(new);allfiles.append(str(p))
print('bulk constructor migration',len(allfiles),'files')
pathlib.Path('.scratch-client/bulk-files').write_text('\n'.join(allfiles))
