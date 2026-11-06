DELETE FROM users WHERE is_guest;
ALTER TABLE users
    DROP CONSTRAINT users_login_unless_guest,
    DROP COLUMN is_guest,
    ALTER COLUMN email SET NOT NULL,
    ALTER COLUMN password_hash SET NOT NULL;
