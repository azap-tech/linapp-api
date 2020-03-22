CREATE TABLE users (
    id SERIAL NOT NULL PRIMARY KEY,
    hsecret TEXT NOT NULL UNIQUE
);

CREATE TABLE locations(
    id SERIAL PRIMARY KEY REFERENCES users (id),
    name TEXT NOT NULL
);

CREATE TYPE doctorstatus AS ENUM ('AVAILABLE', 'PAUSED', 'STOPED');

CREATE TABLE doctors (
    id SERIAL PRIMARY KEY REFERENCES users (id),
    location_id INTEGER REFERENCES locations(id),
    name TEXT NOT NULL,
    status doctorstatus NOT NULL DEFAULT 'STOPED',
    avatar TEXT,
    phone TEXT
);

CREATE TABLE tickets (
    id SERIAL NOT NULL PRIMARY KEY,
    location_id INTEGER NOT NULL REFERENCES locations(id),
    doctor_id INTEGER REFERENCES doctors(id),
    name TEXT NOT NULL,
    phone TEXT,
    sex TEXT,
    pathology TEXT,
    creation_time TIMESTAMP WITH TIME ZONE NOT NULL,
    started_time TIMESTAMP WITH TIME ZONE NULL,
    done_time TIMESTAMP WITH TIME ZONE NULL,
    canceled_time TIMESTAMP WITH TIME ZONE NULL
);