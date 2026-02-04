DROP TRIGGER IF EXISTS trigger_decrement_vote_count ON votes;
DROP TRIGGER IF EXISTS trigger_increment_vote_count ON votes;
DROP FUNCTION IF EXISTS decrement_vote_count();
DROP FUNCTION IF EXISTS increment_vote_count();
DROP TABLE IF EXISTS votes CASCADE;
DROP TABLE IF EXISTS ideas CASCADE;
DROP TABLE IF EXISTS admin_users CASCADE;
