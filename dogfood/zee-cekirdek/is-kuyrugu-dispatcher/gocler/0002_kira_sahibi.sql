-- Design candidate: immutable v2 lease migration. No mixed helper versions.
CREATE TABLE zee_job_leases (
 job_id bigint PRIMARY KEY REFERENCES zee_jobs(id),
 generation bigint NOT NULL CHECK(generation>0),
 lease_token text NOT NULL CHECK(length(lease_token) BETWEEN 1 AND 128),
 lease_owner text NOT NULL CHECK(length(lease_owner) BETWEEN 1 AND 128),
 issued_at timestamptz NOT NULL,
 expires_at timestamptz NOT NULL,
 renew_until timestamptz NOT NULL,
 renewals integer NOT NULL DEFAULT 0 CHECK(renewals BETWEEN 0 AND 24),
 CHECK(issued_at<=expires_at), CHECK(expires_at<=renew_until));
-- Existing leases retain their exact deadline; migration gives no extension.
INSERT INTO zee_job_leases(job_id,generation,lease_token,lease_owner,issued_at,expires_at,renew_until)
 SELECT id,generation,lease_token,lease_owner,lease_expires-interval '30 seconds',lease_expires,lease_expires
 FROM zee_jobs WHERE state='leased';
