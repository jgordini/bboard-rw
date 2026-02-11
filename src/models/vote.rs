use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Vote {
    pub id: i32,
    pub idea_id: i32,
    pub user_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Vote {
    #[cfg(feature = "ssr")]
    pub async fn create(user_id: i32, idea_id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Vote,
            "INSERT INTO votes (user_id, idea_id) VALUES ($1, $2) RETURNING id, idea_id, user_id, created_at",
            user_id,
            idea_id
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn delete(user_id: i32, idea_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM votes WHERE user_id = $1 AND idea_id = $2",
            user_id,
            idea_id
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    #[cfg(feature = "ssr")]
    pub async fn toggle(user_id: i32, idea_id: i32) -> Result<bool, sqlx::Error> {
        // Check if vote exists
        let has_voted = Self::has_voted(user_id, idea_id).await?;

        if has_voted {
            // Remove vote
            Self::delete(user_id, idea_id).await?;
            Ok(false)
        } else {
            // Add vote
            Self::create(user_id, idea_id).await?;
            Ok(true)
        }
    }

    #[cfg(feature = "ssr")]
    pub async fn has_voted(user_id: i32, idea_id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM votes WHERE user_id = $1 AND idea_id = $2
            ) as "exists!"
            "#,
            user_id,
            idea_id
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok(result)
    }

    #[cfg(feature = "ssr")]
    pub async fn get_voted_ideas(user_id: i32) -> Result<Vec<i32>, sqlx::Error> {
        let votes = sqlx::query!(
            "SELECT idea_id FROM votes WHERE user_id = $1",
            user_id
        )
        .fetch_all(crate::database::get_db())
        .await?;
        Ok(votes.into_iter().map(|v| v.idea_id).collect())
    }

}
