"""Create the knowledge_base table and indexes."""

import sys
from pathlib import Path

from memory_common import PROJECT_ROOT, database_url

SCHEMA_SQL = """
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
"""

SQL_FILE = PROJECT_ROOT / "tools" / "setup_knowledge_base.sql"


def main() -> int:
    import psycopg

    try:
        with psycopg.connect(database_url()) as conn:
            conn.execute(SCHEMA_SQL)
            conn.commit()
    except psycopg.errors.InsufficientPrivilege:
        print(
            "ERROR: current database user cannot create tables in schema public.\n"
            "Run the one-time setup as postgres instead:\n"
            f"  sudo -u postgres psql kiwi_memory < {SQL_FILE}",
            file=sys.stderr,
        )
        return 1

    print("knowledge_base schema ready.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"ERROR: knowledge base setup failed: {error}", file=sys.stderr)
        sys.exit(1)
