use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing)]
    #[cfg_attr(feature = "ssr", sqlx(skip))]
    pub password_hash: Option<String>,
    pub role: i16, // 0: User, 1: Moderator, 2: Admin
    pub created_on: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum Role {
    User = 0,
    Moderator = 1,
    Admin = 2,
}

impl Role {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            0 => Some(Role::User),
            1 => Some(Role::Moderator),
            2 => Some(Role::Admin),
            _ => None,
        }
    }
}

impl User {
    pub fn is_moderator(&self) -> bool {
        self.role >= 1
    }

    pub fn is_admin(&self) -> bool {
        self.role >= 2
    }
}

#[cfg(feature = "ssr")]
impl User {
    /// Create a new user with hashed password
    pub async fn create(email: String, name: String, password: String) -> Result<Self, sqlx::Error> {
        use bcrypt::{hash, DEFAULT_COST};

        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| sqlx::Error::Protocol(format!("Password hashing failed: {}", e)))?;

        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, name, password_hash, role)
            VALUES ($1, $2, $3, 0)
            RETURNING id, email, name, NULL as password_hash, role, created_on
            "#,
            email,
            name,
            password_hash
        )
        .fetch_one(crate::database::get_db())
        .await
    }

    /// Authenticate a user with email and password
    pub async fn authenticate(email: String, password: String) -> Result<Option<Self>, sqlx::Error> {
        use bcrypt::verify;

        // First, get the user with password hash
        let result = sqlx::query!(
            "SELECT id, email, name, password_hash, role, created_on FROM users WHERE email = $1",
            email
        )
        .fetch_optional(crate::database::get_db())
        .await?;

        match result {
            Some(record) => {
                // Verify password
                let password_matches = verify(password, &record.password_hash)
                    .map_err(|e| sqlx::Error::Protocol(format!("Password verification failed: {}", e)))?;

                if password_matches {
                    Ok(Some(User {
                        id: record.id,
                        email: record.email,
                        name: record.name,
                        password_hash: None, // Don't include hash in returned user
                        role: record.role,
                        created_on: record.created_on,
                    }))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Get user by ID
    pub async fn get_by_id(id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, NULL as password_hash, role, created_on
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(crate::database::get_db())
        .await
    }

    /// Get user by email
    pub async fn get_by_email(email: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, NULL as password_hash, role, created_on
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(crate::database::get_db())
        .await
    }

    /// Update user role (admin only). Cannot change an admin's role.
    pub async fn update_role(id: i32, role: i16) -> Result<(), sqlx::Error> {
        let target = Self::get_by_id(id).await?;
        let Some(u) = target else {
            return Err(sqlx::Error::RowNotFound);
        };
        if u.role >= 2 {
            return Err(sqlx::Error::Protocol("Cannot change an admin's role".into()));
        }
        sqlx::query!(
            "UPDATE users SET role = $1 WHERE id = $2",
            role,
            id
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    /// Delete a user (admin only). Cannot delete any admin.
    pub async fn delete(id: i32) -> Result<(), sqlx::Error> {
        let target = Self::get_by_id(id).await?;
        let Some(u) = target else {
            return Err(sqlx::Error::RowNotFound);
        };
        if u.role >= 2 {
            return Err(sqlx::Error::Protocol("Cannot delete an admin".into()));
        }
        sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(crate::database::get_db())
            .await?;
        Ok(())
    }

    /// Update user name
    pub async fn update_name(id: i32, name: String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET name = $1 WHERE id = $2",
            name,
            id
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    /// Update user password
    pub async fn update_password(id: i32, new_password: String) -> Result<(), sqlx::Error> {
        use bcrypt::{hash, DEFAULT_COST};

        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| sqlx::Error::Protocol(format!("Password hashing failed: {}", e)))?;

        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            password_hash,
            id
        )
        .execute(crate::database::get_db())
        .await?;
        Ok(())
    }

    /// Bootstrap admin user from environment variables
    /// Creates admin if no admin exists
    pub async fn bootstrap_admin() -> Result<(), sqlx::Error> {
        // Check if any admin exists
        let admin_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE role = 2"
        )
        .fetch_one(crate::database::get_db())
        .await?
        .unwrap_or(0);

        if admin_count > 0 {
            return Ok(()); // Admin already exists
        }

        // Get credentials from environment
        let email = std::env::var("INITIAL_ADMIN_EMAIL").unwrap_or_else(|_| "admin".to_string());
        let password = std::env::var("INITIAL_ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

        // Warn if using default credentials
        if email == "admin" && password == "admin" {
            eprintln!("⚠️  WARNING: Using default admin credentials (admin/admin). Please change these in production!");
        }

        // Create admin user
        use bcrypt::{hash, DEFAULT_COST};
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| sqlx::Error::Protocol(format!("Password hashing failed: {}", e)))?;

        sqlx::query!(
            "INSERT INTO users (email, name, password_hash, role) VALUES ($1, $2, $3, 2)",
            email,
            "Administrator",
            password_hash
        )
        .execute(crate::database::get_db())
        .await?;

        println!("✅ Admin user created: {}", email);
        Ok(())
    }

    /// Get all users (admin only)
    pub async fn get_all() -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, NULL as password_hash, role, created_on
            FROM users
            ORDER BY created_on DESC
            "#
        )
        .fetch_all(crate::database::get_db())
        .await
    }

    /// Get user's ideas
    pub async fn get_ideas(&self) -> Result<Vec<crate::models::idea::Idea>, sqlx::Error> {
        crate::models::idea::Idea::get_by_user(self.id).await
    }
}
