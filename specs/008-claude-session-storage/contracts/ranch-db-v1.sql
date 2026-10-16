-- Ranch database schema contract, version 1.
-- Raw BLOB columns are authoritative; projected columns are nullable indexes.
-- Relational Claude archive only: no graph/node-edge or generic cross-tool tables.

PRAGMA foreign_keys = ON;

-- Turso Connection::execute_batch does not add a transaction. The runtime
-- migration adapter must execute this contract with these transaction bounds
-- and verify foreign_keys on every newly opened connection.
BEGIN IMMEDIATE;

CREATE TABLE claude_archive_sessions (
    session_id TEXT PRIMARY KEY CHECK (length(trim(session_id)) > 0),
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    ended_at TEXT,
    first_cwd TEXT,
    latest_cwd TEXT,
    latest_transcript_path TEXT,
    title TEXT,
    model TEXT,
    last_hook_event TEXT
);

CREATE TABLE claude_hook_receipts (
    receipt_id INTEGER PRIMARY KEY,
    received_at TEXT NOT NULL,
    transport_action TEXT NOT NULL,
    session_id TEXT REFERENCES claude_archive_sessions(session_id),
    hook_event_name TEXT,
    prompt_id TEXT,
    agent_id TEXT,
    cwd TEXT,
    transcript_path TEXT,
    agent_transcript_path TEXT,
    payload_sha256 BLOB NOT NULL CHECK (length(payload_sha256) = 32),
    raw_payload BLOB NOT NULL,
    parse_error TEXT
);

CREATE INDEX claude_hook_receipts_by_session
    ON claude_hook_receipts(session_id, received_at);
CREATE INDEX claude_hook_receipts_by_event
    ON claude_hook_receipts(hook_event_name, received_at);
CREATE INDEX claude_hook_receipts_by_prompt
    ON claude_hook_receipts(prompt_id);
CREATE INDEX claude_hook_receipts_by_digest
    ON claude_hook_receipts(payload_sha256);

CREATE TABLE claude_transcript_sources (
    source_id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES claude_archive_sessions(session_id),
    source_kind TEXT NOT NULL CHECK (source_kind IN ('main', 'subagent')),
    agent_id TEXT,
    canonical_path TEXT NOT NULL,
    generation INTEGER NOT NULL CHECK (generation > 0),
    file_identity TEXT,
    opened_at TEXT NOT NULL,
    last_synced_at TEXT NOT NULL,
    superseded_at TEXT,
    byte_cursor INTEGER NOT NULL DEFAULT 0 CHECK (byte_cursor >= 0),
    next_ordinal INTEGER NOT NULL DEFAULT 0 CHECK (next_ordinal >= 0),
    observed_length INTEGER NOT NULL DEFAULT 0 CHECK (observed_length >= 0),
    observed_mtime INTEGER,
    boundary_sha256 BLOB CHECK (boundary_sha256 IS NULL OR length(boundary_sha256) = 32),
    UNIQUE (canonical_path, generation)
);

CREATE UNIQUE INDEX one_active_claude_transcript_generation
    ON claude_transcript_sources(canonical_path)
    WHERE superseded_at IS NULL;
CREATE INDEX claude_transcript_sources_by_session
    ON claude_transcript_sources(session_id, opened_at);

CREATE TABLE claude_transcript_records (
    source_id INTEGER NOT NULL REFERENCES claude_transcript_sources(source_id),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    byte_offset INTEGER NOT NULL CHECK (byte_offset >= 0),
    byte_length INTEGER NOT NULL CHECK (byte_length >= 0),
    raw_sha256 BLOB NOT NULL CHECK (length(raw_sha256) = 32),
    raw_json BLOB NOT NULL,
    record_delimiter BLOB NOT NULL CHECK (record_delimiter IN (X'', X'0A', X'0D0A')),
    parse_error TEXT,
    record_type TEXT,
    entry_uuid TEXT,
    parent_uuid TEXT,
    logical_parent_uuid TEXT,
    record_session_id TEXT,
    record_session_id_snake TEXT,
    recorded_at TEXT,
    prompt_id TEXT,
    agent_id TEXT,
    is_sidechain INTEGER CHECK (is_sidechain IS NULL OR is_sidechain IN (0, 1)),
    message_role TEXT,
    message_id TEXT,
    model TEXT,
    PRIMARY KEY (source_id, ordinal),
    UNIQUE (source_id, byte_offset)
);

CREATE INDEX claude_transcript_records_by_type
    ON claude_transcript_records(record_type);
CREATE INDEX claude_transcript_records_by_uuid
    ON claude_transcript_records(entry_uuid);
CREATE INDEX claude_transcript_records_by_parent
    ON claude_transcript_records(parent_uuid);
CREATE INDEX claude_transcript_records_by_logical_parent
    ON claude_transcript_records(logical_parent_uuid);
CREATE INDEX claude_transcript_records_by_prompt
    ON claude_transcript_records(prompt_id);
CREATE INDEX claude_transcript_records_by_agent
    ON claude_transcript_records(agent_id);
CREATE INDEX claude_transcript_records_by_session_time
    ON claude_transcript_records(record_session_id, recorded_at);
CREATE INDEX claude_transcript_records_by_snake_session_time
    ON claude_transcript_records(record_session_id_snake, recorded_at);
CREATE INDEX claude_transcript_records_by_message
    ON claude_transcript_records(message_id);

PRAGMA user_version = 1;
COMMIT;
