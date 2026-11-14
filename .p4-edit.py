from pathlib import Path
import re

def calls(s, pattern, suffix='?'):
    matches=list(re.finditer(pattern+r'\(',s))
    for m in reversed(matches):
        start=m.end(); level=1; i=start
        # All these calls' strings have balanced parentheses.
        while level:
            if s[i]=='(': level+=1
            elif s[i]==')': level-=1
            i+=1
        s=s[:i]+suffix+s[i:]
    return s

def methods(path,names,pattern):
    p=Path(path);s=p.read_text()
    for name in names.split():
        m=re.search(r'(?m)^    (?:pub )?fn '+name+r"(?:<'a>)?\(",s);assert m,name
        end=s.index('\n    }',m.end())
        body=s[m.start():end]
        sigend=body.index(' {')
        body=body[:sigend]+' -> Result<(), String>'+body[sigend:]
        body=calls(body,pattern)
        body=body.replace('return;', 'return Ok(());')
        body+='\n        Ok(())'
        s=s[:m.start()]+body+s[end:]
    p.write_text(s)

methods('jig/testing/jig-conformance/src/core.rs','observe inbound cold submitted durable fresh_mark durable_at delivered completion assigned call_answer reply_receipts',r'self\.(?:observe|durable_at|reply_receipts)')
p=Path('jig/testing/jig-conformance/src/core.rs');s=p.read_text()
s=s.replace('self.referee.observe(now, observed).unwrap_or_else(|error| panic!("referee {error:?} at {now}"));','self.referee.observe(now, observed).map_err(|error| format!("referee {error:?} at {now}"))?;')
s=s.replace('''assert!(
                                pending.writes.contains(&Write::Erase(key.clone())),
                                "read marker erases with its owner"
                            );''','''if !pending.writes.contains(&Write::Erase(key.clone())) {
                                return Err("read marker erases with its owner".into());
                            }''')
s=s.replace('assert_eq!(kept, value, "immutable activity row changed");','if kept != value {\n                        return Err("immutable activity row changed".into());\n                    }')
s=s.replace('assert_eq!(task, answered, "party\'s words answered for another task");','if task != answered {\n                            return Err("party\'s words answered for another task".into());\n                        }')
s=s.replace('''assert!(
                matches!(store.rows.get(&key), Some(core::Record::Tasks(tasks::Stored::Activity(row))) if row == kept),
                "immutable activity row disappeared"
            );''','''if !matches!(store.rows.get(&key), Some(core::Record::Tasks(tasks::Stored::Activity(row))) if row == kept) {
                return Err("immutable activity row disappeared".into());
            }''')
s=s.replace('''assert!(
            !writes || serial == 0 || self.settled_calls.contains(&(task, attempt, serial)),
            "answer preceded its durable named-call record"
        );''','''if writes && serial != 0 && !self.settled_calls.contains(&(task, attempt, serial)) {
            return Err("answer preceded its durable named-call record".into());
        }''')
s=s.replace('assert!(self.applied >= fence.after, "Seen reply preceded its durable position/forget commit");','if self.applied < fence.after {\n                            return Err("Seen reply preceded its durable position/forget commit".into());\n                        }')
p.write_text(s)
methods('jig/testing/jig-core-kit/src/conformance.rs','systems workers inbound submitted durable fresh_mark cold delivered',r'self\.(?:observe|inner\.(?:inbound|completion|submitted|durable|fresh_mark|cold|assigned|delivered|call_answer))')
