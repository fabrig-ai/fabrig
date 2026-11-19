import json,sys,pathlib,collections
changes=collections.defaultdict(set)
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
    if 'generated' not in p.parts and p.exists():changes[p].add((s['byte_start'],s['byte_end'],s['suggested_replacement']))
  for c in m.get('children',[]):walk(c)
 if code in ['unused_qualifications','unused_imports','clippy::needless_borrow','clippy::redundant_locals','clippy::let_and_return','clippy::redundant_field_names','unused_braces','unused_parens','clippy::unnecessary_wraps']:walk(m)
 if code=='unused_variables':
  # underscore unused parameters; remove pure local limit bindings later
  for sp in m['spans']:
   if not sp['is_primary']:continue
   p=pathlib.Path(sp['file_name']);s=p.read_text();a=sp['byte_start'];b=sp['byte_end']
   if s[a:b] in ['limits','bounds']:
    if s[:a].rstrip().endswith('let'):
     start=s.rfind('\n',0,a)+1;end=s.find(';',b)+1;changes[p].add((start,end,''))
    else:changes[p].add((a,b,'_'+s[a:b]))
for p,edits in changes.items():
 s=p.read_bytes();last=len(s)+1
 for a,b,n in sorted(edits,reverse=True):
  if b>last:continue
  s=s[:a]+n.encode()+s[b:];last=a
 p.write_bytes(s)
print('suggestions',sum(map(len,changes.values())))
