-- Fish species a player has caught (Fishing tickets)
CREATE TABLE aquarium (
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    species TEXT NOT NULL,
    caught_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, species)
);
