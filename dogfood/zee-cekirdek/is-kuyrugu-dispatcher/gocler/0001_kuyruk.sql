-- K-247: anonim ürün prototipi; job verisi çalıştırılabilir SQL/kod değildir.
CREATE TABLE zee_job_limits (
 singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton),
 active_limit integer NOT NULL CHECK(active_limit BETWEEN 1 AND 16),
 gate bigint NOT NULL DEFAULT 0);
INSERT INTO zee_job_limits(singleton,active_limit) VALUES(true,2);
CREATE TABLE zee_jobs (
 id bigserial PRIMARY KEY,
 business_key text NOT NULL CHECK(length(business_key) BETWEEN 1 AND 128),
 approval_revision bigint NOT NULL CHECK(approval_revision>0),
 payload text NOT NULL CHECK(octet_length(payload)<=8192),
 state text NOT NULL DEFAULT 'ready' CHECK(state IN ('ready','leased','retry_wait','done','dead_letter')),
 available_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 attempts integer NOT NULL DEFAULT 0 CHECK(attempts BETWEEN 0 AND 3),
 lease_token text NOT NULL DEFAULT '' CHECK(length(lease_token)<=128),
 lease_owner text NOT NULL DEFAULT '' CHECK(length(lease_owner)<=128),
 generation bigint NOT NULL DEFAULT 0 CHECK(generation>=0),
 lease_expires timestamptz,
 phase text NOT NULL DEFAULT 'none' CHECK(phase IN ('none','reserved','started')),
 last_outcome text NOT NULL DEFAULT '',
 transitioned_at timestamptz NOT NULL DEFAULT clock_timestamp(),
 UNIQUE(business_key,approval_revision),
 CHECK((state='leased' AND phase<>'none' AND lease_token<>'' AND lease_owner<>'' AND lease_expires IS NOT NULL)
    OR (state<>'leased' AND phase='none' AND lease_expires IS NULL)),
 CHECK(state<>'dead_letter' OR attempts=3),
 CHECK(phase<>'started' OR attempts>=1));
CREATE INDEX zee_jobs_due ON zee_jobs(available_at,id) WHERE state IN ('ready','retry_wait');
