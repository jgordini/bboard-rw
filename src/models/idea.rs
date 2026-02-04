use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Idea {
    pub id: i32,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub vote_count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdeaForm {
    pub content: String,
    pub captcha_token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteForm {
    pub idea_id: i32,
    pub voter_fingerprint: String,
}

impl Idea {
    #[cfg(feature = "ssr")]
    pub async fn get_by_id(id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Idea,
            "SELECT id, content, created_at, vote_count FROM ideas WHERE id = $1",
            id
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn get_all() -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Idea,
            "SELECT id, content, created_at, vote_count FROM ideas ORDER BY vote_count DESC, created_at DESC"
        )
        .fetch_all(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn create(content: String) -> Result<Self, sqlx::Error> {
        let idea = sqlx::query_as!(
            Idea,
            "INSERT INTO ideas (content) VALUES ($1) RETURNING id, content, created_at, vote_count",
            content
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok(idea)
    }

    #[cfg(feature = "ssr")]
    pub async fn delete(id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ideas WHERE id = $1", id)
            .execute(crate::database::get_db())
            .await?;
        Ok(())
    }

    #[cfg(feature = "ssr")]
    pub async fn delete_all() -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ideas")
            .execute(crate::database::get_db())
            .await?;
        Ok(())
    }

    #[cfg(feature = "ssr")]
    pub async fn delete_older_than_days(days: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ideas WHERE created_at < NOW() - INTERVAL '1 day' * $1", days as i64)
            .execute(crate::database::get_db())
            .await?;
        Ok(())
    }

    #[cfg(feature = "ssr")]
    pub async fn get_statistics() -> Result<(i64, i64), sqlx::Error> {
        let stats = sqlx::query!(
            "SELECT 
                COUNT(*) as total_ideas,
                COALESCE(SUM(vote_count), 0) as total_votes
            FROM ideas"
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok((
            stats.total_ideas.unwrap_or(0),
            stats.total_votes.unwrap_or(0),
        ))
    }
}
