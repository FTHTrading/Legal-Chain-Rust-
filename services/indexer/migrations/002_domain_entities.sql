-- 002: Domain entity tables
-- Materialized views of on-chain legal objects, updated by the indexer.

-- Matters
CREATE TABLE IF NOT EXISTS matters (
    matter_id        BIGINT PRIMARY KEY,
    creator          TEXT NOT NULL,
    matter_type      TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'Draft',
    jurisdiction_hash TEXT NOT NULL,
    created_block    BIGINT NOT NULL,
    updated_block    BIGINT NOT NULL,
    indexed_at       TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_matters_status ON matters (status);
CREATE INDEX IF NOT EXISTS idx_matters_creator ON matters (creator);

-- Evidence
CREATE TABLE IF NOT EXISTS evidence (
    evidence_id    BIGINT PRIMARY KEY,
    matter_id      BIGINT NOT NULL,
    registrar      TEXT NOT NULL,
    content_hash   TEXT NOT NULL,
    status         TEXT NOT NULL DEFAULT 'Submitted',
    custody_state  TEXT NOT NULL DEFAULT 'InPossession',
    created_block  BIGINT NOT NULL,
    updated_block  BIGINT NOT NULL,
    indexed_at     TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_evidence_matter ON evidence (matter_id);
CREATE INDEX IF NOT EXISTS idx_evidence_status ON evidence (status);

-- Documents
CREATE TABLE IF NOT EXISTS documents (
    document_id      BIGINT PRIMARY KEY,
    matter_id        BIGINT NOT NULL,
    registrar        TEXT NOT NULL,
    content_hash     TEXT NOT NULL,
    version          INTEGER NOT NULL DEFAULT 1,
    status           TEXT NOT NULL DEFAULT 'Draft',
    filing_readiness TEXT NOT NULL DEFAULT 'NotReady',
    created_block    BIGINT NOT NULL,
    updated_block    BIGINT NOT NULL,
    indexed_at       TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_documents_matter ON documents (matter_id);
CREATE INDEX IF NOT EXISTS idx_documents_status ON documents (status);

-- Audit trail
CREATE TABLE IF NOT EXISTS audit_trail (
    audit_id     BIGINT PRIMARY KEY,
    matter_id    BIGINT,
    actor        TEXT NOT NULL,
    action       TEXT NOT NULL,
    target_type  TEXT NOT NULL,
    target_id    BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    indexed_at   TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_matter ON audit_trail (matter_id);
CREATE INDEX IF NOT EXISTS idx_audit_actor  ON audit_trail (actor);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_trail (action);

-- Approvals
CREATE TABLE IF NOT EXISTS approvals (
    approval_id   BIGINT PRIMARY KEY,
    matter_id     BIGINT NOT NULL,
    subject_type  TEXT NOT NULL,
    subject_id    BIGINT NOT NULL,
    requester     TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'Pending',
    created_block BIGINT NOT NULL,
    updated_block BIGINT NOT NULL,
    indexed_at    TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_approvals_matter ON approvals (matter_id);
CREATE INDEX IF NOT EXISTS idx_approvals_status ON approvals (status);

-- Identities
CREATE TABLE IF NOT EXISTS identities (
    credential_id BIGINT PRIMARY KEY,
    subject       TEXT NOT NULL,
    role          TEXT NOT NULL,
    registered_by TEXT NOT NULL,
    is_active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_block BIGINT NOT NULL,
    updated_block BIGINT NOT NULL,
    indexed_at    TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_identities_subject ON identities (subject);
CREATE INDEX IF NOT EXISTS idx_identities_role    ON identities (role);
