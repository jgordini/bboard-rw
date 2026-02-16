-- Add stable CAS identity linkage for SSO accounts.
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS cas_subject VARCHAR(255);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'users_cas_subject_unique'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT users_cas_subject_unique UNIQUE (cas_subject);
    END IF;
END $$;
