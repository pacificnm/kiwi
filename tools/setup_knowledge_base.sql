-- One-time setup for the knowledge_base table (Rust book, eGUI, etc.).
-- Run as the PostgreSQL superuser. Pipe via stdin (postgres cannot read ~/):
--   sudo -u postgres psql kiwi_memory < tools/setup_knowledge_base.sql
--
-- Replace "jaimie" in the GRANT lines if your application DB user differs.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS knowledge_base (
    id           bigserial PRIMARY KEY,
    collection   text NOT NULL,
    source_path  text NOT NULL,
    content      text NOT NULL,
    content_hash text NOT NULL,
    embedding    vector(1536) NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS kb_hash_idx
    ON knowledge_base (content_hash);

CREATE INDEX IF NOT EXISTS kb_coll_idx
    ON knowledge_base (collection);

CREATE INDEX IF NOT EXISTS kb_embed_idx
    ON knowledge_base
    USING hnsw (embedding vector_cosine_ops);

GRANT SELECT, INSERT, UPDATE, DELETE ON knowledge_base TO "jaimie";
GRANT USAGE, SELECT ON SEQUENCE knowledge_base_id_seq TO "jaimie";
