-- Add migration script here
ALTER TABLE room ALTER COLUMN created_at SET DEFAULT (NOW() AT TIME ZONE 'utc')