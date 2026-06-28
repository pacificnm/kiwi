use anyhow::{Context, Result};
use pgvector::Vector;

pub struct ContextDb {
    client: postgres::Client,
}

pub struct ContextEntry {
    pub id: i64,
    pub session_key: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

impl ContextDb {
    pub fn connect(url: &str) -> Result<Self> {
        let client = postgres::Client::connect(url, postgres::NoTls)
            .with_context(|| format!("failed to connect to PostgreSQL at {url}"))?;
        Ok(Self { client })
    }

    pub fn setup_schema(&mut self, dim: usize) -> Result<()> {
        self.client
            .execute("CREATE EXTENSION IF NOT EXISTS vector", &[])
            .context("failed to create vector extension")?;

        self.client
            .execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS agent_context_memory (
                        id          bigserial PRIMARY KEY,
                        session_key text NOT NULL DEFAULT '',
                        title       text NOT NULL DEFAULT '',
                        content     text NOT NULL,
                        tags        text[] NOT NULL DEFAULT '{{}}',
                        embedding   vector({dim}) NOT NULL,
                        created_at  timestamptz NOT NULL DEFAULT now()
                    )"
                ),
                &[],
            )
            .context("failed to create agent_context_memory table")?;

        self.client
            .execute(
                "CREATE INDEX IF NOT EXISTS ctx_embedding_idx
                 ON agent_context_memory USING hnsw (embedding vector_cosine_ops)",
                &[],
            )
            .context("failed to create embedding index")?;

        self.client
            .execute(
                "CREATE INDEX IF NOT EXISTS ctx_session_time_idx
                 ON agent_context_memory (session_key, created_at DESC)",
                &[],
            )
            .context("failed to create session index")?;

        self.client
            .execute(
                "CREATE INDEX IF NOT EXISTS ctx_time_idx
                 ON agent_context_memory (created_at DESC)",
                &[],
            )
            .context("failed to create time index")?;

        Ok(())
    }

    pub fn save(
        &mut self,
        content: &str,
        title: &str,
        session_key: &str,
        tags: &[String],
        embedding: &[f32],
    ) -> Result<i64> {
        let vec = Vector::from(embedding.to_vec());
        let row = self
            .client
            .query_one(
                "INSERT INTO agent_context_memory
                    (content, title, session_key, tags, embedding)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING id",
                &[&content, &title, &session_key, &tags, &vec],
            )
            .context("failed to insert context memory entry")?;
        Ok(row.get::<_, i64>(0))
    }

    pub fn search(
        &mut self,
        query_embedding: &[f32],
        limit: i32,
        session_key: &str,
    ) -> Result<Vec<ContextEntry>> {
        let vec = Vector::from(query_embedding.to_vec());
        let rows = if session_key.is_empty() {
            self.client.query(
                "SELECT id, session_key, title, content, tags,
                        to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS ts
                 FROM agent_context_memory
                 ORDER BY embedding <=> $1
                 LIMIT $2",
                &[&vec, &limit],
            )
        } else {
            self.client.query(
                "SELECT id, session_key, title, content, tags,
                        to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS ts
                 FROM agent_context_memory
                 WHERE session_key = $3
                 ORDER BY embedding <=> $1
                 LIMIT $2",
                &[&vec, &limit, &session_key],
            )
        }
        .context("failed to search context memory")?;

        Ok(rows.into_iter().map(row_to_entry).collect())
    }

    pub fn list(&mut self, limit: i32, session_key: &str) -> Result<Vec<ContextEntry>> {
        let rows = if session_key.is_empty() {
            self.client.query(
                "SELECT id, session_key, title, content, tags,
                        to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS ts
                 FROM agent_context_memory
                 ORDER BY created_at DESC
                 LIMIT $1",
                &[&limit],
            )
        } else {
            self.client.query(
                "SELECT id, session_key, title, content, tags,
                        to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS ts
                 FROM agent_context_memory
                 WHERE session_key = $2
                 ORDER BY created_at DESC
                 LIMIT $1",
                &[&limit, &session_key],
            )
        }
        .context("failed to list context memory")?;

        Ok(rows.into_iter().map(row_to_entry).collect())
    }

    pub fn get(&mut self, entry_id: i64) -> Result<Option<ContextEntry>> {
        let rows = self
            .client
            .query(
                "SELECT id, session_key, title, content, tags,
                        to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS ts
                 FROM agent_context_memory
                 WHERE id = $1",
                &[&entry_id],
            )
            .context("failed to get context memory entry")?;

        Ok(rows.into_iter().next().map(row_to_entry))
    }

    pub fn existing_dim(&mut self) -> Result<Option<usize>> {
        let rows = self
            .client
            .query(
                "SELECT atttypmod
                 FROM pg_attribute
                 JOIN pg_class ON pg_class.oid = pg_attribute.attrelid
                 WHERE pg_class.relname = 'agent_context_memory'
                   AND pg_attribute.attname = 'embedding'
                   AND pg_attribute.attnum > 0",
                &[],
            )
            .context("failed to query schema information")?;

        if let Some(row) = rows.first() {
            let typmod: i32 = row.get(0);
            if typmod > 4 {
                return Ok(Some((typmod - 4) as usize));
            }
        }
        Ok(None)
    }
}

fn row_to_entry(row: postgres::Row) -> ContextEntry {
    ContextEntry {
        id: row.get("id"),
        session_key: row.get("session_key"),
        title: row.get("title"),
        content: row.get("content"),
        tags: row.get("tags"),
        created_at: row.get::<_, String>("ts"),
    }
}
