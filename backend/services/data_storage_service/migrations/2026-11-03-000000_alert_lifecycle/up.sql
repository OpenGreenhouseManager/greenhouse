-- Alerts get a kind and a lifetime.
--
-- kind 'one_time'   : a single event; starts and ends at created_at.
-- kind 'concurrent' : a lasting condition; open while it keeps being fired.
--                     Every fire moves expires_at to now + keep-alive. It ends
--                     when stopped or when expires_at passes without a fire.
--
-- last_seen_at : most recent fire
-- expires_at   : end of the alert unless fired again
-- ended_at     : when it ended; NULL while open
-- end_reason   : 'stopped' or 'expired' for ended concurrent alerts
-- occurrences  : number of fires
ALTER TABLE alert
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'one_time'
        CHECK (kind IN ('one_time', 'concurrent')),
    ADD COLUMN last_seen_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN expires_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN ended_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN end_reason TEXT CHECK (end_reason IN ('stopped', 'expired')),
    ADD COLUMN occurrences BIGINT NOT NULL DEFAULT 1;

-- Existing rows were single events.
UPDATE alert SET last_seen_at = created_at, expires_at = created_at, ended_at = created_at;

ALTER TABLE alert
    ALTER COLUMN last_seen_at SET NOT NULL,
    ALTER COLUMN expires_at SET NOT NULL;

-- At most one open alert per source and identifier: firing an open alert
-- updates it instead of opening a second one.
CREATE UNIQUE INDEX idx_alert_open
    ON alert (datasource_id, identifier)
    WHERE ended_at IS NULL;

-- Finds open alerts whose keep-alive ran out.
CREATE INDEX idx_alert_open_expires_at
    ON alert (expires_at)
    WHERE ended_at IS NULL;
