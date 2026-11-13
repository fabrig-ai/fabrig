from pathlib import Path
import re
s=Path('.split-clippy.log').read_text()
fixes={}
for block in re.split(r'(?=^error)',s,flags=re.M):
 if 'could have a `#[must_use]`' in block:
  m=re.search(r'--> ([^:]+):(\d+):',block)
  if m:fixes.setdefault(m[1],set()).add(int(m[2])-1)
for path,lines in fixes.items():
 p=Path(path);s=p.read_text().splitlines(True)
 for n in sorted(lines,reverse=True):s.insert(n,re.match(r'\s*',s[n])[0]+'#[must_use]\n')
 p.write_text(''.join(s))
print('must_use fixes:',sum(map(len,fixes.values())))
