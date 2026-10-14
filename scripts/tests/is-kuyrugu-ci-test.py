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

    def test_all_twenty_one_invariants_are_required(self):
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


    def reject_named(self, name, mutations):
        index = list(EXPECTED).index(name)
        self.reject_observations([(index, field, value) for field, value in mutations])

    def test_original_eleven_without_heartbeat_campaign_rejected(self):
        with self.assertRaises(ValueError):
            CHECK(cases()[:11], 0)

    def test_handler_must_exceed_original_lease_in_actual_bounded_seconds(self):
        self.reject_named("separate_lease_heartbeat_commits_while_handler_row_locked",
                          [("held_seconds", 29.0), ("held_seconds", 34.9), ("held_seconds", 61.0),
                           ("held_seconds", float("nan")), ("held_seconds", float("inf")),
                           ("held_seconds", "35.0"), ("held_seconds", 35),
                           ("original30s_expired", False), ("handler_still_blocked_at_renewal_ack", False)])

    def test_independent_heartbeat_commits_must_match_actual_sequence_count(self):
        self.reject_named("separate_lease_heartbeat_commits_while_handler_row_locked",
                          [("heartbeat_acks", 0), ("heartbeat_acks", 1), ("heartbeat_acks", 25),
                           ("heartbeat_acks", True), ("renewals_after", 8), ("renewals_before", 1),
                           ("effects", 2), ("done", False)])

    def test_done_renewal_must_preserve_prior_actual_heartbeat_count(self):
        self.reject_named("done_job_renewal_rejected", [("renewals", 8), ("renewals", True)])
        values = cases()
        values[11]["observations"].update(heartbeat_acks=6, renewals_after=6)
        with self.assertRaises(ValueError):
            CHECK(values, 0)
        values[12]["observations"]["renewals"] = 6
        CHECK(values, 0)

    def test_revoked_or_reused_generation_must_never_resurrect_a_lease(self):
        self.reject_named("early_revoked_lease_reused_token_does_not_resurrect_old_row",
                          [("old_authoritative_expired", False), ("old_compatibility_deadline_future", False),
                           ("returned_current_job", False), ("active_leases", 2)])
        self.reject_named("wrong_token_generation_owner_no_mutation", [("rejections", 2), ("renewals", 1)])
        self.reject_named("expired_old_generation_no_resurrection_or_replacement_mutation",
                          [("generation_increased", False), ("new_renewals", 1)])

    def test_absolute_server_clock_cap_and_expired_renewal_refusal_required(self):
        self.reject_named("renewal_keeps_120second_server_clock_cap",
                          [("renew_until_unchanged", False), ("renew_window_seconds", 121),
                           ("renew_window_seconds", True), ("renewals", 2)])
        self.reject_named("absolute_lifetime_cap_rejected", [("renewals", 1)])

    def test_skiplocked_and_real_dead_backend_required_before_recovery(self):
        self.reject_named("expired_locked_dead_worker_skiplocked_reaper_then_fenced_replacement",
                          [("natural_lease_expiry", False), ("reaper_while_locked", 1),
                           ("reaper_after_backend_gone", 0), ("backend_gone", False), ("killed", False),
                           ("old_generation_rejected", False), ("effects", 2), ("attempts", 3),
                           ("no_heartbeat_resurrection", False)])

    def test_heartbeat_c027_requires_independent_before_after_commit_observations(self):
        self.reject_named("heartbeat_commit_before_c027_no_replay",
                          [("c027", False), ("renewals", 1), ("effects", 1), ("commits_seen", 2),
                           ("commit_forwarded", True), ("commit_complete", True), ("commit_ready", True)])
        self.reject_named("heartbeat_commit_after_c027_no_replay",
                          [("c027", False), ("renewals", 0), ("effects", 1), ("commits_seen", 2),
                           ("commit_forwarded", False), ("commit_complete", False), ("commit_ready", False)])


if __name__ == "__main__":
    unittest.main()
