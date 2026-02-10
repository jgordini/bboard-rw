-- Add comments_enabled column to ideas table
ALTER TABLE ideas ADD COLUMN comments_enabled BOOLEAN NOT NULL DEFAULT true;
