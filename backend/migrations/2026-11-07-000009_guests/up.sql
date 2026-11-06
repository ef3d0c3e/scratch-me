-- Guests play without signing up: no email or password until they keep the account
ALTER TABLE users
    ALTER COLUMN email DROP NOT NULL,
    ALTER COLUMN password_hash DROP NOT NULL,
    ADD COLUMN is_guest BOOLEAN NOT NULL DEFAULT false,
    ADD CONSTRAINT users_login_unless_guest
        CHECK (is_guest OR (email IS NOT NULL AND password_hash IS NOT NULL));
