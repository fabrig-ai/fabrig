#!/usr/bin/env python3
"""Independent-process lifetime tests and injected heavy completion adapter."""
import base64
import importlib.util
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('transport_fixture', Path(__file__).with_name('transport_fixture.py'))
f = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(f)
s, d, w = f.supervisor, f.durable, f.worker


def receipt(intent, *, released=True):
    log = b'fake heavy sealed log\n'
    task = {'token': 'e'*32, 'id': '17', 'queue': 'q', 'label': intent['label'],
            'repo': intent['cwd'], 'args': intent['payload'], 'commit': intent['candidate'],
            'tree': 'a'*40, 'state': 'released' if released else 'active'}
    if released:
        task['receipt'] = {k: task[k] for k in ('token', 'id', 'queue', 'args', 'tree')}
        task['receipt'].update(schema='heavy-receipt-v1', code=0, release_complete=True,
            cgroup_empty=True, row_durable=True, log_sha256=w.digest(log))
        task['log_base64'] = base64.b64encode(log).decode()
    return {'schema': 'heavy-completion-query-v1', 'queue': 'q', 'label': intent['label'], 'matches': [task]}


class SupervisorTests(f.EndpointFixture, unittest.TestCase):
    def pid_exit(self, pid):
        fd = os.pidfd_open(pid)
        self.addCleanup(os.close, fd)
        return fd

    def test_controller_loss_does_not_stop_worker_and_checkout_remains_owned(self):
        ready, go = self.root / 'ready', self.root / 'go'
        os.mkfifo(ready, 0o600)
        os.mkfifo(go, 0o600)
        ready_fd, go_fd = os.open(ready, os.O_RDWR), os.open(go, os.O_RDWR)
        self.addCleanup(os.close, ready_fd)
        self.addCleanup(os.close, go_fd)
        script = f'import os\nwith open({str(ready)!r},"w") as f: f.write(str(os.getpid())+"\\n"); f.flush()\nwith open({str(go)!r}) as f: f.read(1)\n'
        request, bundle = self.request(tools=script)
        client_ready_r, client_ready_w = os.pipe()
        hold_r, hold_w = os.pipe()
        code = '''import importlib.util, os, sys
spec=importlib.util.spec_from_file_location('t',sys.argv[1]); t=importlib.util.module_from_spec(spec); spec.loader.exec_module(t)
e=t.durable.read_json(t.durable.read_bytes(sys.argv[2])); req=t.durable.read_json(t.durable.read_bytes(sys.argv[3])); bundle=t.durable.read_json(t.durable.read_bytes(sys.argv[4]))
t.Transport(e,timeout=10).start(req,bundle)
os.write(int(sys.argv[5]),b'accepted'); os.read(int(sys.argv[6]),1)
'''
        endpoints, reqfile, bundlefile = [self.root / name for name in ('endpoints.json','req.json','bundle.json')]
        for path, value in ((endpoints,self.endpoints),(reqfile,request),(bundlefile,bundle)):
            d.publish(path,d.encoded(value),immutable=True)
        child = subprocess.Popen(['python3','-I','-B','-c',code,str(f.DIRECTORY/'transport.py'),
            str(endpoints),str(reqfile),str(bundlefile),str(client_ready_w),str(hold_r)],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            pass_fds=(client_ready_w,hold_r))
        self.addCleanup(child.kill if child.poll() is None else lambda: None)
        os.close(client_ready_w); os.close(hold_r)
        self.addCleanup(os.close, client_ready_r); self.addCleanup(os.close, hold_w)
        self.assertTrue(select.select([client_ready_r],[],[],10)[0])
        self.assertEqual(os.read(client_ready_r,32),b'accepted')
        self.assertTrue(select.select([ready_fd],[],[],10)[0])
        command_pid = int(os.read(ready_fd,64).strip())
        command_fd = self.pid_exit(command_pid)
        child.kill(); child.communicate(timeout=5)
        self.assertEqual(self.real_transport.status(request)['state'],'running')
        with self.assertRaises(RuntimeError):
            with f.mq.authority.checkout_lock(self.workers[request['gate']]):
                self.fail('controller death released physical worker')
        os.write(go_fd,b'x')
        self.assertTrue(select.select([command_fd],[],[],10)[0])
        self.assertEqual(self.finish_real(request)['state'],'terminal')

    def test_supervisor_loss_remains_unknown_after_command_exit_and_same_boot_refuses_repair(self):
        ready, go = self.root/'ready', self.root/'go'
        os.mkfifo(ready,0o600); os.mkfifo(go,0o600)
        ready_fd,go_fd=os.open(ready,os.O_RDWR),os.open(go,os.O_RDWR)
        self.addCleanup(os.close,ready_fd); self.addCleanup(os.close,go_fd)
        script=f'import os\nwith open({str(ready)!r},"w") as f: f.write(str(os.getpid())+"\\n"); f.flush()\nwith open({str(go)!r}) as f: f.read(1)\n'
        request,bundle=self.request(tools=script)
        self.real_transport.start(request,bundle)
        self.assertTrue(select.select([ready_fd],[],[],10)[0])
        command_pid=int(os.read(ready_fd,64).strip())
        command_fd=self.pid_exit(command_pid)
        identity=d.read_json(d.read_bytes(self.job(request)/'started.json'))
        supervisor_fd=self.pid_exit(identity['pid'])
        os.kill(identity['pid'],signal.SIGKILL)
        self.assertTrue(select.select([supervisor_fd],[],[],10)[0])
        self.assertEqual(self.real_transport.status(request)['state'],'unknown')
        os.write(go_fd,b'x')
        self.assertTrue(select.select([command_fd],[],[],10)[0])
        self.assertEqual(self.real_transport.status(request)['state'],'unknown')
        config=self.configs[request['gate']]
        with self.assertRaisesRegex(RuntimeError,'same boot'):
            s.reconcile_reboot(config,request)
        with self.assertRaises(RuntimeError):
            with f.mq.authority.checkout_lock(self.workers[request['gate']]):
                self.fail('unproved job released')
        # Model a separately approved dedicated-host reboot. This is injected
        # boot evidence, never a claim that this test rebooted any machine.
        with patch.object(s,'boot',return_value='new-fixture-boot'):
            s.reconcile_reboot(config,request)
            reply,blobs=s.status(config,request)
        self.assertEqual(reply['state'],'reconciled')
        self.assertEqual(d.read_json(blobs[0])['reconciled'],True)
        self.assertTrue((self.job(request)/'reconciliation.json').exists())
        self.assertFalse((self.workers[request['gate']]/'.git'/s.MARKER).exists())

    def command_world(self):
        request,bundle=self.request()
        config=self.configs[request['gate']]
        s.accept(config,request,bundle,d.checked(bundle['path'],bundle),launch=lambda *_:None)
        job,claim=s.read_claim(config,request)
        return config,job,claim

    def test_nonempty_heavy_command_persists_exact_admitted_identity_receipt_and_log(self):
        config,job,claim=self.command_world()
        executable=self.root/'fake-heavy'
        executable.write_text('#!/bin/sh\nprintf "fake ordinary payload\\n"\n')
        executable.chmod(0o700)
        config=dict(config,heavy=str(executable))
        def query(_,label):
            intent=d.read_json(d.read_bytes(job/'command-0.intent'))
            self.assertEqual(intent['label'],label)
            return receipt(intent)
        commands=s.Commands(config,claim,job,query=query)
        with (job/'output').open('w') as output:
            result=commands.run(['heavy','--gate','python3','-c','print(1)'],cwd=Path(config['repo']),
                stdin=subprocess.DEVNULL,stdout=output,stderr=subprocess.STDOUT)
        self.assertEqual(result.returncode,0)
        saved=d.read_json(d.read_bytes(job/'command-0.heavy'))
        self.assertEqual(saved['matches'][0]['token'],'e'*32)
        self.assertEqual(commands.evidence()[0]['done']['heavy'],saved)
        self.assertIn('fake ordinary payload',(job/'output').read_text())

    def test_unresolved_heavy_and_admission_ambiguity_never_complete(self):
        config,job,claim=self.command_world()
        intent={'index':0,'command':['heavy','--gate','python3','-c','pass'],
                'cwd':config['repo'],'candidate':claim['request']['candidate'],'heavy':True,
                'label':'mq-'+claim['job']+'-0','payload':['python3','-c','pass']}
        d.publish(job/'command-0.intent',d.encoded(intent),immutable=True)
        # A disappeared supervisor plus a released flock is insufficient even
        # with a new boot: heavy's supported query must prove all matching jobs.
        for evidence in (receipt(intent,released=False),dict(receipt(intent),matches=[receipt(intent)['matches'][0]]*2)):
            with patch.object(s,'boot',return_value='new-fixture-boot'), patch.object(s,'heavy_query',return_value=evidence):
                with self.assertRaises(ValueError):
                    s.reconcile_reboot(config,claim['request'])
            self.assertTrue((Path(config['repo'])/'.git'/s.MARKER).exists())
        with patch.object(s,'boot',return_value='new-fixture-boot'), patch.object(s,'heavy_query',return_value=receipt(intent)):
            s.reconcile_reboot(config,claim['request'])
        self.assertFalse((Path(config['repo'])/'.git'/s.MARKER).exists())

    def test_lost_worker_with_live_fake_heavy_requires_released_task_evidence(self):
        config,job,claim=self.command_world()
        ready_r,ready_w=os.pipe(); go_r,go_w=os.pipe()
        for fd in (ready_r,ready_w,go_r,go_w): self.addCleanup(os.close,fd)
        executable=self.root/'fake-heavy'
        executable.write_text('#!/usr/bin/env python3\nimport os\nos.write('+str(ready_w)+',str(os.getpid()).encode()); os.read('+str(go_r)+',1)\n')
        executable.chmod(0o700)
        custom=dict(config,heavy=str(executable))
        path=self.root/'fake-command-config.json'
        d.publish(path,d.encoded(custom),immutable=True)
        helper="""import importlib.util,sys,pathlib,subprocess
spec=importlib.util.spec_from_file_location('s',sys.argv[1]);s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
c=s.durable.read_json(s.durable.read_bytes(sys.argv[2]));job=pathlib.Path(sys.argv[3]);claim=s.durable.read_json(s.durable.read_bytes(job/'claim.json'))
repo=pathlib.Path(c['repo'])
with s.authority.Lock(repo.with_name(repo.name+'.queue.lock')):
 s.subreaper();s.durable.publish(job/'started.json',s.durable.encoded(s.process_identity()),immutable=True)
 with (job/'output').open('w') as out:
  s.Commands(c,claim,job).run(['heavy','--gate','python3','-c','pass'],cwd=repo,stdin=subprocess.DEVNULL,stdout=out,stderr=subprocess.STDOUT,pass_fds=(int(sys.argv[4]),int(sys.argv[5])))
"""
        child=subprocess.Popen(['python3','-I','-B','-c',helper,str(f.DIRECTORY/'supervisor.py'),
            str(path),str(job),str(ready_w),str(go_r)],stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,stderr=subprocess.PIPE,pass_fds=(ready_w,go_r))
        self.assertTrue(select.select([ready_r],[],[],10)[0])
        payload_pid=int(os.read(ready_r,64));payload_fd=self.pid_exit(payload_pid)
        child.kill();child.communicate(timeout=5)
        self.assertFalse(select.select([payload_fd],[],[],0)[0])
        self.assertEqual(s.status(config,claim['request'])[0]['state'],'unknown')
        intent=d.read_json(d.read_bytes(job/'command-0.intent'))
        with patch.object(s,'boot',return_value='new-fixture-boot'),patch.object(s,'heavy_query',return_value=receipt(intent,released=False)):
            with self.assertRaises(ValueError):s.reconcile_reboot(config,claim['request'])
        self.assertTrue((Path(config['repo'])/'.git'/s.MARKER).exists())
        os.write(go_w,b'x')
        self.assertTrue(select.select([payload_fd],[],[],10)[0])
        self.assertFalse((job/'command-0.done').exists())
        # Client/payload exit still does not supply heavy's containment receipt.
        self.assertEqual(s.status(config,claim['request'])[0]['state'],'unknown')
        with patch.object(s,'boot',return_value='new-fixture-boot'),patch.object(s,'heavy_query',return_value=receipt(intent)):
            s.reconcile_reboot(config,claim['request'])
        self.assertFalse((Path(config['repo'])/'.git'/s.MARKER).exists())

    def test_tools_timeout_remains_red_after_group_reap(self):
        config,job,claim=self.command_world()
        commands=s.Commands(config,claim,job)
        with (job/'output').open('w') as output, self.assertRaises(subprocess.TimeoutExpired):
            commands.run(['python3','-I','-B','-c','import signal; signal.pause()'],
                cwd=Path(config['repo']),stdin=subprocess.DEVNULL,stdout=output,
                stderr=subprocess.STDOUT,timeout=0.01)
        self.assertEqual(commands.evidence()[0]['done']['returncode'],124)

    def test_subreaper_waits_for_orphan_grandchild_after_direct_command_exits(self):
        config,job,claim=self.command_world()
        ready_r,ready_w=os.pipe(); go_r,go_w=os.pipe()
        for fd in (ready_r,ready_w,go_r,go_w): self.addCleanup(os.close,fd)
        childcode='import os\nif os.fork()==0:\n os.write('+str(ready_w)+',b"orphan"); os.read('+str(go_r)+',1); os._exit(0)\nos._exit(0)'
        helper='''import importlib.util,sys,pathlib,subprocess
spec=importlib.util.spec_from_file_location('s',sys.argv[1]);s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
c=s.durable.read_json(s.durable.read_bytes(sys.argv[2])); job=pathlib.Path(sys.argv[3]); claim=s.durable.read_json(s.durable.read_bytes(job/'claim.json'))
s.subreaper(); commands=s.Commands(c,claim,job)
with (job/'output').open('w') as out:
 commands.run(['python3','-I','-B','-c',sys.argv[4]],cwd=pathlib.Path(c['repo']),stdin=subprocess.DEVNULL,stdout=out,stderr=subprocess.STDOUT,pass_fds=(int(sys.argv[5]),int(sys.argv[6])))
'''
        child=subprocess.Popen(['python3','-I','-B','-c',helper,str(f.DIRECTORY/'supervisor.py'),
            self.endpoints[config['gate']]['config'],str(job),childcode,str(ready_w),str(go_r)],
            stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,pass_fds=(ready_w,go_r))
        self.assertTrue(select.select([ready_r],[],[],10)[0])
        self.assertEqual(os.read(ready_r,64),b'orphan')
        self.assertFalse((job/'command-0.done').exists())
        self.assertIsNone(child.poll())
        os.write(go_w,b'x')
        out,err=child.communicate(timeout=10)
        self.assertEqual(child.returncode,0,(out,err))
        self.assertTrue((job/'command-0.done').exists())


if __name__ == '__main__':
    unittest.main()
