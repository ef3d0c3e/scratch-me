// @generated automatically by Diesel CLI.

diesel::table! {
    cosmetics (user_id, kind, item) {
        user_id -> Int8,
        kind -> Text,
        item -> Text,
        obtained_at -> Timestamptz,
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
    }
}

diesel::joinable!(cosmetics -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(streaks -> users (user_id));
diesel::joinable!(tickets -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(cosmetics, sessions, streaks, tickets, users,);
