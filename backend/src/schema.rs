// @generated automatically by Diesel CLI.

diesel::table! {
    achievements (user_id, id) {
        user_id -> Int8,
        id -> Text,
        earned_at -> Timestamptz,
    }
}

diesel::table! {
    announcements (id) {
        id -> Int8,
        author_id -> Nullable<Int8>,
        body -> Text,
        created_at -> Timestamptz,
        removed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    cosmetics (user_id, kind, item) {
        user_id -> Int8,
        kind -> Text,
        item -> Text,
        obtained_at -> Timestamptz,
    }
}

diesel::table! {
    mutes (user_id, muted_id) {
        user_id -> Int8,
        muted_id -> Int8,
    }
}

diesel::table! {
    quest_days (user_id, day) {
        user_id -> Int8,
        day -> Date,
        rerolled -> Bool,
        bonus_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    quests (user_id, day, key) {
        user_id -> Int8,
        day -> Date,
        key -> Text,
        progress -> Int4,
        done_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Int8,
        user_id -> Int8,
        token_hash -> Bytea,
        created_at -> Timestamptz,
        last_seen_at -> Timestamptz,
        expires_at -> Timestamptz,
        user_agent -> Text,
    }
}

diesel::table! {
    shouts (id) {
        id -> Int8,
        user_id -> Int8,
        body -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    stats (user_id) {
        user_id -> Int8,
        scratched -> Int4,
        wins -> Int4,
        best_win -> Int8,
        quests_done -> Int4,
    }
}

diesel::table! {
    streaks (user_id) {
        user_id -> Int8,
        day -> Int4,
        best -> Int4,
        last_claim -> Nullable<Date>,
    }
}

diesel::table! {
    tickets (id) {
        id -> Int8,
        user_id -> Int8,
        series -> Text,
        layout -> Jsonb,
        prize -> Jsonb,
        source -> Text,
        created_at -> Timestamptz,
        scratched_at -> Nullable<Timestamptz>,
        picks -> Nullable<Jsonb>,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        coins -> Int8,
        tz -> Text,
        pocket_count -> Int4,
        pocket_at -> Timestamptz,
        created_at -> Timestamptz,
        theme -> Text,
        tool -> Text,
        xp -> Int8,
        title -> Nullable<Text>,
        is_admin -> Bool,
    }
}

diesel::joinable!(achievements -> users (user_id));
diesel::joinable!(cosmetics -> users (user_id));
diesel::joinable!(quest_days -> users (user_id));
diesel::joinable!(quests -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(shouts -> users (user_id));
diesel::joinable!(stats -> users (user_id));
diesel::joinable!(streaks -> users (user_id));
diesel::joinable!(tickets -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    achievements,
    announcements,
    cosmetics,
    mutes,
    quest_days,
    quests,
    sessions,
    shouts,
    stats,
    streaks,
    tickets,
    users,
);
