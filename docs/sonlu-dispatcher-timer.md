# K-268 / P09 finite dispatcher timer preparation

This is an external preparation package against the existing CLI at source
54a8d6dc0efb060ae5644b37546da40ad03ee035. It reserves ADR-126 / GB-084 and
changes no compiler, grammar, source tree or supported API. No service was
installed, linked, started or enabled. No actual native dispatcher/PG job was
run by this package. The development machine is macOS; Linux user-systemd
acceptance remains open. The 37 tests are wrapper/model/short owned Python
fixtures, not native jobs, not systemd tests and not provider acceptance.

The existing CLI accepts only the fixed SQL-intent pilot: reap/claim/begin/
apply/renew/fail; a payload is data for zee_job_intents, never executable SQL,
shell, SMTP, HTTP or file callbacks. Its explicit owned localhost PostgreSQL
TLSrequire DSN/CA/credential mapping, clean BuildJSON source and binary/project
pins are retained. This wrapper fixes --ticks 1, retains bounded lease and
attempt behavior, and passes no optional arbitrary command or descriptor list.

## Files and invocation

finite_timer.py uses lifecycle_core.py, a narrow extraction of nine exact
reviewed509106d7 function/class definitions. [portable lineage fixture](../compiler/tests/fixtures/sonlu-dispatcher-lineage-v1.json) binds original and
extracted bytes plus each definition; no artifact, ZIP, bundle, release CLI or
tomllib code is shipped. This extraction receives its own independent review. Python >=3.11 and Unix WNOWAIT are prerequisites. Config is
owner-only0600, single-link, canonical, <=16KiB and exact-key JSON. It contains
paths and pins, never URL/password/key/SQL/body values. Required fields:

- schema: zee-sql-timer-config-1
- repo, source_sha: an exact clean owned clone and its full40hex commit
- dil, binary_sha256: canonical clean native binary path and64hex SHA
- project, project_sha256: existing fixed SQL-intent project and its aggregate
  project_digest from scripts/job-dispatcher.py (five exact files)
- pg_port, ca: explicit owned localhost PG port and public trust PEM path
- credential_root, credential_name: K245 private root and one filename mapped
  to ZEE_JOB_CI_URL; the credential file contains the existing exact owned DSN
- owner: existing bounded owner token, no secret or business event identity

No runnable example config with fabricated binary/project/source hashes is
provided. Populate these from actual final-clean preparation, outside source.
The source/external helper hash map and original private config snapshot are
checked before and after each successful tick.
Source HEAD/tree must remain exact clean; native snapshots/BuildJSON enforce
binary provenance again. This does not create a clean binary or migrations.
The operator must prepare/migrate their disposable owned PG using existing
K258 native fixture; no existing DB is adopted or bootstrapped by this wrapper.

Every invocation defaults to check-only and creates no state:

    python3 -I -B scripts/finite_timer.py --config /owned/config.json --state /owned/state

--render /owned/fresh-units writes only two unit files after the same preflight;
no systemctl, installation or start is invoked. --run is the explicit future
finite execution path. Source/config/native paths never become shell text.
Unit rendering rejects whitespace, control, percent and dollar substitutions;
all unit paths are canonical and absolute. Render/state paths must be external
to repo/project/credentials, canonical, with exact current-owner0700 parents (group/world write or access refused).
Effective UID0 is rejected. Existing state must be exact current-owner0700.

## Pause and exclusive ownership

The static lock file is0600/current-owner/single-link/nofollow and flock is
exclusive/nonblocking. It is held across the full tick and postchecks. A second
manual activation refuses75 before child launch; same systemd service also
serializes activation. A durable inflight.json is created with O_EXCL and
fsync(file)+fsync(directory) BEFORE Popen. Existing latch, even from another
activation/crash, refuses admission. Never erase state or lock as a restart
technique. Source/config validation errors exit2; paused/uncertain execution
exits3; lock contention75. These are wrapper outcomes, not fabricated native
child exit statuses.

Only exact child exit0 plus successful cleanup, unchanged source pins and
fsynced fixed completion metadata allows latch removal. Exit3, nonzero, TERM,
INT, timeout, output overflow, selector setup failure, postcheck failure or
process death before the authorized clear boundary leaves it paused. Timer triggers may re-enter this cheap check;
they cannot start another job while paused. COMMIT ambiguity NEVER causes an
automatic fail write, retry, SQL replay or invented completion acknowledgement.

The narrow reviewed-lineage helper starts/holds its own session/group with WNOWAIT until
TERM/grace/KILL/reap. TERM/INT cancellation is installed before source queries
and causes owned cleanup; no PID/port discovery or foreign adoption occurs.
The child is bounded165s and1MiB captured output; raw stdout/stderr and command
argv are never persisted or emitted. last.json has only fixed outcome, source
SHA, elapsed seconds and output digest. Private tmp is under owned state.
Runtime execution is Linux-only with an available `/proc` descriptor namespace.
No pathname fallback exists. Parent, state and tmp directory descriptors remain
open across dispatcher launch and through owned helper cleanup. The wrapper
checks their dev/inode/type/mode/owner identities against held descriptors and
namespace entries. Child cwd and TMPDIR are `/proc/<this-wrapper-PID>/fd/<its-own
held-state-or-tmp-FD>`, derived only from current getpid and locally opened FDs;
the target inode identities must match. No PID/FD comes from config and no
arbitrary descriptor list is inherited. If the state/tmp pathname is renamed
between a check and Popen, these proc references still identify held directories;
postchecks reject namespace changes and never clear the pause or write a redirected
completion record. Private0700 parents exclude another UID from substituting the
namespace. This is not protection from a malicious same-UID operator who erases
or replaces the entire state between separate activations. Unsupported platform,
unavailable proc alias proof or wrong helper digest quietly exits2, never a
raw assertion/path traceback. No model fallback can be selected by production CLI.

Config/latch/reconciliation reads stat the nofollow entry before open, compare
opened fd metadata, read via a bounded loop (short-read/EINTR handling), compare
post-read fstat plus path identity, size, mode/UID/nlink/mtime/ctime. Config also
checks its parent namespace identity. This rejects observed replacement or
mutation during the snapshot; it is not immunity from an attacker able to forge
all filesystem metadata. Tests physically swap files before open/after read,
mutate content, replace parent/state/tmp namespaces, and force97byte short reads.
Mac tests explicitly mock the Linux proc path resolver and run only short owned
Python helpers; they do not establish the real proc traversal/native behavior.
A real Linux owned native acceptance remains required, including parent death,
cgroup cleanup, no leaked inherited descriptors and pathname-swap confinement.

Source queries use existing bounded helper limits. Single unit activation has
an explicit180s start ceiling; preflight costs may cause this external ceiling
to interrupt work, which correctly leaves admission paused. Stop ceiling10s
is separate. The completion-clear boundary has three outcomes. Full success is exit0. Unknown
SQL/native outcome, cleanup or postpin failure before successful clear remains
latch-present/exit3; unlink failure also preserves that pause. ONLY after native
exit0, owned cleanup, postpins and fsynced last.json completed metadata may the
wrapper unlink the exact latch. If that unlink succeeds but its final directory
fsync fails, the result is DISTINCT exit4 with fixed schema
zee-sql-timer-clear-publication-1 and outcome
completion-visible-durability-unconfirmed. The latch is visibly absent; this is
KNOWN SQL completion with uncertain clear durability, not Paused or unknown
COMMIT. No best-effort restore is claimed. A later activation may admit a fresh
tick while the latch remains visibly absent; power loss may restore an old latch
and conservatively pause. Metadata contains no raw exception or path. Exit4 is
left visible as unit failure (not silently mapped to success), while the timer
can later activate; it must not be interpreted as a provider failure/retry signal.

DirFDs stay held through successful owned cleanup; cleanup failure remains paused
and requires the service cgroup to terminate before reconciliation.
SIGKILL/power-loss cannot run Python finally: the future Linux
service cgroup cleanup and explicit reconciliation remain required. No standalone
--run SIGKILL descendant-cleanup guarantee is claimed.

## Future Linux user-unit validation — NOT EXECUTED

Generated units use Type=oneshot, Restart=no, explicit TimeoutStartSec=180,
TimeoutStopSec=10, KillMode=mixed and SendSIGKILL=yes. TERM reaches the main
wrapper, which cancels/cleans its owned group; systemd provides final cgroup
KILL if needed. No root service or PATH/HOME installation is part of this
package. Timer starts30s after activation and60s after last deactivation,
AccuracySec1s, with no wall-clock calendar, wake or backfill policy. Persistent
is explicitly false (upstream documents that it only affects calendar timers).
These settings are supported by primary upstream documentation:
[monotonic timer semantics](https://raw.githubusercontent.com/systemd/systemd/v257/man/systemd.timer.xml),
[mixed cgroup termination](https://raw.githubusercontent.com/systemd/systemd/v257/man/systemd.kill.xml),
[oneshot start timeout](https://raw.githubusercontent.com/systemd/systemd/v257/man/systemd.service.xml).
This is a semantics reference, not a minimum-version or installed-host claim.

On a disposable owned Linux user manager, AFTER independent review and explicit
execution authorization, the future operator must:

1. Record OS/systemd/python versions, final source/helper/config/binary/project
   hashes, exact unit bytes, and initial own PG state. Reject existing same-name
   zee-sql-intent.service/timer entries unless FragmentPath resolves exactly to
   these newly owned unit files. Never stop/replace an unknown existing unit.
2. Run systemd-analyze --user verify on the two generated files. It has not been
   run on this Mac. Then only runtime link the exact owned absolute files via
   systemctl --user link --runtime; do not enable, linger, publish or install
   under a system directory. Start only the timer after another identity check.
3. Prove one SQL-only tick, next bounded activation, no overlap, normal stop,
   TERM during an owned handler, parent death, timeout and output overflow. Use
   independent owned PG observations, exact cgroup-empty checks and foreign
   sentinels. A missing latch after authorized clear indicates visible known completion;
   it is not an SMTP receipt or external effect proof.
4. Trigger exit3/unknown outcome and verify subsequent timer invocations never
   launch work. Preserve latch/source/result hashes and the PG observation.
5. Stop only the identity-matched owned timer/service; prove own cgroup empty.
   Remove only runtime links verified to point to the owned units; never delete
   arbitrary unit/lock/state paths. No cleanup procedure has run here.

## Explicit reconciliation, not automatic recovery

Stop the identity-matched timer first. An operator separately verifies service
inactive, owned cgroup empty, and actual SQL outcome (commit, rollback, or proof
that no child started). If outcome is still unknown, KEEP PAUSED. The wrapper
cannot authenticate these facts from self-authored JSON. There is no lookup,
SQL query or provider acknowledgement hidden behind the following receipt.

A private0600 reconciliation JSON has exactly:

    schema: zee-sql-timer-reconcile-1
    source_sha: exact same full SHA
    binary_sha256: exact same binary SHA
    inflight_sha256: SHA256 of the exact current private inflight.json
    service_inactive_reviewed: true
    owned_cgroup_empty_reviewed: true
    sql_outcome_reviewed: commit-verified | rollback-verified | no-child-started-verified

After that human review, --reconcile /owned/review.json together with
--reconciliation-sha256 <exact64hex> takes the same exclusive lock, validates
all bindings, fsyncs last-reconciliation.json and removes ONLY the static
inflight.json through its held directory FD. It never launches dispatcher/native work or deletes a path supplied by the
receipt; the CLI still performs bounded read-only Git identity queries. The inflight bytes must also have exactly schema/source_sha/binary_sha256/
created_ns; schema must be zee-sql-timer-inflight-1 and created_ns must be a
positive int <=2**63-1 (bool/float/string/nonfinite rejected). Duplicate keys,
unknown/missing keys and wrong identities fail before unlink and preserve the
exact bytes. Stale/wrong/false/unbound receipts fail closed. If review metadata
was fsynced and unlink succeeded but final directory fsync fails, exit4 emits
reconciliation-visible-durability-unconfirmed: the review was applied visibly,
its clear durability is unconfirmed. This is not a false Paused claim, not DB
proof and not permission to auto-reconcile an unknown SQL result.
A source/binary change, malformed latch or interrupted *.new metadata update
requires preserving evidence and a separately reviewed state migration; do not
remove arbitrary files or create a fresh state to evade a pause. The receipt
is operator assertion plus integrity binding, not cryptographic authorization
or programmatic proof of a successful transaction. No provider fields exist.

## SMTP and PDF remain code/protocol work

The source has SMTP send and readonly PDF MIME attachment support, not an
SMTP/PDF durable effect consumer tied to this queue. An effect consumer still
needs explicit immutable intent/result states, ownership, idempotency key or
provider dedupe contract, attempt boundaries, known-negative retry/backoff,
unknown-outcome quarantine, and independently observed acknowledgement semantics.
A disconnect after send cannot prove delivery or permit blind resend. PDF
rendering/generation, durable file output/publication and atomically relating
that file to a queue result also remain unimplemented for this dispatcher.
Existing readonly attachments are not PDF generation or delivery acknowledgement.
Live provider/product acceptance is separate from these internally actionable
contracts and implementation tasks. P09 remains partial; no external input is
used as an excuse to mark the missing consumer implementation complete.

## V3 review provenance

V1 and V2 source/manifests plus independent R1/R2 probes remain immutable. The
v2 final-unlink/fsync model exposed a false Paused claim with an absent latch;
v3 reports exact visible publication state as described above. The malformed
latch model exposed validation of only two identity keys; v3 validates the
entire typed record. These are wrapper contract corrections, not compiler bugs,
provider duplicates or real Linux/native/systemd acceptance. Explicit CLI exit4
handler models and owned short Python fault tests are counted only as preparation.
