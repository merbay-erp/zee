#!/usr/bin/env python3
"""The real TLS gate cannot silently pass an absent/ignored/failed selector."""

from pathlib import Path
import runpy
import unittest

MODULE = runpy.run_path(str(Path(__file__).resolve().parents[1] / "postgresql-tls-ci.py"))
CHECK = MODULE["check_results"]
TESTS = MODULE["EXPECTED_TESTS"]


def transcript(summary="4 passed; 0 failed; 0 ignored", tests=TESTS):
    return "\n".join(f"test {test} ... ok" for test in tests) + f"\ntest result: ok. {summary};\n"


class StrictGateTests(unittest.TestCase):
    def test_exact_four_real_passes(self):
        CHECK(transcript(), 0)

    def test_missing_selector_cannot_pass(self):
        for output in [transcript(tests=TESTS[:-1]), "test result: ok. 0 passed; 0 failed; 0 ignored;\n"]:
            with self.assertRaises(ValueError):
                CHECK(output, 0)

    def test_ignored_case_and_false_summary_cannot_pass(self):
        for summary in ["3 passed; 0 failed; 1 ignored", "0 passed; 0 failed; 4 ignored", "5 passed; 0 failed; 0 ignored"]:
            with self.assertRaises(ValueError):
                CHECK(transcript(summary), 0)

    def test_failure_exit_and_duplicate_summary_cannot_pass(self):
        with self.assertRaises(ValueError):
            CHECK(transcript(), 1)
        with self.assertRaises(ValueError):
            CHECK(transcript() + "test result: ok. 4 passed; 0 failed; 0 ignored;\n", 0)


if __name__ == "__main__":
    unittest.main()
