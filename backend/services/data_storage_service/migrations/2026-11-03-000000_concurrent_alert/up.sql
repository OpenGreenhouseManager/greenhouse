-- Concurrent alerts: a lasting condition that stays open while its sender
-- keeps firing it. The existing `alert` table keeps holding one-time alerts.
--
-- An alert is open while expires_at > now(). Every fire moves expires_at to
-- now + keep_alive_secs; stopping sets it to now. It expired (rather than being
-- stopped) when expires_at = last_seen_at + keep_alive_secs.
CREATE TABLE concurrent_alert (
    id UUID PRIMARY KEY,
    severity severity NOT NULL,
    identifier TEXT NOT NULL,
    value TEXT NOT NULL,
    note TEXT,
    datasource_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_seen_at TIMESTAMP WITH TIME ZONE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    keep_alive_secs INTEGER NOT NULL CHECK (keep_alive_secs > 0),
    occurrences BIGINT NOT NULL DEFAULT 1
);

-- Finds the open alert of a source and identifier.
CREATE INDEX idx_concurrent_alert_key
    ON concurrent_alert (datasource_id, identifier, expires_at);
