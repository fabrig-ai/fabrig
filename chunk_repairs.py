from pathlib import Path
import json,re,sys
paths=set()
for line in open(sys.argv[1]):
 r=json.loads(line)
 if r.get('reason')!='compiler-message':continue
 m=r['message']; c=(m.get('code')or{}).get('code','')
 if c=='clippy::chunks_exact_to_as_chunks':paths.update(s['file_name'] for s in m['spans'] if s['is_primary'])
for name in paths:
 p=Path(name);t=p.read_text()
 def change(m):
  mutable=m[1] or '';n=m[2];tail=t[m.end():].lstrip()
  return '.as_chunks'+mutable+'::<'+n+'>().0'+(('.iter_mut()' if mutable else '.iter()') if tail.startswith('.') else '')
 t=re.sub(r'\.chunks_exact(_mut)?\((\d+)\)',change,t)
 p.write_text(t)
print('chunk sites in',len(paths),'files')
