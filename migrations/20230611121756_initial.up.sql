CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE owner (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

     username TEXT NOT NULL UNIQUE,

     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE repository (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     
     owner UUID NOT NULL REFERENCES owner(id),
     namespace_name TEXT NOT NULL UNIQUE,
     
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE upload_session (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     repository TEXT NOT NULL REFERENCES repository(namespace_name),
     
     digest TEXT,
     previous_session UUID REFERENCES upload_session(id),
     is_finished BOOLEAN NOT NULL DEFAULT FALSE,

     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE blob (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

     repository TEXT NOT NULL REFERENCES repository(namespace_name),
     digest TEXT NOT NULL,

     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

     UNIQUE(id, repository),
     UNIQUE(id, digest)
);

CREATE TABLE manifest (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

     repository TEXT NOT NULL REFERENCES repository(namespace_name),
     tag TEXT NOT NULL,
     blob_id UUID NOT NULL REFERENCES blob(id),
     digest TEXT NOT NULL,
     content_type_top TEXT NOT NULL,
     content_type_sub TEXT NOT NULL,

     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

     UNIQUE (repository, tag)
);

CREATE TABLE manifest_layer (
     manifest_id UUID NOT NULL REFERENCES manifest(id),
     blob_id UUID NOT NULL REFERENCES blob(id),

     media_type TEXT NOT NULL,
     size BIGINT NOT NULL,

     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

     PRIMARY KEY (manifest_id, blob_id)
);
