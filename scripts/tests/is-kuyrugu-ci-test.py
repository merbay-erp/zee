#!/usr/bin/env python3
"""K-247 native skor kartı sahte başarı, eksik gözlem veya tekrar kabul etmez."""
import copy
from pathlib import Path
import runpy
import unittest

MODULE = runpy.run_path(str(Path(__file__).resolve().parents[1] / "is-kuyrugu-ci.py"))
CHECK, EXPECTED = MODULE["check_results"], MODULE["EXPECTED"]


def cases():
    return [{"name": name, "status": "passed", "ignored": False,
             "observations": copy.deepcopy(observations)} for name, observations in EXPECTED.items()]


class StrictQueueGateTests(unittest.TestCase):
    def reject_observations(self, mutations):
        for index, field, value in mutations:
            with self.subTest(index=index, field=field, value=value):
                values = cases()
                values[index]["observations"][field] = value
                with self.assertRaises(ValueError):
                    CHECK(values, 0)

    def test_all_eleven_invariants_are_required(self):
        CHECK(cases(), 0)

    def test_empty_or_missing_campaign_rejected(self):
        for values in ([], cases()[:-1]):
            with self.assertRaises(ValueError):
                CHECK(values, 0)

    def test_duplicate_case_rejected(self):
        values = cases(); values[-1] = values[0]
        with self.assertRaises(ValueError):
            CHECK(values, 0)

    def test_reordered_cases_rejected(self):
        values = cases(); values[0], values[1] = values[1], values[0]
        with self.assertRaises(ValueError):
            CHECK(values, 0)

    def test_unknown_case_or_result_shape_rejected(self):
        for mutate in (lambda v: v[0].update(name="invented"),
                       lambda v: v[0].update(extra="unobserved"),
                       lambda v: v.__setitem__(0, None)):
            values = cases(); mutate(values)
            with self.assertRaises(ValueError):
                CHECK(values, 0)

    def test_failed_ignored_or_implicit_result_rejected(self):
        for field, value in (("ignored", True), ("ignored", None), ("status", "failed"),
                             ("status", "skipped")):
            values = cases(); values[0][field] = value
            with self.assertRaises(ValueError):
                CHECK(values, 0)
        values = cases(); del values[0]["ignored"]
        with self.assertRaises(ValueError):
            CHECK(values, 0)

    def test_nonzero_or_boolean_exit_rejected(self):
        for exit_code in (1, -9, False, True, "0"):
            with self.assertRaises(ValueError):
                CHECK(cases(), exit_code)

    def test_boolean_is_not_a_numeric_database_observation(self):
        self.reject_observations([(0, "jobs", True), (2, "effects_before", False),
                                  (8, "effects", True), (3, "attempts", 3.0)])

    def test_missing_or_extra_observation_rejected(self):
        values = cases(); del values[0]["observations"]["same_id"]
        with self.assertRaises(ValueError):
            CHECK(values, 0)
        self.reject_observations([(0, "invented", True)])

    def test_idempotency_or_payload_interpretation_failure_rejected(self):
        self.reject_observations([(0, "jobs", 2), (0, "same_id", False),
                                  (0, "changed_payload_rejected", False), (0, "payload_is_data", False)])

    def test_fake_concurrency_or_exceeded_lease_bound_rejected(self):
        self.reject_observations([(1, "waiting_workers", 0), (1, "distinct_jobs", 1),
                                  (1, "active_leases", 3), (1, "third_worker_empty", False),
                                  (1, "same_owner_empty", False)])

    def test_stale_generation_or_double_effect_rejected(self):
        self.reject_observations([(2, "expired_token_reuse_selects_current_job", False),
                                  (2, "generation_increased", False), (2, "stale_rejected", False),
                                  (2, "effects_before", 1), (2, "effects_after", 2),
                                  (2, "repeat_rejected", False), (2, "state_done", False)])

    def test_missing_backoff_or_fourth_attempt_rejected(self):
        self.reject_observations([(3, "first_backoff", 0), (3, "second_backoff", 1),
                                  (3, "early_claim_empty", False), (3, "attempts", 4),
                                  (3, "handler_statements", 4), (4, "effects", 1),
                                  (4, "dead_letter", False), (4, "fourth_claim_empty", False)])

    def test_fake_crash_or_replayed_acknowledged_commit_rejected(self):
        self.reject_observations([(5, "actual_lock_wait", False), (5, "uncommitted_invisible", False),
                                  (5, "killed", False), (5, "effects_after_kill", 1),
                                  (5, "effects_after_recovery", 2), (6, "ack_observed", False),
                                  (6, "handler_statements", 2), (6, "reconciled_done", False)])

    def test_handler_unknown_commit_cannot_replay_or_claim_wrong_outcome(self):
        self.reject_observations([(7, "c027", False), (7, "stdout_empty", False),
                                  (7, "commit_forwarded", True), (7, "effects", 1),
                                  (8, "commit_complete", False), (8, "commit_ready", False),
                                  (8, "commits_seen", 2), (8, "effects", 0),
                                  (8, "state_done", False), (8, "reconcile_no_replay", False)])

    def test_enqueue_unknown_commit_requires_actual_key_reconciliation(self):
        self.reject_observations([(9, "jobs_before_reconcile", 1), (9, "reconciled_missing", False),
                                  (9, "jobs_after_explicit_decision", 2), (10, "reconciled_ready", False),
                                  (10, "jobs_before_reconcile", 0), (10, "jobs_after_reconcile", 2)])


if __name__ == "__main__":
    unittest.main()
