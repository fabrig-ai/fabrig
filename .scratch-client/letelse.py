import pathlib,re,sys
from rewrite import close
for arg in sys.argv[1:]:
 p=pathlib.Path(arg)
 for p in p.rglob('*.rs') if p.is_dir() else [p]:
  s=p.read_text();edits=[]
  for m in re.finditer(r'let (?:Ok|Some)\([^\n=]*\) =\s*(\{)',s):
   a=m.start(1);b=close(s,a)+1
   if s[b:].lstrip().startswith('else'):edits.append((a,b,'('+s[a:b]+')'))
  for a,b,n in sorted(edits,reverse=True):s=s[:a]+n+s[b:]
  if edits:p.write_text(s)
