DROP INDEX IF EXISTS idx_alert_open_expires_at;
DROP INDEX IF EXISTS idx_alert_open;

ALTER TABLE alert
    DROP COLUMN IF EXISTS occurrences,
    DROP COLUMN IF EXISTS end_reason,
    DROP COLUMN IF EXISTS ended_at,
    DROP COLUMN IF EXISTS expires_at,
    DROP COLUMN IF EXISTS last_seen_at,
    DROP COLUMN IF EXISTS kind;
