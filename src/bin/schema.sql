CREATE TABLE IF NOT EXISTS problems (
    api_id INTEGER PRIMARY KEY NOT NULL,
    date_deleted TIMESTAMP,
    date_inserted TIMESTAMP NOT NULL,
    date_updated TIMESTAMP,
    downgraded BOOLEAN NOT NULL,
    grade TEXT NOT NULL,
    has_beta_video BOOLEAN NOT NULL,
    holdsetup INTEGER NOT NULL,
    is_benchmark BOOLEAN NOT NULL,
    is_master BOOLEAN NOT NULL,
    method TEXT CHECK(method IN ('feet_follow_hands', 'screw_ons_only', 'feet_follow_hands_and_screw_ons', 'footless_and_kick_board')) NOT NULL,
    moon_board_configuration_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    repeats INTEGER NOT NULL,
    setby TEXT NOT NULL,
    setby_id TEXT NOT NULL,
    upgraded BOOLEAN NOT NULL,
    user_grade TEXT,
    user_rating INTEGER
);

CREATE TABLE IF NOT EXISTS moves (
    description TEXT NOT NULL,
    is_end BOOLEAN NOT NULL,
    is_start BOOLEAN NOT NULL,
    problem_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS holdsets_for_problems (
    problem_id INTEGER NOT NULL,
    api_id INTEGER NOT NULL,
    description TEXT NOT NULL,
    locations INTEGER
)
