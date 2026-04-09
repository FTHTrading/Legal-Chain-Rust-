-- 001: Core indexer tables
-- Tracks indexed blocks and decoded chain events for Legal-Chain.

-- Blocks that have been indexed
CREATE TABLE IF NOT EXISTS indexed_blocks (
    block_number    BIGINT PRIMARY KEY,
    block_hash      TEXT NOT NULL,
    parent_hash     TEXT NOT NULL,
    state_root      TEXT NOT NULL,
    extrinsics_root TEXT NOT NULL,
    event_count     INTEGER NOT NULL DEFAULT 0,
    extrinsic_count INTEGER NOT NULL DEFAULT 0,
    block_timestamp TIMESTAMP,
    indexed_at      TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Raw chain events (legal pallets only)
CREATE TABLE IF NOT EXISTS chain_events (
    id          BIGSERIAL PRIMARY KEY,
    block_number BIGINT NOT NULL REFERENCES indexed_blocks(block_number),
    event_index INTEGER NOT NULL,
    pallet      TEXT NOT NULL,
    variant     TEXT NOT NULL,
    data        JSONB NOT NULL DEFAULT '{}',
    indexed_at  TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE (block_number, event_index)
);

CREATE INDEX IF NOT EXISTS idx_events_pallet ON chain_events (pallet);
CREATE INDEX IF NOT EXISTS idx_events_variant ON chain_events (variant);
CREATE INDEX IF NOT EXISTS idx_events_block   ON chain_events (block_number);
