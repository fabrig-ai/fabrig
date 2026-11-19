import pathlib,re,sys
from rewrite import close,types
for arg in sys.argv[1:]:
 p=pathlib.Path(arg)
 for p in p.rglob('*.rs') if p.is_dir() else [p]:
  s=p.read_text();edits=[]
  for m in re.finditer(r'\bw::(\w+)::into\(',s):
   if 'into' not in types.get(m[1],{}):continue
   b=close(s,m.end()-1)+1;expr=s[m.end():b-1].strip().removeprefix('&');n='('+expr+').into'
   if not s[b:].lstrip().startswith('.'):n='&'+n
   edits.append((m.start(),b,n))
  for a,b,n in sorted(edits,reverse=True):s=s[:a]+n+s[b:]
  if edits:p.write_text(s)
