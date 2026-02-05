use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
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

#[cfg(feature = "ssr")]
impl Flag {
    /// Create a flag for an idea or comment
    pub async fn create(user_id: i32, target_type: &str, target_id: i32) -> Result<(), sqlx::Error> {
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

    /// Check if a user has already flagged an item
    pub async fn has_flagged(user_id: i32, target_type: &str, target_id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM flags
                WHERE user_id = $1 AND target_type = $2 AND target_id = $3
            ) as "exists!"
            "#,
            user_id,
            target_type,
            target_id
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok(result)
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

    /// Get flag count for a specific item
    pub async fn get_flag_count(target_type: &str, target_id: i32) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM flags
            WHERE target_type = $1 AND target_id = $2
            "#,
            target_type,
            target_id
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok(count)
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

    /// Get users who flagged a specific item
    pub async fn get_flaggers(target_type: &str, target_id: i32) -> Result<Vec<crate::models::user::User>, sqlx::Error> {
        sqlx::query_as!(
            crate::models::user::User,
            r#"
            SELECT u.id, u.email, u.name, NULL as password_hash, u.role, u.created_on
            FROM users u
            INNER JOIN flags f ON u.id = f.user_id
            WHERE f.target_type = $1 AND f.target_id = $2
            ORDER BY f.created_at ASC
            "#,
            target_type,
            target_id
        )
        .fetch_all(crate::database::get_db())
        .await
    }
}
