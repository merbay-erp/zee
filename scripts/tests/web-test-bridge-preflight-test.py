"""K305 deterministic lifecycle preparation; no real child, signal delivery or socket."""
import argparse
from contextlib import ExitStack
import importlib.util
import json
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

FILES = Path(__file__).resolve().parents[2]
TOOL = FILES / 'scripts/tests/web-test-bridge-test.py'
CASES = ['all_or_none', 'suite_exact_hash_before_units', 'suite_fifo_no_open',
         'duplicate_role_rejected', 'unit_failure_never_native', 'operator_python_role_no_shell']


class Contract(unittest.TestCase):
    def exercise(self, fault=None, number=signal.SIGTERM):
        # Import the actual main, then replace only its external owner/OS boundaries.
        spec = importlib.util.spec_from_file_location('bridge_under_test', TOOL)
        tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(tool)
        with tempfile.TemporaryDirectory(prefix='zee-bridge-prep-') as directory:
            root = Path(directory).resolve()
            suite = root / 'compiler/tests/fixtures/owned-web-kit-suite.json'
            suite.parent.mkdir(parents=True)
            suite.write_text('{"mock":true}\n')
            out = root / 'output'
            events, children, stops = [], [], []
            state = {'spawned': 0, 'triggered': False, 'missing_handler': False,
                     'now': 0.0, 'read': 0, 'cleanup': False, 'stop_fault': False}
            handlers = {signal.SIGTERM: signal.SIG_DFL, signal.SIGINT: signal.SIG_DFL}
            installed = {}
            actual_write = Path.write_text

            def event(name):
                events.append(name)
                if name == fault and not state['triggered']:
                    state['triggered'] = True
                    handler = installed.get(number)
                    if handler is None:
                        state['missing_handler'] = True
                    else:
                        # Callback delivery only: never os.kill or an actual child.
                        handler(number, None)

            def set_handler(sig, handler):
                previous = handlers[sig]
                handlers[sig] = handler
                if callable(handler):
                    installed[sig] = handler
                    events.append('handler_installed')
                else:
                    event('handler_restored')
                    if fault == 'restore_failure' and sig == signal.SIGTERM:
                        raise RuntimeError('mock restore failure')
                return previous

            class Owner:
                def __init__(self, args, log):
                    self.children = children
                    self.args = args
                    event('owner_init')

                def spawn(self, argv, capture=False):
                    assert capture and self.args.stop_grace == 35
                    state['spawned'] += 1
                    child = SimpleNamespace(stdout=SimpleNamespace(), status=2)
                    children.append(child)  # Retain the original fake owner object first.
                    if '--test-output' in argv:
                        target = Path(argv[argv.index('--test-output') + 1])
                        if target.name == 'failed-unit':
                            target.mkdir()
                            (target / 'unit.json').write_text(json.dumps({'passed': False, 'native_started': False}))
                            child.status = 1
                    event('spawn')
                    return child

                def exit_status(self, child):
                    assert child in children
                    if fault == 'terminal_timeout':
                        state['now'] = 11.0
                    return child.status

                def stop_child(self, child):
                    assert child in children
                    stops.append((child, state['cleanup']))
                    event('child_cleanup')
                    if fault == 'stop_failure' and not state['stop_fault']:
                        state['stop_fault'] = True
                        raise RuntimeError('mock stop failure')
                    if fault == 'cleanup_remaining':
                        raise RuntimeError('mock retained child')
                    children.remove(child)

                def cleanup(self):
                    state['cleanup'] = True
                    event('owner_cleanup')
                    for child in list(children):
                        self.stop_child(child)
                    if fault == 'cleanup_failure':
                        raise RuntimeError('mock cleanup failure after closure')

            class Selector:
                def __init__(self):
                    if fault == 'selector_constructor':
                        raise RuntimeError('mock selector constructor')
                    self.keys = {}

                def register(self, obj, flags):
                    if fault == 'selector_register':
                        raise RuntimeError('mock selector register')
                    self.keys[1] = SimpleNamespace(fileobj=obj, fd=-123)

                def get_map(self):
                    if fault == 'timeout':
                        state['now'] = 11.0
                    return self.keys

                def select(self, timeout):
                    assert timeout == .05
                    event('wait')
                    return [(key, 1) for key in self.keys.values()]

                def unregister(self, obj):
                    self.keys.clear()

                def close(self):
                    events.append('selector_close')
                    if fault == 'selector_close':
                        raise RuntimeError('mock selector close')

            def read(fd, size):
                assert fd == -123 and size == 4096
                state['read'] += 1
                return b'x' * 65537 if fault == 'overflow' else b''

            def write(path, text, *args, **kwargs):
                result = actual_write(path, text, *args, **kwargs)
                if path == out / 'proof.json':
                    event('proof_write')
                    if installed and json.loads(text)['status'] == 'passed' and not state['triggered']:
                        assert handlers == installed, 'flag handlers must remain active during publication'
                return result

            def fake_spec(name, path):
                assert name == 'bridge_owner' and Path(path) == root / 'scripts/worker-supervisor.py'
                loader = SimpleNamespace(exec_module=lambda module: setattr(module, 'Supervisor', Owner))
                return SimpleNamespace(loader=loader)

            real_check = getattr(tool, 'check', None)

            def check():
                if 'proof_write' in events:
                    event('final_check')
                return real_check()

            argv = [str(TOOL), '--framework-root', str(root), '--dil', str(root / 'never-execute-dil'),
                    '--postgres-bin-dir', str(root / 'never-use-pg'), '--output', str(out)]
            with ExitStack() as stack:
                forbidden = [stack.enter_context(mock.patch.object(module, name, side_effect=AssertionError('real resource forbidden')))
                             for module, name in [(subprocess, 'Popen'), (socket, 'socket'), (socket, 'create_connection')]]
                stack.enter_context(mock.patch.object(sys, 'argv', argv))
                stack.enter_context(mock.patch.object(signal, 'signal', side_effect=set_handler))
                stack.enter_context(mock.patch.object(tool.importlib.util, 'spec_from_file_location', side_effect=fake_spec))
                stack.enter_context(mock.patch.object(tool.importlib.util, 'module_from_spec', return_value=SimpleNamespace()))
                stack.enter_context(mock.patch.object(tool.selectors, 'DefaultSelector', Selector))
                stack.enter_context(mock.patch.object(tool.os, 'read', side_effect=read))
                stack.enter_context(mock.patch.object(tool.os, 'mkfifo', side_effect=lambda path: actual_write(path, 'mock FIFO, not opened\n')))
                stack.enter_context(mock.patch.object(tool.time, 'monotonic', side_effect=lambda: state['now']))
                stack.enter_context(mock.patch.object(tool.time, 'sleep', side_effect=AssertionError('unexpected wait')))
                stack.enter_context(mock.patch.object(Path, 'write_text', write))
                if real_check is not None:
                    stack.enter_context(mock.patch.object(tool, 'check', side_effect=check))
                result, error = None, None
                try:
                    result = tool.main()
                except BaseException as caught:
                    error = type(caught).__name__
                for forbidden_call in forbidden:
                    forbidden_call.assert_not_called()
            proof_path = out / 'proof.json'
            proof = json.loads(proof_path.read_text()) if proof_path.exists() else None
            return SimpleNamespace(result=result, error=error, proof=proof, state=state, events=events,
                                   children=list(children), stops=stops, handlers=handlers,
                                   has_final_check=real_check is not None)

    def assert_failed(self, result, closed=True):
        self.assertIsNone(result.error)
        self.assertEqual(result.result, 2)
        self.assertIsNotNone(result.proof)
        self.assertEqual(result.proof['status'], 'failed-preserved')
        self.assertEqual(result.proof['owned_groups_closed'], closed)
        self.assertEqual(bool(result.children), not closed)

    def cancellation(self, phase, spawned):
        for number in (signal.SIGTERM, signal.SIGINT):
            with self.subTest(phase=phase, signal=number):
                result = self.exercise(phase, number)
                if phase == 'final_check':
                    self.assertTrue(result.has_final_check, 'structural: final observation hook is absent')
                self.assertTrue(result.state['triggered'])
                self.assertFalse(result.state['missing_handler'], 'structural: no early flag handler was registered')
                self.assert_failed(result)
                self.assertTrue(result.proof['cancelled'])
                self.assertTrue(result.proof['signal_handlers_installed'])
                self.assertEqual(result.state['spawned'], spawned)
                self.assertEqual(result.handlers, {signal.SIGTERM: signal.SIG_DFL, signal.SIGINT: signal.SIG_DFL})

    def test_normal_six_existing_cli_cases(self):
        result = self.exercise()
        self.assertIsNone(result.error)
        self.assertEqual(result.result, 0)
        self.assertEqual(result.proof['status'], 'passed')
        self.assertEqual(result.proof['cases'], CASES)
        self.assertTrue(result.proof['owned_groups_closed'])
        self.assertEqual(result.state['spawned'], 6)
        self.assertFalse(result.children)
        self.assertEqual(result.handlers, {signal.SIGTERM: signal.SIG_DFL, signal.SIGINT: signal.SIG_DFL})
        if result.has_final_check:
            self.assertFalse(result.proof['cancelled'])
            self.assertTrue(result.proof['signal_handlers_installed'])

    def test_handlers_precede_owner_setup(self):
        self.cancellation('owner_init', 0)

    def test_cancel_after_retained_spawn_stops_before_next_child(self):
        self.cancellation('spawn', 1)

    def test_cancel_during_wait_stops_before_next_child(self):
        self.cancellation('wait', 1)

    def test_cancel_during_child_cleanup_stops_before_next_child(self):
        self.cancellation('child_cleanup', 1)

    def test_cancel_during_final_owner_cleanup_rejects_pass(self):
        self.cancellation('owner_cleanup', 6)

    def test_cancel_during_proof_publication_rewrites_failed(self):
        self.cancellation('proof_write', 6)

    def test_cancel_at_final_observation_rewrites_failed(self):
        self.cancellation('final_check', 6)

    def test_cancel_during_handler_restore_rewrites_failed(self):
        self.cancellation('handler_restored', 6)

    def test_handler_restore_failure_rewrites_failed(self):
        result = self.exercise('restore_failure')
        self.assert_failed(result)
        self.assertEqual(result.proof['handler_failure_type'], 'RuntimeError')

    def test_terminal_immediate_exit_still_checks_command_deadline(self):
        result = self.exercise('terminal_timeout')
        self.assert_failed(result)
        self.assertEqual(result.proof['failure_type'], 'AssertionError')

    def test_selector_setup_and_close_keep_original_child_owned_until_cleanup(self):
        for fault in ('selector_constructor', 'selector_register', 'selector_close'):
            with self.subTest(fault=fault):
                result = self.exercise(fault)
                self.assert_failed(result)
                self.assertTrue(result.stops)
                self.assertTrue(all(child is result.stops[0][0] for child, _ in result.stops))

    def test_final_cleanup_errors_still_publish_failed_receipt(self):
        for fault, closed in (('cleanup_failure', True), ('cleanup_remaining', False)):
            with self.subTest(fault=fault):
                result = self.exercise(fault)
                self.assert_failed(result, closed)
                self.assertEqual(result.proof['cleanup_failure_type'], 'RuntimeError')

    def test_command_stop_failure_retries_only_original_owned_child(self):
        result = self.exercise('stop_failure')
        self.assert_failed(result)
        self.assertEqual(len(result.stops), 2)
        self.assertIs(result.stops[0][0], result.stops[1][0])
        self.assertEqual([x[1] for x in result.stops], [False, True])

    def test_existing_time_and_output_bounds_remain_fail_closed(self):
        for fault in ('timeout', 'overflow'):
            with self.subTest(fault=fault):
                result = self.exercise(fault)
                self.assert_failed(result)
                self.assertEqual(result.proof['failure_type'], 'AssertionError')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(add_help=False, allow_abbrev=False)
    parser.add_argument('--tool', type=Path, default=TOOL)
    args, remainder = parser.parse_known_args()
    TOOL = args.tool.resolve(strict=True)
    unittest.main(argv=[sys.argv[0], *remainder])
