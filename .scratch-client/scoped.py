import pathlib,re,sys
from rewrite import rewrite,close
for arg in sys.argv[1:]:
 p=pathlib.Path(arg)
 for p in p.rglob('*.rs') if p.is_dir() else [p]:
  s=p.read_text();edits=[]
  for m in re.finditer(r'\bfn \w+\s*\(',s):
   sig=close(s,m.end()-1);a=s.find('{',sig)
   if a<0:continue
   b=close(s,a)+1;body=s[a:b]
   if re.search(r'use temper_client_protocol_wire::.*\bv1 as w\b',body):
    n=rewrite(body,['w'])
    if n!=body:edits.append((a,b,n))
  last=len(s)+1
  for a,b,n in sorted(edits,reverse=True):
   if b>last:continue
   s=s[:a]+n+s[b:];last=a
  if edits:p.write_text(s)
