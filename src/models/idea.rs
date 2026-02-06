use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Idea {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub tags: String,
    pub stage: String,
    pub is_public: bool,
    pub is_off_topic: bool,
    pub pinned_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub vote_count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdeaWithAuthor {
    pub idea: Idea,
    pub author_name: String,
    pub author_email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdeaForm {
    pub title: String,
    pub content: String,
}

// Valid stage values
pub const STAGES: [&str; 4] = ["Ideate", "Review", "In Progress", "Completed"];

impl Idea {
    pub fn is_pinned(&self) -> bool {
        self.pinned_at.is_some()
    }

    pub fn is_valid_stage(stage: &str) -> bool {
        STAGES.contains(&stage)
    }
}

#[cfg(feature = "ssr")]
impl Idea {
    /// Get idea by ID (only public and not off-topic for regular users)
    pub async fn get_by_id(id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Idea,
            r#"
            SELECT id, user_id, title, content, tags, stage, is_public, is_off_topic,
                   pinned_at, created_at, vote_count
            FROM ideas
            WHERE id = $1 AND is_public = true AND is_off_topic = false
            "#,
            id
        )
        .fetch_optional(crate::database::get_db())
        .await
    }

    /// Get idea by ID (including hidden items for moderators)
    pub async fn get_by_id_mod(id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Idea,
            r#"
            SELECT id, user_id, title, content, tags, stage, is_public, is_off_topic,
                   pinned_at, created_at, vote_count
            FROM ideas
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(crate::database::get_db())
        .await
    }

    /// Get idea with author information
    pub async fn get_with_author(id: i32) -> Result<Option<IdeaWithAuthor>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT
                i.id, i.user_id, i.title, i.content, i.tags, i.stage, i.is_public, i.is_off_topic,
                i.pinned_at, i.created_at, i.vote_count,
                u.name as author_name, u.email as author_email
            FROM ideas i
            INNER JOIN users u ON i.user_id = u.id
            WHERE i.id = $1 AND i.is_public = true AND i.is_off_topic = false
            "#,
            id
        )
        .fetch_optional(crate::database::get_db())
        .await?;

        Ok(result.map(|r| IdeaWithAuthor {
            idea: Idea {
                id: r.id,
                user_id: r.user_id,
                title: r.title,
                content: r.content,
                tags: r.tags.clone(),
                stage: r.stage,
                is_public: r.is_public,
                is_off_topic: r.is_off_topic,
                pinned_at: r.pinned_at,
                created_at: r.created_at,
                vote_count: r.vote_count,
            },
            author_name: r.author_name,
            author_email: r.author_email,
        }))
    }

    /// Get all public ideas (not off-topic)
    pub async fn get_all() -> Result<Vec<IdeaWithAuthor>, sqlx::Error> {
        let results = sqlx::query!(
            r#"
            SELECT
                i.id, i.user_id, i.title, i.content, i.tags, i.stage, i.is_public, i.is_off_topic,
                i.pinned_at, i.created_at, i.vote_count,
                u.name as author_name, u.email as author_email
            FROM ideas i
            INNER JOIN users u ON i.user_id = u.id
            WHERE i.is_public = true AND i.is_off_topic = false
            ORDER BY
                (CASE WHEN i.pinned_at IS NOT NULL THEN 0 ELSE 1 END),
                i.pinned_at DESC NULLS LAST,
                i.vote_count DESC,
                i.created_at DESC
            "#
        )
        .fetch_all(crate::database::get_db())
        .await?;

        Ok(results
            .into_iter()
            .map(|r| IdeaWithAuthor {
                idea: Idea {
                    id: r.id,
                    user_id: r.user_id,
                    title: r.title,
                    content: r.content,
                    tags: r.tags.clone(),
                    stage: r.stage,
                    is_public: r.is_public,
                    is_off_topic: r.is_off_topic,
                    pinned_at: r.pinned_at,
                    created_at: r.created_at,
                    vote_count: r.vote_count,
                },
                author_name: r.author_name,
                author_email: r.author_email,
            })
            .collect())
    }

    /// Get ideas by user (for profile page)
    pub async fn get_by_user(user_id: i32) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Idea,
            r#"
            SELECT id, user_id, title, content, tags, stage, is_public, is_off_topic,
                   pinned_at, created_at, vote_count
            FROM ideas
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(crate::database::get_db())
        .await
    }

    /// Get off-topic ideas (moderator view)
    pub async fn get_off_topic() -> Result<Vec<IdeaWithAuthor>, sqlx::Error> {
        let results = sqlx::query!(
            r#"
            SELECT
                i.id, i.user_id, i.title, i.content, i.tags, i.stage, i.is_public, i.is_off_topic,
                i.pinned_at, i.created_at, i.vote_count,
                u.name as author_name, u.email as author_email
            FROM ideas i
            INNER JOIN users u ON i.user_id = u.id
            WHERE i.is_off_topic = true
            ORDER BY i.created_at DESC
            "#
        )
        .fetch_all(crate::database::get_db())
        .await?;

        Ok(results
            .into_iter()
            .map(|r| IdeaWithAuthor {
                idea: Idea {
                    id: r.id,
                    user_id: r.user_id,
                    title: r.title,
                    content: r.content,
                    tags: r.tags.clone(),
                    stage: r.stage,
                    is_public: r.is_public,
                    is_off_topic: r.is_off_topic,
                    pinned_at: r.pinned_at,
                    created_at: r.created_at,
                    vote_count: r.vote_count,
                },
                author_name: r.author_name,
                author_email: r.author_email,
            })
            .collect())
    }

    /// Create a new idea
    pub async fn create(user_id: i32, title: String, content: String, tags: String) -> Result<Self, sqlx::Error> {
        let tags_trimmed = tags.trim().to_string();
        sqlx::query_as!(
            Idea,
            r#"
            INSERT INTO ideas (user_id, title, content, tags, stage, is_public, is_off_topic)
            VALUES ($1, $2, $3, $4, 'Ideate', true, false)
            RETURNING id, user_id, title, content, tags, stage, is_public, is_off_topic,
                      pinned_at, created_at, vote_count
            "#,
            user_id,
            title,
            content,
            tags_trimmed
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    /// Update idea content (author only)
    pub async fn update_content(id: i32, user_id: i32, title: String, content: String) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE ideas SET title = $1, content = $2 WHERE id = $3 AND user_id = $4",
            title,
            content,
            id,
            user_id
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update idea content (moderator/admin)
    pub async fn update_content_mod(id: i32, title: String, content: String) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE ideas SET title = $1, content = $2 WHERE id = $3",
            title,
            content,
            id
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update idea tags (moderator/admin)
    pub async fn update_tags_mod(id: i32, tags: String) -> Result<bool, sqlx::Error> {
        let tags_trimmed = tags.trim().to_string();
        let result = sqlx::query!(
            "UPDATE ideas SET tags = $1 WHERE id = $2",
            tags_trimmed,
            id
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update idea stage (moderator only)
    pub async fn update_stage(id: i32, stage: String) -> Result<(), sqlx::Error> {
        if !Self::is_valid_stage(&stage) {
            return Err(sqlx::Error::Protocol(format!("Invalid stage: {}", stage)));
        }

        sqlx::query!(
            "UPDATE ideas SET stage = $1 WHERE id = $2",
            stage,
            id
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    /// Toggle pinned status (moderator only)
    pub async fn toggle_pin(id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE ideas
            SET pinned_at = CASE
                WHEN pinned_at IS NULL THEN NOW()
                ELSE NULL
            END
            WHERE id = $1
            RETURNING pinned_at IS NOT NULL as "is_pinned!"
            "#,
            id
        )
        .fetch_one(crate::database::get_db())
        .await?;

        Ok(result.is_pinned)
    }

    /// Mark idea as off-topic (moderator only)
    pub async fn mark_off_topic(id: i32, is_off_topic: bool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE ideas SET is_off_topic = $1 WHERE id = $2",
            is_off_topic,
            id
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    /// Delete idea (moderator only)
    pub async fn delete(id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ideas WHERE id = $1", id)
            .execute(crate::database::get_db())
            .await?;
        Ok(())
    }

    /// Delete all ideas (admin only)
    pub async fn delete_all() -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM ideas")
            .execute(crate::database::get_db())
            .await?;
        Ok(())
    }

    /// Delete ideas older than N days (admin only)
    pub async fn delete_older_than_days(days: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM ideas WHERE created_at < NOW() - INTERVAL '1 day' * $1",
            days as i64
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    /// Get statistics
    pub async fn get_statistics() -> Result<(i64, i64), sqlx::Error> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_ideas,
                COALESCE(SUM(vote_count), 0) as total_votes
            FROM ideas
            WHERE is_public = true AND is_off_topic = false
            "#
        )
        .fetch_one(crate::database::get_db())
        .await?;
        Ok((
            stats.total_ideas.unwrap_or(0),
            stats.total_votes.unwrap_or(0),
        ))
    }
}
