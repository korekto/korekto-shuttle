DROP TABLE IF EXISTS "user";

CREATE TABLE "user" (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  provider_login VARCHAR NOT NULL UNIQUE,
  email VARCHAR NOT NULL,
  avatar_url VARCHAR NOT NULL,
  installation_id VARCHAR,
  github_user_tokens JSONB,
  admin BOOLEAN DEFAULT FALSE NOT NULL,
  teacher BOOLEAN DEFAULT FALSE NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);
