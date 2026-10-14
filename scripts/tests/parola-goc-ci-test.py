#!/usr/bin/env python3
"""Strict native gate never accepts missing, ignored or false SQL observations."""

import copy
from pathlib import Path
import runpy
import unittest

MODULE = runpy.run_path(str(Path(__file__).resolve().parents[1] / "parola-goc-ci.py"))
CHECK, EXPECTED = MODULE["check_results"], MODULE["EXPECTED"]


def cases():
    return [{"name": name, "status": "passed", "ignored": False,
             "observations": copy.deepcopy(observations)} for name, observations in EXPECTED.items()]


class StrictNativeGateTests(unittest.TestCase):
    def test_all_seven_actual_case_invariants_required(self):
        CHECK(cases(), 0)

    def test_missing_case_or_empty_campaign_rejected(self):
        for values in ([], cases()[:-1]):
            with self.assertRaises(ValueError):
                CHECK(values, 0)

    def test_duplicate_case_rejected(self):
        values = cases()
        values[-1] = values[0]
        with self.assertRaises(ValueError):
            CHECK(values, 0)

    def test_ignored_or_failed_case_rejected(self):
        for field, value in (("ignored", True), ("status", "failed"), ("status", "skipped")):
            values = cases()
            values[0][field] = value
            with self.assertRaises(ValueError):
                CHECK(values, 0)

    def test_nonzero_exit_cannot_pass(self):
        with self.assertRaises(ValueError):
            CHECK(cases(), 1)

    def test_cas_two_winners_or_zero_waiting_clients_rejected(self):
        for field, value in (("accepted", 2), ("waiting_clients", 0), ("sessions", 2)):
            values = cases()
            values[3]["observations"][field] = value
            with self.assertRaises(ValueError):
                CHECK(values, 0)

    def test_uncommitted_session_or_missing_actual_bind_attempt_rejected(self):
        for index, field, value in ((6, "sessions_before_commit", 1),
                                    (5, "bind_attempts", 0), (4, "legacy_preserved", False)):
            values = cases()
            values[index]["observations"][field] = value
            with self.assertRaises(ValueError):
                CHECK(values, 0)

    def test_model_extraction_cannot_silently_replace_missing_existing_helper(self):
        with self.assertRaises(ValueError):
            MODULE["extract_model"]("not the existing anonymous model")

    def test_boolean_value_is_not_a_numeric_sql_count(self):
        values = cases()
        values[3]["observations"]["accepted"] = True
        with self.assertRaises(ValueError):
            CHECK(values, 0)


if __name__ == "__main__":
    unittest.main()
