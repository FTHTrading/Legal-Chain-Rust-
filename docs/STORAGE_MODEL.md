# LEGAL-CHAIN — Storage Model

## On-Chain Storage (Substrate Runtime State)

All on-chain storage uses Substrate's Merkle-Patricia trie. Storage items are defined in FRAME pallets using typed storage macros.

### Storage Types Used

| Type | Usage | Key Scheme |
|------|-------|------------|
| `StorageValue<_, T>` | Singleton values (NextId counters) | Fixed key |
| `StorageMap<_, Blake2_128Concat, K, V>` | Primary record maps (ID → Record) | Blake2-128 concatenated |
| `StorageDoubleMap<_, B, K1, B, K2, V>` | Secondary indexes (Creator × ID → ()) | Blake2-128 concatenated |

### Key Hashing: Blake2_128Concat

All user-controlled keys use `Blake2_128Concat` which provides:
- Resistance to storage key collision attacks
- Key enumeration support (for iteration and migration)
- Deterministic key derivation

### Pallet Storage Layout

**pallet-matters:**
- `NextMatterId: StorageValue<_, u64>` — auto-increment counter
- `Matters: StorageMap<_, Blake2_128Concat, u64, MatterRecord<T>>` — primary store
- `MattersByCreator: StorageDoubleMap<_, Blake2_128Concat, AccountId, Blake2_128Concat, u64, ()>` — index

**pallet-evidence:**
- `NextEvidenceId: StorageValue<_, u64>`
- `EvidenceRecords: StorageMap<_, Blake2_128Concat, u64, EvidenceRecord<T>>`
- `EvidenceByMatter: StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u64, ()>`

**pallet-documents:**
- `NextDocumentId: StorageValue<_, u64>`
- `Documents: StorageMap<_, Blake2_128Concat, u64, DocumentRecord<T>>`
- `DocumentsByMatter: StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u64, ()>`

**pallet-audit:**
- `NextAuditId: StorageValue<_, u64>`
- `AuditEvents: StorageMap<_, Blake2_128Concat, u64, AuditRecord<T>>`
- `AuditByMatter: StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, u64, ()>`
- `AuditByActor: StorageDoubleMap<_, Blake2_128Concat, AccountId, Blake2_128Concat, u64, ()>`

### Bounded Types

All variable-length fields use `BoundedVec` with config-defined maximums:
- `MaxTitleLength` — typically 256 bytes
- `MaxDescriptionLength` — typically 1024 bytes  
- `MaxUriLength` — typically 512 bytes
- `MaxPartiesPerMatter` — typically 32
- `MaxMetadataLength` — typically 2048 bytes

### Storage Migrations

Each pallet declares a `StorageVersion`. When upgrading pallets, storage migrations are executed via `frame_support::traits::OnRuntimeUpgrade`. Migration code reads old format, transforms, writes new format, and bumps the version.

## Off-Chain Storage (Phase 3+)

**Local Development:**
- File system at `./storage/` relative to node
- Organized as `storage/{matter_id}/{object_type}/{content_hash}`
- Encrypted with AES-256-GCM per-matter key

**Production:**
- S3-compatible object store
- Bucket per environment (dev/staging/prod)
- Server-side encryption with KMS-managed keys
- Lifecycle policies for archival (move to Glacier after case closure + retention period)

## Indexer Database (Phase 3+)

PostgreSQL schema mirrors on-chain state with additional query capabilities:
- Foreign key relationships between domain objects
- Full-text search indexes on metadata fields
- Materialized views for common query patterns
- Event log tables for complete audit trail
- Partitioned by block range for performance
