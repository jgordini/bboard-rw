static DB: std::sync::OnceLock<sqlx::PgPool> = std::sync::OnceLock::new();

async fn create_pool() -> Result<sqlx::PgPool, String> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|e| format!("DATABASE_URL is not set or invalid: {e}"))?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(database_url.as_str())
        .await
        .map_err(|e| format!("Failed to connect to database: {e}"))?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| format!("Database migrations failed: {e}"))?;

    Ok(pool)
}

pub async fn init_db() -> Result<(), String> {
    let pool = create_pool().await?;
    DB.set(pool)
        .map_err(|_| "Database pool was already initialized".to_string())
}

pub fn get_db<'a>() -> &'a sqlx::PgPool {
    DB.get().expect("database unitialized")
}
