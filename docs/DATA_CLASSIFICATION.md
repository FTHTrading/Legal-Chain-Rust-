# LEGAL-CHAIN — Data Classification

## Classification Levels

All data in the legal-chain ecosystem falls into one of these categories:

### Level 1 — PUBLIC (On-Chain, Unencrypted)

Data that is inherently public by virtue of being on a blockchain.

| Data | Example | Storage |
|------|---------|---------|
| Block headers | Hash, height, timestamp, author | Chain state |
| Transaction metadata | Extrinsic index, sender, call type | Chain state |
| Matter IDs and status | Matter #42, status=Active | Pallet storage |
| Content hashes | H256 of documents/evidence | Pallet storage |
| Audit event records | Actor, action type, target reference | Pallet storage |
| Approval status flags | Approved/Pending/Rejected | Pallet storage |

**Rule:** No plaintext names, descriptions, content, or personally identifiable information at this level.

### Level 2 — INTERNAL (Indexed, Access-Controlled)

Data derived from on-chain events, stored in Postgres by the indexer, served via authenticated API.

| Data | Example | Storage |
|------|---------|---------|
| Decoded event history | Full event fields with timestamps | Postgres |
| Aggregated matter timelines | All events for matter #42 | Postgres view |
| Search indexes | Full-text on metadata | Postgres |
| API session data | Auth tokens, rate limits | Redis/memory |

**Rule:** Accessible only to authenticated users with appropriate role. No raw content, only references.

### Level 3 — CONFIDENTIAL (Off-Chain, Encrypted)

Actual document content, evidence files, and privileged communications.

| Data | Example | Storage |
|------|---------|---------|
| Document binaries | PDFs, Word docs, scans | Encrypted FS/S3 |
| Evidence files | Photos, videos, audio | Encrypted FS/S3 |
| Privileged communications | Attorney-client memos | Encrypted FS/S3 |
| Settlement details | Payment terms, amounts | Encrypted FS/S3 |

**Rule:** Encrypted at rest (AES-256-GCM). Encrypted in transit (TLS 1.3). Access requires matter-level authorization. Content hash verified against on-chain record.

### Level 4 — RESTRICTED (Secrets)

Cryptographic keys, credentials, and infrastructure secrets.

| Data | Example | Storage |
|------|---------|---------|
| Validator private keys | Sr25519/Ed25519 keys | Hardware security module or encrypted keystore |
| Database credentials | Postgres connection strings | Environment variables / secrets manager |
| API keys | Service-to-service tokens | Environment variables / secrets manager |
| Encryption keys | Per-matter AES keys | Key management service |

**Rule:** Never logged. Never committed. Never on-chain. Rotated on schedule.

## On-Chain Content Rules

1. **NEVER** store: names, addresses, phone numbers, SSN, financial account numbers, document text, email content, privileged communications
2. **ALWAYS** store as hash: document content, evidence content, jurisdiction text, party descriptions
3. **MAY** store directly: numeric IDs, enum status values, timestamps, account IDs (public keys), boolean flags, version numbers
