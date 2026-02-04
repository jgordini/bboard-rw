use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Comment {
    pub id: i32,
    pub idea_id: i32,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Comment {
    #[cfg(feature = "ssr")]
    pub async fn create(idea_id: i32, content: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Comment,
            "INSERT INTO comments (idea_id, content) VALUES ($1, $2) RETURNING id, idea_id, content, created_at",
            idea_id,
            content
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn get_by_idea_id(idea_id: i32) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Comment,
            "SELECT id, idea_id, content, created_at FROM comments WHERE idea_id = $1 ORDER BY created_at ASC",
            idea_id
        )
        .fetch_all(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn count_by_idea_id(idea_id: i32) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM comments WHERE idea_id = $1",
            idea_id
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok(result.count.unwrap_or(0))
    }

    #[cfg(feature = "ssr")]
    pub async fn count_all_grouped() -> Result<Vec<(i32, i64)>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT idea_id, COUNT(*) as count FROM comments GROUP BY idea_id"
        )
        .fetch_all(crate::database::get_db())
        .await?;
        Ok(rows.into_iter().map(|r| (r.idea_id, r.count.unwrap_or(0))).collect())
    }
}
