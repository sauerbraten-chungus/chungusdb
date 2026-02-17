CREATE TABLE players (
    chungid UUID PRIMARY KEY DEFAULT uuidv7(),
    name VARCHAR(80) NOT NULL,
    frags INTEGER NOT NULL DEFAULT 0 CHECK (frags >= 0),
    deaths INTEGER NOT NULL DEFAULT 0 CHECK (deaths >= 0),
    accuracy NUMERIC(5,2) NOT NULL DEFAULT 0.00 CHECK (accuracy BETWEEN 0.00 AND 100.00),
    matches_played INTEGER NOT NULL DEFAULT 0 CHECK (matches_played >= 0),
    elo INTEGER NOT NULL DEFAULT 1000 CHECK (elo >= 0),
    commendations INTEGER NOT NULL DEFAULT 0 CHECK (commendations >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE matches (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE match_participants (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    match_id UUID REFERENCES matches(id),
    chungid UUID REFERENCES players(chungid),
    name VARCHAR(80) NOT NULL,
    frags INTEGER NOT NULL DEFAULT 0 CHECK (frags >= 0),
    deaths INTEGER NOT NULL DEFAULT 0 CHECK (deaths >= 0),
    accuracy NUMERIC(5,2) NOT NULL DEFAULT 0.00 CHECK (accuracy BETWEEN 0.00 AND 100.00),
    elo INTEGER NOT NULL DEFAULT 1000 CHECK (elo >= 0)
);

CREATE INDEX idx_match_participants_chungid ON match_participants(chungid);
CREATE INDEX idx_match_participants_match_id ON match_participants(match_id);

-- Test
INSERT INTO players (chungid, name, frags, deaths, accuracy, matches_played, elo, commendations)
VALUES ('019400aa-0000-7000-8000-000000000001', 'chungusgamer2012', 100, 50, 75.50, 3, 1250, 5);
