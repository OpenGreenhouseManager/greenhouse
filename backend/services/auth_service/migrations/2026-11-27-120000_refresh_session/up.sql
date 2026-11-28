-- login_session now holds the SHA-256 of the refresh token instead of the
-- whole JWT. Old values are meaningless, so every user logs in once again.
UPDATE users SET login_session = '';
ALTER TABLE users ADD COLUMN session_id UUID;
ALTER TABLE users ADD COLUMN session_expires_at TIMESTAMPTZ;
