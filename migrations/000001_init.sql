CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

--DROP TABLE IF EXISTS "user";

CREATE TABLE IF NOT EXISTS "user" (
  id SERIAL PRIMARY KEY,
  uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  provider_name VARCHAR NOT NULL,
  provider_login VARCHAR NOT NULL UNIQUE,
  provider_email VARCHAR NOT NULL,
  school_email VARCHAR NOT NULL,
  avatar_url VARCHAR NOT NULL,
  installation_id VARCHAR,
  github_user_tokens JSONB,
  admin BOOLEAN DEFAULT FALSE NOT NULL,
  teacher BOOLEAN DEFAULT FALSE NOT NULL,
  first_name VARCHAR NOT NULL,
  last_name VARCHAR NOT NULL,
  school_group VARCHAR NOT NULL,
  UNIQUE (first_name, last_name)
);

--DROP TABLE IF EXISTS "module" CASCADE;

CREATE TABLE IF NOT EXISTS "module" (
  id SERIAL PRIMARY KEY,
  uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  name VARCHAR NOT NULL,
  start TIMESTAMPTZ NOT NULL,
  stop TIMESTAMPTZ NOT NULL,
  unlock_key VARCHAR NOT NULL
);

--DROP TABLE IF EXISTS "assignment";

CREATE TABLE IF NOT EXISTS "assignment" (
  id SERIAL PRIMARY KEY,
  uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  module_id integer,
  name VARCHAR NOT NULL,
  start TIMESTAMPTZ NOT NULL,
  stop TIMESTAMPTZ NOT NULL,
  description VARCHAR NOT NULL,
  type VARCHAR NOT NULL,
  factor_percentage INTEGER NOT NULL,
  subject_url VARCHAR NOT NULL,
  grader_url VARCHAR NOT NULL,
  repository_name VARCHAR NOT NULL,
  grader_run_url VARCHAR NOT NULL,
  CONSTRAINT fk_assignment_module_id
        FOREIGN KEY(module_id)
        REFERENCES module(id)
        ON DELETE CASCADE
);

DROP TABLE IF EXISTS "unparseable_webhook";

CREATE TABLE IF NOT EXISTS "unparseable_webhook" (
  created_at TIMESTAMPTZ DEFAULT NOW(),
  origin VARCHAR NOT NULL,
  event VARCHAR NOT NULL,
  payload VARCHAR NOT NULL,
  error VARCHAR NOT NULL
);
