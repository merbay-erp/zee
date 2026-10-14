#!/usr/bin/env python3
"""The real TLS gate cannot silently pass an absent/ignored/failed selector."""

from pathlib import Path
import runpy
import unittest

MODULE = runpy.run_path(str(Path(__file__).resolve().parents[1] / "postgresql-tls-ci.py"))
CHECK = MODULE["check_results"]
TESTS = MODULE["EXPECTED_TESTS"]
OBSERVATIONS = MODULE["FAULT_OBSERVATIONS"]


def transcript(summary="7 passed; 0 failed; 0 ignored", tests=TESTS, observations=OBSERVATIONS):
    return "\n".join(f"test {test} ... ok" for test in tests) + f"\ntest result: ok. {summary};\n" + "\n".join(observations) + "\n"


class StrictGateTests(unittest.TestCase):
    def test_exact_seven_real_passes(self):
        CHECK(transcript(), 0)

    def test_missing_selector_cannot_pass(self):
        for output in [transcript(tests=TESTS[:-1]), "test result: ok. 0 passed; 0 failed; 0 ignored;\n"]:
            with self.assertRaises(ValueError):
                CHECK(output, 0)

    def test_ignored_case_and_false_summary_cannot_pass(self):
        for summary in ["6 passed; 0 failed; 1 ignored", "0 passed; 0 failed; 7 ignored", "8 passed; 0 failed; 0 ignored", "5 passed; 0 failed; 0 ignored"]:
            with self.assertRaises(ValueError):
                CHECK(transcript(summary), 0)

    def test_failure_exit_and_duplicate_summary_cannot_pass(self):
        with self.assertRaises(ValueError):
            CHECK(transcript(), 1)
        with self.assertRaises(ValueError):
            CHECK(transcript() + "test result: ok. 7 passed; 0 failed; 0 ignored;\n", 0)

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

    def test_boolean_or_noninteger_exit_cannot_pass(self):
        for code in [False, True, "0", 0.0]:
            with self.assertRaises(ValueError):
                CHECK(transcript(), code)


if __name__ == "__main__":
    unittest.main()
