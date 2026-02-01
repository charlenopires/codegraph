//! Redis cache for embeddings
//!
//! Caches embeddings with a 1-hour TTL to reduce load on
//! embedding generation and Qdrant queries.

use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::debug;
use uuid::Uuid;

use crate::error::Result;

/// Default TTL for cached embeddings (1 hour)
pub const DEFAULT_TTL_SECS: u64 = 3600;

/// Cache key prefix for embeddings
pub const CACHE_PREFIX: &str = "codegraph:embedding:";

/// Redis-based embedding cache
#[derive(Clone)]
pub struct EmbeddingCache {
    /// Redis connection manager
    conn: ConnectionManager,
    /// TTL in seconds
    ttl: u64,
}

impl EmbeddingCache {
    /// Create a new cache with Redis connection
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self {
            conn,
            ttl: DEFAULT_TTL_SECS,
        })
    }

    /// Create cache with custom TTL
    pub async fn with_ttl(redis_url: &str, ttl_secs: u64) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self { conn, ttl: ttl_secs })
    }

    /// Get an embedding from cache
    pub async fn get_embedding(&mut self, id: Uuid) -> Result<Option<Vec<f32>>> {
        let key = format!("{}{}", CACHE_PREFIX, id);

        let result: Option<String> = self.conn.get(&key).await?;

        match result {
            Some(json) => {
                let embedding: Vec<f32> = serde_json::from_str(&json)?;
                debug!(id = %id, "Cache hit for embedding");
                metrics::counter!("vector_cache_hits").increment(1);
                Ok(Some(embedding))
            }
            None => {
                debug!(id = %id, "Cache miss for embedding");
                metrics::counter!("vector_cache_misses").increment(1);
                Ok(None)
            }
        }
    }

    /// Store an embedding in cache
    pub async fn set_embedding(&mut self, id: Uuid, embedding: &[f32]) -> Result<()> {
        let key = format!("{}{}", CACHE_PREFIX, id);
        let json = serde_json::to_string(embedding)?;

        let _: () = self.conn.set_ex(&key, &json, self.ttl).await?;

        debug!(id = %id, ttl = self.ttl, "Cached embedding");
        metrics::counter!("vector_cache_sets").increment(1);

        Ok(())
    }

    /// Get or compute an embedding
    ///
    /// Returns cached value if available, otherwise calls the compute function
    /// and caches the result.
    pub async fn get_or_compute<F, Fut>(
        &mut self,
        id: Uuid,
        compute: F,
    ) -> Result<Vec<f32>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<f32>>>,
    {
        // Try cache first
        if let Some(embedding) = self.get_embedding(id).await? {
            return Ok(embedding);
        }

        // Compute and cache
        let embedding = compute().await?;
        self.set_embedding(id, &embedding).await?;

        Ok(embedding)
    }

    /// Delete an embedding from cache
    pub async fn delete_embedding(&mut self, id: Uuid) -> Result<bool> {
        let key = format!("{}{}", CACHE_PREFIX, id);
        let deleted: i64 = self.conn.del(&key).await?;

        if deleted > 0 {
            debug!(id = %id, "Deleted cached embedding");
        }

        Ok(deleted > 0)
    }

    /// Delete multiple embeddings from cache
    pub async fn delete_many(&mut self, ids: &[Uuid]) -> Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let keys: Vec<String> = ids
            .iter()
            .map(|id| format!("{}{}", CACHE_PREFIX, id))
            .collect();

        let deleted: u64 = self.conn.del(&keys).await?;

        debug!(deleted = deleted, "Deleted cached embeddings");

        Ok(deleted)
    }

    /// Check if an embedding is cached
    pub async fn exists(&mut self, id: Uuid) -> Result<bool> {
        let key = format!("{}{}", CACHE_PREFIX, id);
        let exists: bool = self.conn.exists(&key).await?;
        Ok(exists)
    }

    /// Get the current TTL
    pub fn ttl(&self) -> u64 {
        self.ttl
    }

    /// Get cache stats
    pub async fn stats(&mut self) -> Result<CacheStats> {
        // Use INFO command to get stats
        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut self.conn)
            .await?;

        // Parse basic stats (simplified)
        let hits = parse_info_value(&info, "keyspace_hits").unwrap_or(0);
        let misses = parse_info_value(&info, "keyspace_misses").unwrap_or(0);

        Ok(CacheStats {
            hits,
            misses,
            hit_ratio: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        })
    }

    /// Clear all cached embeddings
    pub async fn clear(&mut self) -> Result<u64> {
        // Scan for all keys with our prefix and delete them
        let mut cursor = 0u64;
        let mut total_deleted = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(format!("{}*", CACHE_PREFIX))
                .arg("COUNT")
                .arg(100)
                .query_async(&mut self.conn)
                .await?;

            if !keys.is_empty() {
                let deleted: u64 = self.conn.del(&keys).await?;
                total_deleted += deleted;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        debug!(deleted = total_deleted, "Cleared embedding cache");

        Ok(total_deleted)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Hit ratio (0.0 - 1.0)
    pub hit_ratio: f64,
}

/// Parse a value from Redis INFO output
fn parse_info_value(info: &str, key: &str) -> Option<u64> {
    for line in info.lines() {
        if line.starts_with(key) {
            if let Some(value) = line.split(':').nth(1) {
                return value.trim().parse().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_format() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = format!("{}{}", CACHE_PREFIX, id);
        assert_eq!(key, "codegraph:embedding:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_parse_info_value() {
        let info = "keyspace_hits:12345\nkeyspace_misses:6789\n";
        assert_eq!(parse_info_value(info, "keyspace_hits"), Some(12345));
        assert_eq!(parse_info_value(info, "keyspace_misses"), Some(6789));
        assert_eq!(parse_info_value(info, "nonexistent"), None);
    }
}
