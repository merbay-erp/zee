#!/usr/bin/env python3
"""The real TLS gate cannot silently pass an absent/ignored/failed selector."""

from pathlib import Path
import runpy
import unittest
from unittest import mock

MODULE = runpy.run_path(str(Path(__file__).resolve().parents[1] / "postgresql-tls-ci.py"))
CHECK = MODULE["check_results"]
TESTS = MODULE["EXPECTED_TESTS"]
OBSERVATIONS = MODULE["FAULT_OBSERVATIONS"]


def transcript(summary="8 passed; 0 failed; 0 ignored", tests=TESTS, observations=OBSERVATIONS):
    return "\n".join(f"test {test} ... ok" for test in tests) + f"\ntest result: ok. {summary};\n" + "\n".join(observations) + "\n"


class StrictGateTests(unittest.TestCase):
    def test_exact_eight_real_passes(self):
        CHECK(transcript(), 0)

    def test_missing_selector_cannot_pass(self):
        for output in [transcript(tests=TESTS[:-1]), "test result: ok. 0 passed; 0 failed; 0 ignored;\n"]:
            with self.assertRaises(ValueError):
                CHECK(output, 0)

    def test_ignored_case_and_false_summary_cannot_pass(self):
        for summary in ["7 passed; 0 failed; 1 ignored", "0 passed; 0 failed; 8 ignored", "9 passed; 0 failed; 0 ignored", "5 passed; 0 failed; 0 ignored"]:
            with self.assertRaises(ValueError):
                CHECK(transcript(summary), 0)

    def test_failure_exit_and_duplicate_summary_cannot_pass(self):
        with self.assertRaises(ValueError):
            CHECK(transcript(), 1)
        with self.assertRaises(ValueError):
            CHECK(transcript() + "test result: ok. 8 passed; 0 failed; 0 ignored;\n", 0)

    def test_duplicate_or_unexpected_case_cannot_pass(self):
        for tests in [TESTS + (TESTS[0],), TESTS + ("invented_case",), TESTS[:-1] + (TESTS[0],)]:
            with self.assertRaises(ValueError):
                CHECK(transcript(tests=tests), 0)

    def test_original_five_without_fault_cases_cannot_pass(self):
        with self.assertRaises(ValueError):
            CHECK(transcript("5 passed; 0 failed; 0 ignored", TESTS[:5]), 0)

    def test_fault_observation_absent_or_duplicated_cannot_pass(self):
        for observations in [(), OBSERVATIONS[:1], OBSERVATIONS + (OBSERVATIONS[0],)]:
            with self.assertRaises(ValueError):
                CHECK(transcript(observations=observations), 0)

    def test_wrong_state_replay_or_missing_health_cannot_pass(self):
        for old, new in [("40P01", "57014"), ("40001", "40P01"), ("attempts=4", "attempts=5"),
                         ("attempts=2", "attempts=3"), ("durable_sum=0", "durable_sum=1"),
                         ("victim_marker=0", "victim_marker=10"), ("fresh_health=true", "fresh_health=false")]:
            with self.assertRaises(ValueError):
                CHECK(transcript().replace(old, new), 0)

    def test_tls11_exact_verified_capable_control(self):
        MODULE['TLS11']['validate_control']({'protocol':'TLSv1.1','server_protocol':'TLSv1.1','hostname_verified':True,'certificate_verified':True,'cipher':'ECDHE-RSA-AES256-SHA'})

    def test_tls11_absent_or_extra_control_cannot_pass(self):
        for control in [None, {}, {'protocol':'TLSv1.1'}, {'protocol':'TLSv1.1','server_protocol':'TLSv1.1','hostname_verified':True,'certificate_verified':True,'cipher':'AES','extra':True}]:
            with self.assertRaises(ValueError): MODULE['TLS11']['validate_control'](control)

    def test_tls11_unverified_or_wrong_protocol_cannot_pass(self):
        base={'protocol':'TLSv1.1','server_protocol':'TLSv1.1','hostname_verified':True,'certificate_verified':True,'cipher':'AES'}
        for key,value in [('protocol','TLSv1.2'),('server_protocol','TLSv1.2'),('hostname_verified',False),('certificate_verified','true')]:
            with self.assertRaises(ValueError): MODULE['TLS11']['validate_control']({**base,key:value})

    def test_tls11_missing_or_malformed_cipher_cannot_pass(self):
        base={'protocol':'TLSv1.1','server_protocol':'TLSv1.1','hostname_verified':True,'certificate_verified':True,'cipher':'AES'}
        for cipher in ['', None, 'x'*129, 'AES\n', 'Ş']:
            with self.assertRaises(ValueError): MODULE['TLS11']['validate_control']({**base,'cipher':cipher})

    def test_tls11_expired_handshake_budget_prevents_later_io(self):
        probe = MODULE['TLS11']['probe']
        tls = probe.__globals__
        for expired_at in ('before-handshake', 'during-handshake'):
            with self.subTest(expired_at=expired_at):
                clock = [0.0]
                raw_writes, tls_writes, handshake_hosts = [], [], []
                closed = []

                class Raw:
                    def __enter__(self):
                        return self
                    def __exit__(self, *args):
                        closed.append('raw')
                    def settimeout(self, timeout):
                        self_timeout = timeout
                        if not 0 < self_timeout <= 5:
                            raise AssertionError('remaining timeout must be positive and bounded')
                    def sendall(self, data):
                        raw_writes.append(data)
                    def recv(self, size):
                        if expired_at == 'before-handshake':
                            clock[0] = 6.0
                        return b'S'

                class Stream(Raw):
                    def __exit__(self, *args):
                        closed.append('tls')
                    def version(self):
                        return 'TLSv1.1'
                    def cipher(self):
                        return ('AES', 'TLSv1.1', 128)
                    def sendall(self, data):
                        tls_writes.append(data)

                class Context:
                    def set_ciphers(self, value):
                        pass
                    def load_verify_locations(self, **args):
                        pass
                    def wrap_socket(self, raw, *, server_hostname):
                        handshake_hosts.append(server_hostname)
                        clock[0] = 6.0
                        return Stream()

                with mock.patch.object(tls['ssl'], 'SSLContext', return_value=Context()), \
                     mock.patch.object(tls['socket'], 'create_connection', return_value=Raw()) as connect, \
                     mock.patch.object(tls['time'], 'monotonic', side_effect=lambda: clock[0]):
                    with self.assertRaisesRegex(TimeoutError, 'positive control deadline'):
                        probe(12345, '/unused/fixture-ca')
                connect.assert_called_once_with(('127.0.0.1', 12345), timeout=5.0)
                self.assertEqual(len(raw_writes), 1, 'only initial SSLRequest may be written')
                self.assertEqual(tls_writes, [], 'no Startup/Query/Terminate after deadline')
                self.assertEqual(handshake_hosts, [] if expired_at == 'before-handshake' else ['localhost'])
                self.assertEqual(closed, ['raw'] if expired_at == 'before-handshake' else ['tls', 'raw'])

    def test_boolean_or_noninteger_exit_cannot_pass(self):
        for code in [False, True, "0", 0.0]:
            with self.assertRaises(ValueError):
                CHECK(transcript(), code)


if __name__ == "__main__":
    unittest.main()
