ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_cas_subject_unique;

ALTER TABLE users
    DROP COLUMN IF EXISTS cas_subject;
