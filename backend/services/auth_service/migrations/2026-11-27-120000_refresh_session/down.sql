ALTER TABLE users DROP COLUMN session_expires_at;
ALTER TABLE users DROP COLUMN session_id;
UPDATE users SET login_session = '';
