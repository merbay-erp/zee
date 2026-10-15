CREATE TABLE zee_job_intents(job_id bigint PRIMARY KEY REFERENCES zee_jobs(id),approval_revision bigint NOT NULL CHECK(approval_revision>0),payload text NOT NULL CHECK(octet_length(payload)<=8192));
