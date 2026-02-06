use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Comment {
    pub id: i32,
    pub idea_id: i32,
    pub user_id: i32,
    pub content: String,
    pub is_pinned: bool,
    pub is_deleted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommentWithAuthor {
    pub comment: Comment,
    pub author_name: String,
    pub author_email: String,
    pub is_idea_author: bool,
}

impl Comment {
    #[cfg(feature = "ssr")]
    pub async fn create(user_id: i32, idea_id: i32, content: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Comment,
            r#"
            INSERT INTO comments (user_id, idea_id, content)
            VALUES ($1, $2, $3)
            RETURNING id, idea_id, user_id, content, is_pinned, is_deleted, created_at
            "#,
            user_id,
            idea_id,
            content
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn get_by_idea_id(idea_id: i32, include_deleted: bool) -> Result<Vec<CommentWithAuthor>, sqlx::Error> {
        let delete_filter = if include_deleted { "" } else { "AND c.is_deleted = false" };

        let query_str = format!(
            r#"
            SELECT
                c.id, c.idea_id, c.user_id, c.content, c.is_pinned, c.is_deleted, c.created_at,
                u.name as author_name, u.email as author_email,
                (i.user_id = c.user_id) as is_idea_author
            FROM comments c
            INNER JOIN users u ON c.user_id = u.id
            INNER JOIN ideas i ON c.idea_id = i.id
            WHERE c.idea_id = $1 {}
            ORDER BY
                (CASE WHEN c.is_pinned THEN 0 ELSE 1 END),
                c.created_at ASC
            "#,
            delete_filter
        );

        let rows = sqlx::query(&query_str)
            .bind(idea_id)
            .fetch_all(crate::database::get_db())
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                CommentWithAuthor {
                    comment: Comment {
                        id: row.get("id"),
                        idea_id: row.get("idea_id"),
                        user_id: row.get("user_id"),
                        content: row.get("content"),
                        is_pinned: row.get("is_pinned"),
                        is_deleted: row.get("is_deleted"),
                        created_at: row.get("created_at"),
                    },
                    author_name: row.get("author_name"),
                    author_email: row.get("author_email"),
                    is_idea_author: row.get("is_idea_author"),
                }
            })
            .collect())
    }

    #[cfg(feature = "ssr")]
    pub async fn update_content(id: i32, user_id: i32, content: String) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE comments SET content = $1 WHERE id = $2 AND user_id = $3",
            content,
            id,
            user_id
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    #[cfg(feature = "ssr")]
    pub async fn update_content_mod(id: i32, content: String) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE comments SET content = $1 WHERE id = $2",
            content,
            id
        )
        .execute(crate::database::get_db())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    #[cfg(feature = "ssr")]
    pub async fn toggle_pin(id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE comments
            SET is_pinned = NOT is_pinned
            WHERE id = $1
            RETURNING is_pinned as "is_pinned!"
            "#,
            id
        )
        .fetch_one(crate::database::get_db())
        .await?;

        Ok(result.is_pinned)
    }

    #[cfg(feature = "ssr")]
    pub async fn soft_delete(id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE comments SET is_deleted = true WHERE id = $1",
            id
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    #[cfg(feature = "ssr")]
    pub async fn hard_delete(id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM comments WHERE id = $1", id)
            .execute(crate::database::get_db())
            .await?;
        Ok(())
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
        Ok(rows
            .into_iter()
            .map(|r| (r.idea_id, r.count.unwrap_or(0)))
            .collect())
    }

    #[cfg(feature = "ssr")]
    pub async fn get_by_user(user_id: i32) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Comment,
            r#"
            SELECT id, idea_id, user_id, content, is_pinned, is_deleted, created_at
            FROM comments
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(crate::database::get_db())
        .await
    }
}
