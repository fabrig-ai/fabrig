import json,sys,pathlib,re,collections
from rewrite import types,close
changes=collections.defaultdict(set)
def recvstart(s,end):
 i=end
 while i>0:
  c=s[i-1]
  if c.isalnum() or c in '_:.?':i-=1
  elif c in ')]':
   want={')':'(',']':'['}[c];depth=1;i-=1
   while i>0 and depth:
    i-=1
    if s[i]==c:depth+=1
    elif s[i]==want:depth-=1
  elif c.isspace() and i<len(s) and s[i]=='.':i-=1
  else:break
 return i
for line in open(sys.argv[1]):
 try:d=json.loads(line)
 except ValueError:continue
 if d.get('reason')!='compiler-message':continue
 m=d['message'];msg=m['message'];spans=m['spans']
 primary=[x for x in spans if x['is_primary']]
 if not primary:continue
 x=primary[0];p=pathlib.Path(x['file_name'])
 if not p.exists() or 'generated' in p.parts:continue
 s=p.read_text();a=x['byte_start'];b=x['byte_end']
 # compiler bytes are UTF8, translate
 a=len(s.encode()[:a].decode());b=len(s.encode()[:b].decode())
 def add(a,b,n):changes[p].add((a,b,n))
 owner=('temper-client-protocol-wire/src/' in str(p)) or ('temper_client_protocol_wire::' in msg) or ('settings::Agent' in msg) or (('`&w::' in msg or '`&v1::' in msg or '`&forge::' in msg) and 'temper_client_protocol_wire' in s) or any('temper-client-protocol-wire/src/generated/' in x['file_name'] for x in spans)
 if 'is not an iterator' in msg and (t:=re.match(r'`([^`]+)` is not an iterator',msg)):
  meth=s[a:b];typ=t[1].split('::')[-1]
  if meth in types.get(typ,{}):msg=f'no method named `{meth}` found for struct `{t[1]}`'
 if (z:=re.match(r'no method named `(\w+)` found for (?:struct|reference|mutable reference) `([^`]+)`',msg)) and owner:
  meth=z[1];typ=z[2].split('::')[-1];typ=re.sub('[^a-zA-Z0-9_]','',typ)
  if meth in ['measure', 'encode', 'write', 'read', 'decode'] and (meth == 'measure' or meth not in types.get(typ,{})):
   if 'use crate::v1::Record;' not in s and 'use temper_client_protocol_wire::v1::Record;' not in s:
    pos=0
    for line in s.splitlines(keepends=True):
     if line.startswith('//!') or not line.strip():pos+=len(line)
     else:break
    pos=(re.search(r'^use ',s,re.M).start() if re.search(r'^use ',s,re.M) else len(s))
    pos=(re.search(r'^use ',s,re.M).start() if re.search(r'^use ',s,re.M) else len(s))
    add(pos,pos,('use crate::v1::Record;\n' if 'temper-client-protocol-wire/src/' in str(p) else 'use temper_client_protocol_wire::v1::Record;\n'))
   continue
  if meth=='into_parts':add(a-1,close(s,b)+1,'');continue
  if meth not in types.get(typ,{}):continue
  end=close(s,b)+1;start=recvstart(s,a-1);expr=s[start:a-1];borrow=types[typ][meth].startswith('&')
  # chained methods auto-borrow fields; a preceding dereference cancels the old getter's reference
  if borrow and start>0 and s[start-1]=='*':start-=1;borrow=False
  if s[end:].lstrip().startswith('.'):borrow=False
  add(start,end,('&' if borrow else '')+expr+'.'+meth)
 elif 'items from traits can only be used' in (m.get('rendered') or '') and owner:
  if 'use temper_client_protocol_wire::v1::Record;' not in s and 'use crate::v1::Record;' not in s and 'use v1::Record;' not in s:
   # any family's Record is the same trait
   lines=s.splitlines(keepends=True);pos=0
   for line in lines:
    if line.startswith('//!') or not line.strip():pos+=len(line)
    else:break
   pos=(re.search(r'^use ',s,re.M).start() if re.search(r'^use ',s,re.M) else len(s))
   add(pos,pos,('use crate::v1::Record;\n' if 'temper-client-protocol-wire/src/' in str(p) else 'use temper_client_protocol_wire::v1::Record;\n'))
 elif ('/tests' in str(p) or '/testing/' in str(p)) and (m.get('code') or {}).get('code')=='E0061' and s[a:b]=='measure' and s[b:].startswith('()'):
  family='public_v1' if 'public_v1::Limits' in (m.get('rendered') or '') or p.stem=='public_wire' else 'v1'
  add(a,b+2,'measure(&temper_client_protocol_wire::'+family+'::CEILINGS).expect("within ceilings")')
 elif ('/tests' in str(p) or '/testing/' in str(p)) and (m.get('code') or {}).get('code')=='E0308' and s[:a].endswith('.encode(') and s[a:b].startswith('&mut '):
  family='public_v1' if 'public_v1::Limits' in (m.get('rendered') or '') or p.stem=='public_wire' else 'v1'
  add(a-7,a,'write(&temper_client_protocol_wire::'+family+'::CEILINGS, ')
 elif (m.get('code') or {}).get('code')=='unfulfilled_lint_expectations':
  start=s.rfind('#[expect(',0,a+1);end=s.find(')]',b)+2
  if start>=0 and end>start and len(re.findall(r'clippy::\w+',s[start:end]))==1:
   if s[end:end+1]=='\n':end+=1
   add(start,end,'')
 elif (m.get('code') or {}).get('code')=='E0507' and s[b:].startswith('.into()'):
  add(a,b+7,s[a:b]+'.clone()')
 elif (m.get('code') or {}).get('code')=='clippy::redundant_locals':
  start=s.rfind('\n',0,a)+1;end=s.find(';',a)+1
  if re.fullmatch(r'\s*let (\w+) = \1;',s[start:end]):add(start,end,'')
 elif msg=='expected outer doc comment':
  pass
for l in open(sys.argv[1]):
 try:d=json.loads(l)
 except ValueError:continue
 if d.get('reason')!='compiler-message':continue
 m=d['message'];code=(m.get('code') or {}).get('code','');msg=m['message']
 def walk(m):
  ss=m.get('spans',[])
  if ss and all(s.get('suggestion_applicability')=='MachineApplicable' for s in ss):
   for s in ss:
    p=pathlib.Path(s['file_name'])
    if 'generated' not in p.parts and p.exists():changes[p].add((len(p.read_bytes()[:s['byte_start']].decode()),len(p.read_bytes()[:s['byte_end']].decode()),s['suggested_replacement']))
  for c in m.get('children',[]):walk(c)
 if code in ['E0308','clippy::needless_question_mark','clippy::useless_conversion','clippy::op_ref','unused_qualifications','unused_imports','clippy::needless_borrow','clippy::redundant_locals','clippy::let_and_return','clippy::redundant_field_names','unused_braces','unused_parens','clippy::unnecessary_wraps']:walk(m)
 if code=='unused_variables' and any('/tests' in sp['file_name'] or '/testing/' in sp['file_name'] for sp in m['spans']):
  # underscore unused parameters; remove pure local limit bindings later
  for sp in m['spans']:
   if not sp['is_primary']:continue
   p=pathlib.Path(sp['file_name']);s=p.read_text();a=len(p.read_bytes()[:sp['byte_start']].decode());b=len(p.read_bytes()[:sp['byte_end']].decode())
   if s[a:b] in ['limits','bounds','l','wire_limits']:
    if s[:a].rstrip().endswith('let'):
     start=s.rfind('\n',0,a)+1;end=s.find(';',a)+1;changes[p].add((start,end,''))
    else:changes[p].add((a,b,'_'+s[a:b]))

for p,edits in changes.items():
 s=p.read_text();last=len(s)+1
 for a,b,n in sorted(edits,reverse=True):
  if b>last:continue
  s=s[:a]+n+s[b:];last=a
 backup=pathlib.Path('.scratch-client/backups')/str(p);backup.parent.mkdir(parents=True,exist_ok=True);backup.write_text(p.read_text())
 p.write_text(s)
print('edited',len(changes),'files;',sum(map(len,changes.values())),'edits')
