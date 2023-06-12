CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE repository (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     namespace_name TEXT NOT NULL UNIQUE,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE upload_session (
     id UUID DEFAULT uuid_generate_v4() UNIQUE,
     repository TEXT REFERENCES repository(namespace_name),
     previous_session UUID REFERENCES upload_session(id),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     is_finished BOOLEAN NOT NULL DEFAULT FALSE,

     PRIMARY KEY (id, repository)
);
