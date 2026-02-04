use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vote {
    pub id: i32,
    pub idea_id: i32,
    pub voter_fingerprint: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteForm {
    pub idea_id: i32,
    pub voter_fingerprint: String,
}

impl Vote {
    #[cfg(feature = "ssr")]
    pub async fn create(idea_id: i32, voter_fingerprint: String) -> Result<Self, sqlx::Error> {
        let vote = sqlx::query_as!(
            Vote,
            "INSERT INTO votes (idea_id, voter_fingerprint) VALUES ($1, $2) 
             RETURNING id, idea_id, voter_fingerprint, created_at",
            idea_id,
            voter_fingerprint
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok(vote)
    }

    #[cfg(feature = "ssr")]
    pub async fn has_voted(idea_id: i32, voter_fingerprint: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM votes WHERE idea_id = $1 AND voter_fingerprint = $2) as exists",
            idea_id,
            voter_fingerprint
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok(result.exists.unwrap_or(false))
    }

    #[cfg(feature = "ssr")]
    pub async fn get_voted_ideas(voter_fingerprint: &str) -> Result<Vec<i32>, sqlx::Error> {
        let votes = sqlx::query!(
            "SELECT idea_id FROM votes WHERE voter_fingerprint = $1",
            voter_fingerprint
        )
        .fetch_all(crate::database::get_db())
        .await?;
        Ok(votes.into_iter().map(|v| v.idea_id).collect())
    }
}
