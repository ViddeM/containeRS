CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE repository (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     namespace_name TEXT NOT NULL UNIQUE,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE upload_session (
     id UUID DEFAULT uuid_generate_v4(),
     repository_id UUID REFERENCES repository(id),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

     PRIMARY KEY (id, repository_id)
);
