-- Rollback User Authentication System Migration
-- WARNING: This will delete all user data

-- 1. Restore admin_users table
CREATE TABLE IF NOT EXISTS admin_users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Remove columns from comments
ALTER TABLE comments DROP COLUMN IF EXISTS is_deleted;
ALTER TABLE comments DROP COLUMN IF EXISTS is_pinned;
ALTER TABLE comments DROP COLUMN IF EXISTS user_id;

DROP INDEX IF EXISTS idx_comments_is_deleted;
DROP INDEX IF EXISTS idx_comments_is_pinned;
DROP INDEX IF EXISTS idx_comments_user_id;

-- 3. Restore voter_fingerprint to votes
ALTER TABLE votes DROP CONSTRAINT IF EXISTS votes_user_id_idea_id_unique;
ALTER TABLE votes DROP COLUMN IF EXISTS user_id;
ALTER TABLE votes ADD COLUMN voter_fingerprint TEXT;

DROP INDEX IF EXISTS idx_votes_user_id;

-- Restore original unique constraint
ALTER TABLE votes ADD CONSTRAINT votes_idea_id_voter_fingerprint_unique UNIQUE(idea_id, voter_fingerprint);

-- 4. Remove columns from ideas
ALTER TABLE ideas DROP COLUMN IF EXISTS pinned_at;
ALTER TABLE ideas DROP COLUMN IF EXISTS is_off_topic;
ALTER TABLE ideas DROP COLUMN IF EXISTS is_public;
ALTER TABLE ideas DROP COLUMN IF EXISTS stage;
ALTER TABLE ideas DROP COLUMN IF EXISTS user_id;

DROP INDEX IF EXISTS idx_ideas_pinned_at;
DROP INDEX IF EXISTS idx_ideas_is_off_topic;
DROP INDEX IF EXISTS idx_ideas_is_public;
DROP INDEX IF EXISTS idx_ideas_stage;
DROP INDEX IF EXISTS idx_ideas_user_id;

-- 5. Drop email_notifications table
DROP TABLE IF EXISTS email_notifications CASCADE;

-- 6. Drop flags table
DROP TABLE IF EXISTS flags CASCADE;

-- 7. Drop users table
DROP TABLE IF EXISTS users CASCADE;
