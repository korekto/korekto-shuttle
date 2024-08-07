CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

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

CREATE TABLE IF NOT EXISTS module (
  id SERIAL PRIMARY KEY,
  uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  name VARCHAR NOT NULL,
  description VARCHAR NOT NULL,
  source_url VARCHAR NOT NULL,
  start TIMESTAMPTZ NOT NULL,
  stop TIMESTAMPTZ NOT NULL,
  unlock_key VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS teacher_module (
  module_id integer NOT NULL,
  teacher_id integer NOT NULL,
  UNIQUE (module_id, teacher_id),
  CONSTRAINT fk_teacher_module_module_id
        FOREIGN KEY(module_id)
        REFERENCES module(id)
        ON DELETE CASCADE,
  CONSTRAINT fk_teacher_module_teacher_id
        FOREIGN KEY(teacher_id)
        REFERENCES "user"(id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS assignment (
  id SERIAL PRIMARY KEY,
  uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  module_id integer,
  name VARCHAR NOT NULL,
  description VARCHAR NOT NULL,
  start TIMESTAMPTZ NOT NULL,
  stop TIMESTAMPTZ NOT NULL,
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

CREATE TABLE IF NOT EXISTS unparseable_webhook (
  created_at TIMESTAMPTZ DEFAULT NOW(),
  origin VARCHAR NOT NULL,
  event VARCHAR NOT NULL,
  payload VARCHAR NOT NULL,
  error VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS user_module (
  created_at TIMESTAMPTZ DEFAULT NOW(),
  user_id integer NOT NULL,
  module_id integer NOT NULL,
  UNIQUE (user_id, module_id),
  CONSTRAINT fk_user_module_user_id
        FOREIGN KEY(user_id)
        REFERENCES "user"(id)
        ON DELETE CASCADE,
  CONSTRAINT fk_user_module_id
        FOREIGN KEY(module_id)
        REFERENCES module(id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS user_assignment (
  id SERIAL PRIMARY KEY,
  uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  user_id integer NOT NULL,
  assignment_id integer NOT NULL,
  repository_linked boolean NOT NULL DEFAULT FALSE,
  normalized_grade NUMERIC(4, 2) NOT NULL DEFAULT 0,
  grades_history JSONB DEFAULT '[]'::jsonb,
  graded_last_at TIMESTAMPTZ,
  grading_in_progress boolean NULL DEFAULT FALSE,
  previous_grading_error VARCHAR,
  running_grading_metadata JSONB,
  UNIQUE (user_id, assignment_id),
  CONSTRAINT fk_user_assignment_user_id
        FOREIGN KEY(user_id)
        REFERENCES "user"(id)
        ON DELETE CASCADE,
  CONSTRAINT fk_user_assignment_id
        FOREIGN KEY(assignment_id)
        REFERENCES assignment(id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS grading_task (
  id SERIAL PRIMARY KEY,
  uuid UUID DEFAULT gen_random_uuid() NOT NULL UNIQUE,
  user_assignment_id integer NOT NULL,
  user_provider_login VARCHAR NOT NULL,
  status VARCHAR NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  repository VARCHAR NOT NULL,
  grader_repository VARCHAR NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  UNIQUE (user_assignment_id, user_provider_login, status),
  CONSTRAINT fk_grading_task_user_assignment_id
        FOREIGN KEY(user_assignment_id)
        REFERENCES user_assignment(id)
        ON DELETE CASCADE
)
