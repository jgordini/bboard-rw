#[cfg(feature = "ssr")]
mod inner {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
    pub struct Flag {
        pub id: i32,
        pub target_type: String, // 'idea' or 'comment'
        pub target_id: i32,
        pub user_id: i32,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FlaggedItem {
        pub target_type: String,
        pub target_id: i32,
        pub flag_count: i64,
        pub first_flagged: chrono::DateTime<chrono::Utc>,
    }

    impl Flag {
        /// Create a flag for an idea or comment
        pub async fn create(
            user_id: i32,
            target_type: &str,
            target_id: i32,
        ) -> Result<(), sqlx::Error> {
            sqlx::query!(
                r#"
                INSERT INTO flags (user_id, target_type, target_id)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id, target_type, target_id) DO NOTHING
                "#,
                user_id,
                target_type,
                target_id
            )
            .execute(crate::database::get_db())
            .await?;
            Ok(())
        }

        /// Get all flagged items with counts
        pub async fn get_flagged_items() -> Result<Vec<FlaggedItem>, sqlx::Error> {
            sqlx::query_as!(
                FlaggedItem,
                r#"
                SELECT
                    target_type,
                    target_id,
                    COUNT(*) as "flag_count!",
                    MIN(created_at) as "first_flagged!"
                FROM flags
                GROUP BY target_type, target_id
                ORDER BY COUNT(*) DESC, MIN(created_at) ASC
                "#
            )
            .fetch_all(crate::database::get_db())
            .await
        }

        /// Clear all flags for a specific item
        pub async fn clear_flags(target_type: &str, target_id: i32) -> Result<(), sqlx::Error> {
            sqlx::query!(
                "DELETE FROM flags WHERE target_type = $1 AND target_id = $2",
                target_type,
                target_id
            )
            .execute(crate::database::get_db())
            .await?;
            Ok(())
        }
    }
}

#[cfg(feature = "ssr")]
pub use inner::Flag;
