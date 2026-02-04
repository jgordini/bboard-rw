CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Ideas table for anonymous idea submissions
CREATE TABLE IF NOT EXISTS ideas (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL CHECK (char_length(content) <= 500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    vote_count INTEGER NOT NULL DEFAULT 0
);

-- Votes table to track votes per idea
CREATE TABLE IF NOT EXISTS votes (
    id SERIAL PRIMARY KEY,
    idea_id INTEGER NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    voter_fingerprint TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(idea_id, voter_fingerprint)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_ideas_vote_count ON ideas(vote_count DESC);
CREATE INDEX IF NOT EXISTS idx_ideas_created_at ON ideas(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_votes_idea_id ON votes(idea_id);

-- Function to auto-increment vote_count when a vote is inserted
CREATE OR REPLACE FUNCTION increment_vote_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE ideas SET vote_count = vote_count + 1 WHERE id = NEW.idea_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically increment vote_count
CREATE TRIGGER trigger_increment_vote_count
    AFTER INSERT ON votes
    FOR EACH ROW
    EXECUTE FUNCTION increment_vote_count();

-- Function to auto-decrement vote_count when a vote is deleted
CREATE OR REPLACE FUNCTION decrement_vote_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE ideas SET vote_count = vote_count - 1 WHERE id = OLD.idea_id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically decrement vote_count
CREATE TRIGGER trigger_decrement_vote_count
    AFTER DELETE ON votes
    FOR EACH ROW
    EXECUTE FUNCTION decrement_vote_count();

-- Admin users table (simple password-based auth)
CREATE TABLE IF NOT EXISTS admin_users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
