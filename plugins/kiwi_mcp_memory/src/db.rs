use anyhow::{Context, Result};
use pgvector::Vector;

pub struct MemoryDb {
    client: postgres::Client,
}

pub struct SearchResult {
    pub source_path: String,
    pub content: String,
    pub score: f32,
}

pub struct KnowledgeResult {
    pub collection: String,
    pub source_path: String,
    pub content: String,
    pub score: f32,
}

impl MemoryDb {
    pub fn connect(url: &str) -> Result<Self> {
        let client = postgres::Client::connect(url, postgres::NoTls)
            .with_context(|| format!("failed to connect to PostgreSQL at {url}"))?;
        Ok(Self { client })
    }

    /// Create the project_memory table and indexes.
    /// `dim` must match the embedding model (768 for nomic-embed-text, 1536 for OpenAI).
    pub fn setup_schema(&mut self, dim: usize) -> Result<()> {
        self.client
            .execute("CREATE EXTENSION IF NOT EXISTS vector", &[])
            .context("failed to create vector extension")?;

        self.client
            .execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS project_memory (
                        id           bigserial PRIMARY KEY,
                        source_path  text NOT NULL,
                        content      text NOT NULL,
                        content_hash text NOT NULL,
                        embedding    vector({dim}) NOT NULL,
                        created_at   timestamptz NOT NULL DEFAULT now()
                    )"
                ),
                &[],
            )
            .context("failed to create project_memory table")?;

        self.client
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS project_memory_hash_idx
                 ON project_memory (content_hash)",
                &[],
            )
            .context("failed to create hash index")?;

        self.client
            .execute(
                "CREATE INDEX IF NOT EXISTS project_memory_embedding_idx
                 ON project_memory USING hnsw (embedding vector_cosine_ops)",
                &[],
            )
            .context("failed to create hnsw index")?;

        Ok(())
    }

    /// Insert or skip (on hash conflict) a chunk.
    pub fn upsert(
        &mut self,
        source_path: &str,
        content: &str,
        hash: &str,
        embedding: &[f32],
    ) -> Result<()> {
        let vec = Vector::from(embedding.to_vec());
        self.client
            .execute(
                "INSERT INTO project_memory (source_path, content, content_hash, embedding)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (content_hash) DO NOTHING",
                &[&source_path, &content, &hash, &vec],
            )
            .context("failed to upsert project_memory row")?;
        Ok(())
    }

    /// Semantic search: returns up to `limit` results ordered by cosine similarity.
    pub fn search(
        &mut self,
        query_embedding: &[f32],
        limit: i32,
    ) -> Result<Vec<SearchResult>> {
        let vec = Vector::from(query_embedding.to_vec());
        let rows = self
            .client
            .query(
                "SELECT source_path, content,
                        (1.0 - (embedding <=> $1))::real AS score
                 FROM project_memory
                 ORDER BY embedding <=> $1
                 LIMIT $2",
                &[&vec, &limit],
            )
            .context("failed to query project_memory")?;

        Ok(rows
            .into_iter()
            .map(|row| SearchResult {
                source_path: row.get("source_path"),
                content: row.get("content"),
                score: row.get::<_, f32>("score"),
            })
            .collect())
    }

    // ── knowledge_base ────────────────────────────────────────────────────────

    pub fn setup_knowledge_schema(&mut self, dim: usize) -> Result<()> {
        self.client
            .execute("CREATE EXTENSION IF NOT EXISTS vector", &[])
            .context("failed to create vector extension")?;

        self.client
            .execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS knowledge_base (
                        id           bigserial PRIMARY KEY,
                        collection   text NOT NULL,
                        source_path  text NOT NULL,
                        content      text NOT NULL,
                        content_hash text NOT NULL,
                        embedding    vector({dim}) NOT NULL,
                        created_at   timestamptz NOT NULL DEFAULT now()
                    )"
                ),
                &[],
            )
            .context("failed to create knowledge_base table")?;

        self.client
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS kb_hash_idx
                 ON knowledge_base (content_hash)",
                &[],
            )
            .context("failed to create knowledge_base hash index")?;

        self.client
            .execute(
                "CREATE INDEX IF NOT EXISTS kb_coll_idx
                 ON knowledge_base (collection)",
                &[],
            )
            .context("failed to create knowledge_base collection index")?;

        self.client
            .execute(
                "CREATE INDEX IF NOT EXISTS kb_embed_idx
                 ON knowledge_base USING hnsw (embedding vector_cosine_ops)",
                &[],
            )
            .context("failed to create knowledge_base hnsw index")?;

        Ok(())
    }

    pub fn upsert_knowledge(
        &mut self,
        collection: &str,
        source_path: &str,
        content: &str,
        hash: &str,
        embedding: &[f32],
    ) -> Result<()> {
        let vec = Vector::from(embedding.to_vec());
        self.client
            .execute(
                "INSERT INTO knowledge_base (collection, source_path, content, content_hash, embedding)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (content_hash) DO NOTHING",
                &[&collection, &source_path, &content, &hash, &vec],
            )
            .context("failed to upsert knowledge_base row")?;
        Ok(())
    }

    pub fn search_knowledge(
        &mut self,
        query_embedding: &[f32],
        limit: i32,
        collection: Option<&str>,
    ) -> Result<Vec<KnowledgeResult>> {
        let vec = Vector::from(query_embedding.to_vec());
        let rows = match collection {
            Some(coll) => self.client.query(
                "SELECT collection, source_path, content,
                        (1.0 - (embedding <=> $1))::real AS score
                 FROM knowledge_base
                 WHERE collection = $3
                 ORDER BY embedding <=> $1
                 LIMIT $2",
                &[&vec, &limit, &coll],
            ),
            None => self.client.query(
                "SELECT collection, source_path, content,
                        (1.0 - (embedding <=> $1))::real AS score
                 FROM knowledge_base
                 ORDER BY embedding <=> $1
                 LIMIT $2",
                &[&vec, &limit],
            ),
        }
        .context("failed to query knowledge_base")?;

        Ok(rows
            .into_iter()
            .map(|row| KnowledgeResult {
                collection: row.get("collection"),
                source_path: row.get("source_path"),
                content: row.get("content"),
                score: row.get::<_, f32>("score"),
            })
            .collect())
    }

    pub fn list_collections(&mut self) -> Result<Vec<String>> {
        let rows = self
            .client
            .query(
                "SELECT DISTINCT collection FROM knowledge_base ORDER BY collection",
                &[],
            )
            .context("failed to list knowledge_base collections")?;
        Ok(rows.into_iter().map(|r| r.get::<_, String>(0)).collect())
    }

    pub fn existing_knowledge_dim(&mut self) -> Result<Option<usize>> {
        let rows = self
            .client
            .query(
                "SELECT atttypmod
                 FROM pg_attribute
                 JOIN pg_class ON pg_class.oid = pg_attribute.attrelid
                 WHERE pg_class.relname = 'knowledge_base'
                   AND pg_attribute.attname = 'embedding'
                   AND pg_attribute.attnum > 0",
                &[],
            )
            .context("failed to query knowledge_base schema")?;

        if let Some(row) = rows.first() {
            let typmod: i32 = row.get(0);
            if typmod > 4 {
                return Ok(Some((typmod - 4) as usize));
            }
        }
        Ok(None)
    }

    // ── project_memory dimension check ────────────────────────────────────────

    /// Return the vector dimension of the existing embedding column, or None if
    /// the table does not exist yet.
    pub fn existing_dim(&mut self) -> Result<Option<usize>> {
        let rows = self
            .client
            .query(
                "SELECT atttypmod
                 FROM pg_attribute
                 JOIN pg_class ON pg_class.oid = pg_attribute.attrelid
                 WHERE pg_class.relname = 'project_memory'
                   AND pg_attribute.attname = 'embedding'
                   AND pg_attribute.attnum > 0",
                &[],
            )
            .context("failed to query schema information")?;

        if let Some(row) = rows.first() {
            let typmod: i32 = row.get(0);
            // pgvector stores dim as (dim + 4) in typmod
            if typmod > 4 {
                return Ok(Some((typmod - 4) as usize));
            }
        }
        Ok(None)
    }
}
