#!/usr/bin/env python3
"""Stop/drain/restart/retirement using disposable Git and real authority."""
from contextlib import contextmanager
import importlib.util
import os
from pathlib import Path
import select
import subprocess
import sys
import time
import types
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('transport_fixture', Path(__file__).with_name('transport_fixture.py'))
f = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(f)
op,d,w=f.operation,f.durable,f.worker


class OperationTests(f.fixtures.TrainFixture,unittest.TestCase):
    def setUp(self):
        super().setUp()
        self.layout=op.authority.Layout(self.root/'shared')
        self.lease=self.layout.acquire()
        self.addCleanup(self.lease.close)
        self.effect_repo=self.root/'authority clone'
        f.mq.git(self.root,'clone',str(self.origin),str(self.effect_repo))
        self.calls=[]
        def runner(argv,**kwargs):
            self.calls.append(argv)
            if argv[0]=='post': return subprocess.CompletedProcess(argv,0,'','')
            return subprocess.run(argv,**kwargs)
        self.authority=op.authority.GitAuthority(self.layout,self.lease,self.effect_repo,str(self.origin),
            {'ops':'ops@fixture'},runner=runner)
        self.batch=2
        self.store=f.train.StateStore(self.layout.train)
        self.controller=self.reload()
        self.controller.state['last_full']=self.clock
        self.controller.save()
        self.config={'authority_dir':str(self.layout.root),'stop':str(self.root/'queue.stop')}
        d.publish(self.layout.root/'train-operation.json',d.encoded(self.config),immutable=True)

    def test_stop_zero_safe_retirement_is_audited_and_generation_cannot_revive(self):
        op.stop(self.config); self.controller.stop()
        op.drain(self.controller,self.config,0)
        audit=op.retire(self.controller,self.config)
        self.assertTrue((self.layout.root/'retired.json').exists())
        self.assertEqual(audit['state'],d.fingerprint(d.read_bytes(self.store.path)))
        self.layout.legacy_guard(self.effect_repo)
        with self.assertRaisesRegex(RuntimeError,'retired'):
            self.reload()
        self.assertTrue(self.store.path.exists())
        self.assertTrue((self.layout.root/'train-owner.json').exists())

    def test_stop_one_admitted_job_finishes_without_replacement(self):
        self.branch('a'); attempt,record=self.admit()
        op.stop(self.config); self.controller.stop()
        self.assertIsNone(self.controller.admit())
        self.assertFalse(self.controller.drained())
        self.transport.finish(attempt)
        op.cycle(self.controller,self.config)
        self.assertTrue(self.controller.drained())
        self.assertEqual(self.authority.main(),record['candidate'])
        self.assertEqual(self.transport.starts,[attempt])

    def test_stop_two_adjudicates_exact_ordered_p_c_and_drains_both(self):
        self.branch('a'); a,ra=self.admit()
        self.branch('b'); b,rb=self.admit()
        self.assertEqual(rb['parent'],ra['candidate'])
        self.transport.finish(b)
        op.stop(self.config); self.controller.stop()
        op.cycle(self.controller,self.config)
        self.assertFalse(self.controller.drained())
        self.assertEqual(self.authority.main(),self.base)
        self.transport.finish(a)
        op.cycle(self.controller,self.config)
        pushes=[argv[-1] for argv in self.calls if argv[:2]==['git','push'] and argv[-1].endswith(':refs/heads/main')]
        self.assertEqual(pushes,[ra['candidate']+':refs/heads/main',rb['candidate']+':refs/heads/main'])
        self.assertTrue(self.controller.drained())
        self.assertEqual(self.transport.starts,[a,b])
        self.assertFalse(any('--force' in arg for argv in self.calls if argv[-1].endswith(':refs/heads/main') for arg in argv))

    def test_stop_preserves_bisection_and_drains_invalidated_successor(self):
        self.set_batch(2)
        self.branch('a',{'core/src/lib.rs':'a'}); self.branch('b',{'consumer/src/lib.rs':'b'})
        a,ra=self.admit()
        self.branch('c'); b,rb=self.admit()
        self.transport.finish(b)
        self.transport.finish(a,code=1)
        op.stop(self.config); self.controller.stop()
        op.cycle(self.controller,self.config)
        self.assertTrue(rb.get('invalidated'))
        self.assertTrue(self.controller.drained())
        self.assertEqual(len(self.controller.state['pending']),2)
        self.assertEqual(self.authority.main(),self.base)
        self.assertEqual(self.transport.starts,[a,b])
        restored=self.reload()
        self.assertEqual(restored.state['pending'],self.controller.state['pending'])
        self.assertIsNone(restored.admit())
        with self.assertRaisesRegex(RuntimeError,'retirement refused'):
            op.retire(restored,self.config)

    def test_unsafe_legacy_return_and_modified_retirement_snapshot_refused(self):
        with self.assertRaises(RuntimeError): self.layout.legacy_guard(self.effect_repo)
        self.branch('a'); self.admit(); self.controller.stop()
        with self.assertRaises(RuntimeError): op.retire(self.controller,self.config)
        # A separate empty authority demonstrates that post-retirement edits
        # cannot make an audit authorize a different snapshot.
        self.transport.finish(self.controller.state['active'][0])
        op.cycle(self.controller,self.config,admit=False)
        op.retire(self.controller,self.config)
        self.controller.state['idle_since']+=1
        self.controller.save()
        with self.assertRaises(RuntimeError): self.layout.legacy_guard(self.effect_repo)

    def test_restart_timeout_and_unknown_do_not_launch_or_update(self):
        self.branch('a'); self.admit()
        @contextmanager
        def opened(*args,**kwargs): yield self.controller,self.lease
        helper=types.SimpleNamespace(queue_pid_for_lock=lambda _:None)
        self.config['output']=str(self.root/'output')
        self.lease.close()
        # The test controller uses a separate guard lease while restart obtains
        # its serialization lock. No subprocess/main update is mocked as safe.
        self.lease=self.layout.acquire()
        self.authority.lock=self.lease
        for unknown in (False,True):
            self.transport.fail_status=unknown
            with patch.object(op,'configuration',return_value=self.config),patch.object(op,'open_controller',opened),\
                 patch.object(op,'module',return_value=helper),patch.object(op,'launch') as start:
                with self.assertRaises((RuntimeError,TimeoutError)):
                    op.restart(self.root/'config',wait=0)
                start.assert_not_called()
            self.assertTrue(Path(self.config['stop']).exists())
        self.assertFalse(any(argv[:3]==['git','fetch','origin'] for argv in self.calls))

    def test_expired_drain_does_not_poll_or_adjudicate_admitted_work(self):
        self.branch('a'); attempt,record=self.admit()
        with patch.object(self.transport,'status') as status:
            with self.assertRaises(TimeoutError):
                op.drain(self.controller,self.config,0)
            status.assert_not_called()
        self.assertFalse(record['applied'])
        self.assertIsNone(record['outcome'])
        self.assertTrue(Path(self.config['stop']).exists())

    def test_prepared_reload_dispatches_existing_attempt_without_new_admission(self):
        self.branch('prepared')
        with patch.object(self.controller,'dispatch',side_effect=RuntimeError('controller lost before dispatch')):
            with self.assertRaisesRegex(RuntimeError,'controller lost'):
                self.controller.admit()
        attempt=self.controller.state['active'][0]
        record=self.controller.state['records'][attempt]
        self.assertEqual(record['phase'],'prepared')
        restored=self.reload()
        marker=self.prep/'.git/mq-preparation-unresolved.json'
        w.atomic_json(marker,{'attempt':attempt,'generation':restored.config['generation'],'state':str(self.store.path)})
        op.stop(self.config)
        op.cycle(restored,self.config,admit=False)
        self.assertFalse(marker.exists())
        self.assertEqual(self.transport.starts,[attempt])
        self.assertEqual(restored.state['active'],[attempt])
        self.assertEqual(restored.state['serial'],1)
        self.assertEqual(self.transport.jobs[attempt]['request'],record['request'])

    def test_expired_drain_does_not_launch_git_or_post_effect_clients(self):
        endpoint=types.SimpleNamespace(deadline=0)
        with patch.object(op.subprocess,'run') as run, self.assertRaises(TimeoutError):
            op.effect_run(endpoint,['git','ls-remote','fixture'])
        run.assert_not_called()

    def test_error_episode_delivers_once_and_recurrence_gets_new_stable_id(self):
        op.episode(self.controller,RuntimeError('same failure'))
        first=d.read_json(d.read_bytes(self.layout.root/'error-episode.json'))['errors']['same failure']['id']
        op.episode(self.controller,RuntimeError('same failure'))
        self.assertEqual(len([argv for argv in self.calls if argv[0]=='post']),1)
        op.completed_cycle(self.controller)
        op.episode(self.controller,RuntimeError('same failure'))
        second=d.read_json(d.read_bytes(self.layout.root/'error-episode.json'))['errors']['same failure']['id']
        self.assertNotEqual(first,second)
        self.assertEqual(len([argv for argv in self.calls if argv[0]=='post']),2)

    def test_resume_requires_proven_drain_then_keeps_frozen_batch_and_pending(self):
        self.branch('a'); attempt,_=self.admit()
        op.stop(self.config); self.controller.stop()
        with self.assertRaises(RuntimeError): op.resume(self.controller,self.config)
        self.transport.finish(attempt)
        op.resume(self.controller,self.config)
        self.assertFalse(self.controller.state['stopping'])
        self.assertFalse(Path(self.config['stop']).exists())
        self.assertEqual(self.controller.config['batch'],self.batch)


class ExecutableTests(f.EndpointFixture,unittest.TestCase):
    def config_file(self):
        # Keep the executable test's real floor positive. This changes only
        # disposable fixture configuration, never a production default.
        for gate,config in self.configs.items():
            config['disk_floor']=1
            e=self.endpoints[gate]
            Path(e['config']).write_bytes(d.encoded(config))
            e['config_sha256']=w.digest(d.encoded(config))
        authority_repo=self.root/'authority clone'
        f.mq.git(self.root,'clone',str(self.origin),str(authority_repo))
        self.config={'version':1,'authority_dir':str(self.root/'shared'),
            'authority_repo':str(authority_repo),'origin':str(self.origin),'leads':{'ops':'ops@fixture'},
            'preparation':str(self.prep),'artifacts':str(self.artifacts),'generation':'generation-operation',
            'batch':16,'full_every':5,'disk_floor':1,'endpoints':self.endpoints,'rpc_timeout':10,
            'stop':str(self.root/'queue.stop'),'output':str(self.root/'queue.out')}
        path=self.root/'train config.json'
        d.publish(path,d.encoded(self.config),immutable=True)
        self.bin=self.root/'bin';self.bin.mkdir(mode=0o700)
        post=self.bin/'post';post.write_text('#!/bin/sh\nexit 0\n');post.chmod(0o700)
        self.env=dict(os.environ,PATH=str(self.bin)+':'+os.environ['PATH'])
        return path

    def test_explicit_cli_once_crash_recovery_and_ordered_authority(self):
        path=self.config_file()
        with op.open_controller(self.config) as (controller,_):
            controller.state['last_full']=time.time();controller.save()
        self.branch('actual-cli')
        result=subprocess.run(['python3','-I','-B',str(f.DIRECTORY/'mq'),'once','--train-config',str(path)],
            stdin=subprocess.DEVNULL,capture_output=True,env=self.env,timeout=10)
        self.assertEqual(result.returncode,0,result.stderr)
        snapshot=f.train.StateStore(Path(self.config['authority_dir'])/'train').load()
        attempt=snapshot['active'][0];request=snapshot['records'][attempt]['request']
        f.wait_file(self.job(request)/'completion.json')
        self.assertEqual(f.mq.git(self.origin,'rev-parse','main').stdout.strip(),self.base)
        with patch.dict(os.environ,{'PATH':self.env['PATH']}),op.open_controller(self.config) as (controller,_):
            self.assertTrue(controller.unreconciled_gates)
            op.cycle(controller,self.config,admit=False)
            self.assertEqual(controller.unreconciled_gates,set())
            self.assertEqual(controller.authority.main(),request['candidate'])
            self.assertTrue(controller.drained())
            record=controller.state['records'][attempt]
            self.assertEqual(d.fingerprint(d.read_bytes(Path(self.artifacts)/attempt/'supervision.json')),record['completion_evidence']['receipt'])
        status=subprocess.run(['python3','-I','-B',str(f.DIRECTORY/'mq'),'train-status','--train-config',str(path)],
            stdin=subprocess.DEVNULL,capture_output=True,env=self.env,timeout=5)
        self.assertEqual(status.returncode,0,status.stderr)
        self.assertEqual(d.read_json(status.stdout)['main'],'unknown')

    def test_restart_acknowledges_same_authority_and_preserves_train_batch16(self):
        path=self.config_file()
        with op.open_controller(self.config) as (controller,_):
            controller.state['last_full']=time.time();controller.save()
        children=[]
        real_popen=subprocess.Popen
        def launch(*args,**kwargs):
            child=real_popen(*args,**kwargs);children.append(child);return child
        with patch.dict(os.environ,{'PATH':self.env['PATH']}),patch.object(op,'launch',side_effect=launch):
            op.restart(path,wait=2)
        self.assertEqual(len(children),1)
        child=children[0]
        fd=os.pidfd_open(child.pid)
        try:
            # No unlocked gap: replacement holds the same authority inode.
            with self.assertRaises(RuntimeError):
                with op.authority.Layout(self.config['authority_dir']).acquire(): self.fail('authority gap')
            snapshot=f.train.StateStore(Path(self.config['authority_dir'])/'train').load()
            self.assertEqual(snapshot['config']['batch'],16)
            op.stop(self.config)
            self.assertTrue(select.select([fd],[],[],10)[0],Path(self.config['output']).read_text())
            self.assertEqual(child.wait(timeout=2),0,Path(self.config['output']).read_text())
        finally:
            os.close(fd)
            if child.poll() is None: child.kill();child.wait()
        self.assertTrue(Path(self.config['stop']).exists())

    def test_restart_output_failure_restores_stop_without_launch(self):
        path=self.config_file()
        self.config['output']=str(self.root/'missing-parent/queue.out')
        path.write_bytes(d.encoded(self.config))
        with op.open_controller(self.config) as (controller,_):
            controller.state['last_full']=time.time();controller.save()
        with patch.object(op,'launch') as launch, self.assertRaises(FileNotFoundError):
            op.restart(path,wait=2)
        launch.assert_not_called()
        self.assertTrue(Path(self.config['stop']).exists())

    def test_unknown_configuration_and_default_legacy_remain_explicit(self):
        path=self.config_file()
        result=subprocess.run(['python3','-I','-B',str(f.DIRECTORY/'mq'),'plan','--train-config',str(path)],
            stdin=subprocess.DEVNULL,capture_output=True,timeout=5)
        self.assertEqual(result.returncode,1)
        self.assertIn(b'unsupported train action',result.stderr)
        self.assertFalse(Path(self.config['authority_dir']).exists())
        self.config['version']=2
        path.write_bytes(d.encoded(self.config))
        with self.assertRaises(ValueError): op.configuration(path)
        result=subprocess.run(['python3','-I','-B',str(f.DIRECTORY/'mq'),'train-status'],
            stdin=subprocess.DEVNULL,capture_output=True,timeout=5)
        self.assertEqual(result.returncode,2)
        self.assertIn(b'explicit --train-config',result.stderr)


if __name__=='__main__':
    unittest.main()
