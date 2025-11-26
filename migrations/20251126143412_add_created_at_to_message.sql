-- Add migration script here
ALTER TABLE message ADD created_at TIMESTAMP DEFAULT (NOW() AT TIME ZONE 'utc')