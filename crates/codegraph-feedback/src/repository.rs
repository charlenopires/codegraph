//! FeedbackRepository - PostgreSQL persistence for feedback records

use chrono::Utc;
use sqlx::PgPool;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::error::{FeedbackError, Result};
use crate::models::{CreateFeedback, Feedback, FeedbackMetrics, FeedbackSummary, FeedbackType};

/// Repository for persisting feedback in PostgreSQL
#[derive(Clone)]
pub struct FeedbackRepository {
    pool: PgPool,
}

impl FeedbackRepository {
    /// Create a new FeedbackRepository with the given database pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize the database schema for feedback storage
    #[instrument(skip(self))]
    pub async fn init_schema(&self) -> Result<()> {
        info!("Initializing feedback schema");

        // Create feedback_type enum if it doesn't exist
        sqlx::query(
            r#"
            DO $$ BEGIN
                CREATE TYPE feedback_type AS ENUM ('thumbs_up', 'thumbs_down');
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create feedback table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS feedback (
                id UUID PRIMARY KEY,
                generation_id UUID NOT NULL,
                element_ids JSONB NOT NULL DEFAULT '[]',
                feedback_type feedback_type NOT NULL,
                query_context TEXT,
                comment TEXT,
                confidence_delta REAL NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for efficient queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_feedback_generation_id ON feedback(generation_id);
            CREATE INDEX IF NOT EXISTS idx_feedback_created_at ON feedback(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_feedback_type ON feedback(feedback_type);
            CREATE INDEX IF NOT EXISTS idx_feedback_element_ids ON feedback USING GIN(element_ids);
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Feedback schema initialized successfully");
        Ok(())
    }

    /// Save new feedback to the database
    #[instrument(skip(self, feedback), fields(generation_id = %feedback.generation_id))]
    pub async fn save(&self, feedback: CreateFeedback) -> Result<Feedback> {
        let id = Uuid::new_v4();
        let confidence_delta = feedback.feedback_type.confidence_delta();
        let element_ids_json = serde_json::to_value(&feedback.element_ids)?;
        let created_at = Utc::now();

        debug!(
            feedback_id = %id,
            feedback_type = %feedback.feedback_type,
            confidence_delta = confidence_delta,
            element_count = feedback.element_ids.len(),
            "Saving feedback"
        );

        sqlx::query(
            r#"
            INSERT INTO feedback (id, generation_id, element_ids, feedback_type, query_context, comment, confidence_delta, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(feedback.generation_id)
        .bind(&element_ids_json)
        .bind(feedback.feedback_type.to_string())
        .bind(&feedback.query_context)
        .bind(&feedback.comment)
        .bind(confidence_delta)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        info!(
            feedback_id = %id,
            "Feedback saved successfully"
        );

        // Record metrics
        metrics::counter!("feedback_total").increment(1);
        if feedback.feedback_type.is_positive() {
            metrics::counter!("feedback_positive").increment(1);
        } else {
            metrics::counter!("feedback_negative").increment(1);
        }

        Ok(Feedback {
            id,
            generation_id: feedback.generation_id,
            element_ids: feedback.element_ids,
            feedback_type: feedback.feedback_type,
            query_context: feedback.query_context,
            comment: feedback.comment,
            confidence_delta,
            created_at,
        })
    }

    /// Find feedback by ID
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Feedback>> {
        let row = sqlx::query_as::<_, FeedbackRow>(
            r#"
            SELECT id, generation_id, element_ids, feedback_type, query_context, comment, confidence_delta, created_at
            FROM feedback
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Find all feedback for a generation
    #[instrument(skip(self))]
    pub async fn find_by_generation(&self, generation_id: Uuid) -> Result<Vec<Feedback>> {
        let rows = sqlx::query_as::<_, FeedbackRow>(
            r#"
            SELECT id, generation_id, element_ids, feedback_type, query_context, comment, confidence_delta, created_at
            FROM feedback
            WHERE generation_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(generation_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>>>()
    }

    /// Find all feedback that includes a specific element
    #[instrument(skip(self))]
    pub async fn find_by_element(&self, element_id: Uuid) -> Result<Vec<Feedback>> {
        let element_json = serde_json::json!([element_id.to_string()]);

        let rows = sqlx::query_as::<_, FeedbackRow>(
            r#"
            SELECT id, generation_id, element_ids, feedback_type, query_context, comment, confidence_delta, created_at
            FROM feedback
            WHERE element_ids @> $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(&element_json)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>>>()
    }

    /// Get feedback summary for an element
    #[instrument(skip(self))]
    pub async fn get_element_summary(&self, element_id: Uuid) -> Result<FeedbackSummary> {
        let element_json = serde_json::json!([element_id.to_string()]);

        let row = sqlx::query_as::<_, SummaryRow>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE feedback_type = 'thumbs_up') as positive_count,
                COUNT(*) FILTER (WHERE feedback_type = 'thumbs_down') as negative_count,
                COALESCE(SUM(confidence_delta), 0) as net_confidence_delta,
                MAX(created_at) as last_feedback_at
            FROM feedback
            WHERE element_ids @> $1
            "#,
        )
        .bind(&element_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(FeedbackSummary {
            element_id,
            positive_count: row.positive_count.unwrap_or(0),
            negative_count: row.negative_count.unwrap_or(0),
            net_confidence_delta: row.net_confidence_delta.unwrap_or(0.0),
            last_feedback_at: row.last_feedback_at,
        })
    }

    /// Get recent feedback with pagination
    #[instrument(skip(self))]
    pub async fn list_recent(&self, limit: i64, offset: i64) -> Result<Vec<Feedback>> {
        let rows = sqlx::query_as::<_, FeedbackRow>(
            r#"
            SELECT id, generation_id, element_ids, feedback_type, query_context, comment, confidence_delta, created_at
            FROM feedback
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>>>()
    }

    /// Get aggregated feedback metrics
    #[instrument(skip(self))]
    pub async fn get_metrics(&self) -> Result<FeedbackMetrics> {
        let row = sqlx::query_as::<_, MetricsRow>(
            r#"
            SELECT
                COUNT(*) as total_feedback,
                COUNT(*) FILTER (WHERE feedback_type = 'thumbs_up') as positive_count,
                COUNT(*) FILTER (WHERE feedback_type = 'thumbs_down') as negative_count,
                COALESCE(AVG(confidence_delta), 0) as avg_confidence_delta
            FROM feedback
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total = row.total_feedback.unwrap_or(0) as f64;
        let positive = row.positive_count.unwrap_or(0);
        let negative = row.negative_count.unwrap_or(0);

        let positive_ratio = if total > 0.0 {
            positive as f64 / total
        } else {
            0.0
        };

        let negative_ratio = if total > 0.0 {
            negative as f64 / total
        } else {
            0.0
        };

        // Update Prometheus gauges
        metrics::gauge!("feedback_positive_ratio").set(positive_ratio);
        metrics::gauge!("feedback_negative_ratio").set(negative_ratio);
        metrics::gauge!("feedback_avg_confidence_delta").set(row.avg_confidence_delta.unwrap_or(0.0));

        Ok(FeedbackMetrics {
            total_feedback: row.total_feedback.unwrap_or(0),
            positive_count: positive,
            negative_count: negative,
            positive_ratio,
            negative_ratio,
            avg_confidence_delta: row.avg_confidence_delta.unwrap_or(0.0),
        })
    }

    /// Delete old feedback records (for data retention)
    #[instrument(skip(self))]
    pub async fn delete_older_than(&self, days: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM feedback
            WHERE created_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            info!(deleted_count = deleted, "Deleted old feedback records");
        }

        Ok(deleted)
    }
}

// Internal row types for SQLx mapping

#[derive(sqlx::FromRow)]
struct FeedbackRow {
    id: Uuid,
    generation_id: Uuid,
    element_ids: serde_json::Value,
    feedback_type: String,
    query_context: Option<String>,
    comment: Option<String>,
    confidence_delta: f32,
    created_at: chrono::DateTime<Utc>,
}

impl TryFrom<FeedbackRow> for Feedback {
    type Error = FeedbackError;

    fn try_from(row: FeedbackRow) -> std::result::Result<Self, Self::Error> {
        let element_ids: Vec<Uuid> = serde_json::from_value(row.element_ids)?;

        let feedback_type = match row.feedback_type.as_str() {
            "thumbs_up" => FeedbackType::ThumbsUp,
            "thumbs_down" => FeedbackType::ThumbsDown,
            other => return Err(FeedbackError::InvalidFeedbackType(other.to_string())),
        };

        Ok(Feedback {
            id: row.id,
            generation_id: row.generation_id,
            element_ids,
            feedback_type,
            query_context: row.query_context,
            comment: row.comment,
            confidence_delta: row.confidence_delta,
            created_at: row.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct SummaryRow {
    positive_count: Option<i64>,
    negative_count: Option<i64>,
    net_confidence_delta: Option<f32>,
    last_feedback_at: Option<chrono::DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct MetricsRow {
    total_feedback: Option<i64>,
    positive_count: Option<i64>,
    negative_count: Option<i64>,
    avg_confidence_delta: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_type_delta() {
        assert_eq!(FeedbackType::ThumbsUp.confidence_delta(), 0.1);
        assert_eq!(FeedbackType::ThumbsDown.confidence_delta(), -0.15);
    }

    #[test]
    fn test_feedback_type_is_positive() {
        assert!(FeedbackType::ThumbsUp.is_positive());
        assert!(!FeedbackType::ThumbsDown.is_positive());
    }

    #[test]
    fn test_feedback_type_display() {
        assert_eq!(FeedbackType::ThumbsUp.to_string(), "thumbs_up");
        assert_eq!(FeedbackType::ThumbsDown.to_string(), "thumbs_down");
    }
}
