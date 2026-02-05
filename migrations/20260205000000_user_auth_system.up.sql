-- User Authentication System Migration
-- Creates user accounts, flags, email notifications, and updates existing tables

-- 1. Create users table (replaces admin_users)
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(200) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role SMALLINT NOT NULL DEFAULT 0,  -- 0: User, 1: Moderator, 2: Admin
    created_on TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);

-- 2. Create flags table for content moderation
CREATE TABLE IF NOT EXISTS flags (
    id SERIAL PRIMARY KEY,
    target_type VARCHAR(20) NOT NULL,  -- 'idea' or 'comment'
    target_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, target_type, target_id)  -- One flag per user per item
);

CREATE INDEX idx_flags_target ON flags(target_type, target_id);
CREATE INDEX idx_flags_user_id ON flags(user_id);

-- 3. Create email_notifications table (structure only, SMTP deferred)
CREATE TABLE IF NOT EXISTS email_notifications (
    id SERIAL PRIMARY KEY,
    recipient_email VARCHAR(200) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,  -- 0: Pending, 1: Sent, 2: Failed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_notifications_status ON email_notifications(status);

-- 4. TRUNCATE existing data (fresh start)
TRUNCATE TABLE votes CASCADE;
TRUNCATE TABLE comments CASCADE;
TRUNCATE TABLE ideas CASCADE;

-- 5. ALTER ideas table
ALTER TABLE ideas
    ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN stage VARCHAR(50) NOT NULL DEFAULT 'Ideate',
    ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN is_off_topic BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN pinned_at TIMESTAMPTZ NULL;

-- Update title column type
ALTER TABLE ideas ALTER COLUMN title TYPE VARCHAR(100);

-- Make user_id NOT NULL after adding it
ALTER TABLE ideas ALTER COLUMN user_id SET NOT NULL;

CREATE INDEX idx_ideas_user_id ON ideas(user_id);
CREATE INDEX idx_ideas_stage ON ideas(stage);
CREATE INDEX idx_ideas_is_public ON ideas(is_public);
CREATE INDEX idx_ideas_is_off_topic ON ideas(is_off_topic);
CREATE INDEX idx_ideas_pinned_at ON ideas(pinned_at DESC NULLS LAST);

-- 6. ALTER votes table (replace fingerprint with user_id)
ALTER TABLE votes DROP COLUMN voter_fingerprint;
ALTER TABLE votes ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;

-- Make user_id NOT NULL
ALTER TABLE votes ALTER COLUMN user_id SET NOT NULL;

-- Add new unique constraint
ALTER TABLE votes ADD CONSTRAINT votes_user_id_idea_id_unique UNIQUE(user_id, idea_id);

-- Add index for user_id
CREATE INDEX idx_votes_user_id ON votes(user_id);

-- 7. ALTER comments table
ALTER TABLE comments
    ADD COLUMN user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT false;

-- Make user_id NOT NULL after adding it
ALTER TABLE comments ALTER COLUMN user_id SET NOT NULL;

CREATE INDEX idx_comments_user_id ON comments(user_id);
CREATE INDEX idx_comments_is_pinned ON comments(is_pinned);
CREATE INDEX idx_comments_is_deleted ON comments(is_deleted);

-- 8. DROP admin_users table (replaced by users with roles)
DROP TABLE IF EXISTS admin_users CASCADE;
