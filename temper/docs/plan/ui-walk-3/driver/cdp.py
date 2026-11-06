"""Small synchronous CDP connection, using only Python's standard library."""
import base64
import hashlib
import json
import os
import socket
import struct
import time
import urllib.parse


class CDP:
    def __init__(self, address):
        url = urllib.parse.urlsplit(address)
        self.sock = socket.create_connection((url.hostname, url.port), timeout=30)
        key = base64.b64encode(os.urandom(16)).decode()
        self.sock.sendall((f'GET {url.path} HTTP/1.1\r\nHost: {url.netloc}\r\n'
                           f'Upgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\n'
                           'Sec-WebSocket-Version: 13\r\n\r\n').encode())
        response = b''
        while not response.endswith(b'\r\n\r\n'):
            response += self.read(1)
        expected = base64.b64encode(hashlib.sha1((key + '258EAFA5-E914-47DA-95CA-C5AB0DC85B11').encode()).digest())
        if b' 101 ' not in response or expected not in response:
            raise RuntimeError('CDP websocket handshake refused')
        self.serial = 0

    def read(self, n):
        result = b''
        while len(result) < n:
            part = self.sock.recv(n - len(result))
            if not part:
                raise EOFError('Chromium closed CDP')
            result += part
        return result

    def send(self, method, params=None):
        self.serial += 1
        body = json.dumps({'id': self.serial, 'method': method, 'params': params or {}}).encode()
        mask = os.urandom(4)
        n = len(body)
        prefix = bytes([0x81, 0x80 | n]) if n < 126 else bytes([0x81, 0xfe]) + struct.pack('!H', n) if n < 65536 else bytes([0x81, 0xff]) + struct.pack('!Q', n)
        self.sock.sendall(prefix + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(body)))
        return self.serial

    def call(self, method, **params):
        wanted = self.send(method, params)
        fragments = b''
        while True:
            first, second = self.read(2)
            n = second & 127
            if n == 126:
                n = struct.unpack('!H', self.read(2))[0]
            elif n == 127:
                n = struct.unpack('!Q', self.read(8))[0]
            payload = self.read(n)
            if first & 15 == 8:
                raise EOFError('Chromium closed websocket')
            fragments += payload
            if not first & 128:
                continue
            message = json.loads(fragments)
            fragments = b''
            if message.get('method') == 'Page.javascriptDialogOpening':
                self.send('Page.handleJavaScriptDialog', {'accept': True, 'promptText': getattr(self, 'prompt', '')})
            if message.get('id') == wanted:
                if 'error' in message:
                    raise RuntimeError(message['error'])
                return message.get('result', {})

    def js(self, expression):
        self.call('Page.bringToFront')
        result = self.call('Runtime.evaluate', expression=expression, returnByValue=True, awaitPromise=True)
        if 'exceptionDetails' in result:
            raise RuntimeError(result['exceptionDetails'])
        return result.get('result', {}).get('value')

    def go(self, url):
        self.call('Page.navigate', url=url)
        time.sleep(0.5)

    def text(self):
        return self.js("document.body?.innerText || ''")

    def wait(self, words, seconds=40):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            if words.casefold() in (self.text() or '').casefold():
                return
            time.sleep(0.2)
        raise RuntimeError(f'Page did not show {words!r}: {self.text()}')

    def ready(self, name, seconds=120):
        end=time.monotonic()+seconds
        while time.monotonic()<end:
            if any(x['name'].strip()==name and not x.get('disabled') for x in self.controls()):
                return
            time.sleep(.2)
        raise RuntimeError(f'Control stayed disabled: {name}')

    def controls(self):
        return self.js("""[...document.querySelectorAll('a,button,input,textarea,select,summary,[role=button],[role=menuitem]')]
          .filter(e=>e.checkVisibility({checkVisibilityCSS:true}))
          .map(e=>({tag:e.tagName,role:e.getAttribute('role'),name:e.getAttribute('aria-label')||e.innerText||e.labels?.[0]?.innerText||e.placeholder||'',type:e.type,disabled:e.disabled}))""")

    def click(self, name, role=None, occurrence=0):
        # Resolve by accessible role and name, then dispatch a real mouse press.
        expression = """(()=>{const name=NAME,role=ROLE;
          const roles={A:'link',BUTTON:'button',SUMMARY:'button',TEXTAREA:'textbox',SELECT:'combobox'};
          const root=document.querySelector('dialog[open]')||document; const els=[...root.querySelectorAll('a,button,summary,input,textarea,select,[role]')].filter(e=>{
            const r=e.getAttribute('role')||roles[e.tagName]||(e.type==='checkbox'?'checkbox':e.type==='radio'?'radio':e.tagName==='INPUT'?'textbox':'');
            const n=(e.getAttribute('aria-label')||e.innerText||e.labels?.[0]?.innerText||e.placeholder||'').trim().replace(/\\s+/g,' ');
            return (!role||r===role)&&n===name&&!e.disabled&&e.checkVisibility({checkVisibilityCSS:true});
          });const e=els[OCC];if(!e)return null;e.scrollIntoView({block:'center'});const b=e.getBoundingClientRect();return {x:b.x+b.width/2,y:b.y+b.height/2,count:els.length};})()""".replace('NAME', json.dumps(name)).replace('ROLE', json.dumps(role)).replace('OCC', str(occurrence))
        deadline=time.monotonic()+10
        while True:
            target=self.js(expression)
            if target:break
            if time.monotonic()>=deadline:
                raise RuntimeError(f'No visible {role or "control"} {name!r}; {self.controls()}')
            time.sleep(.1)
        for kind in ['mousePressed', 'mouseReleased']:
            self.call('Input.dispatchMouseEvent', type=kind, x=target['x'], y=target['y'], button='left', clickCount=1)
        time.sleep(0.25)

    def fill(self, name, value):
        found = self.js("""(()=>{const e=[...document.querySelectorAll('textarea,input')].find(e=>e.checkVisibility({checkVisibilityCSS:true})&&(e.getAttribute('aria-label')===NAME||e.placeholder===NAME||[...e.labels||[]].some(l=>l.innerText.trim()===NAME)));if(!e)return false;e.focus();e.select();return true})()""".replace('NAME', json.dumps(name)))
        if not found:
            raise RuntimeError(f'No field {name!r}: {self.controls()}')
        self.call('Input.insertText', text=value)

    def shot(self, path, reset=True):
        if reset:
            self.js('window.scrollTo(0,0)')
        time.sleep(0.2)
        path.write_bytes(base64.b64decode(self.call('Page.captureScreenshot', format='png', captureBeyondViewport=False, fromSurface=False)['data']))
