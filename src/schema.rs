table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        summary -> Text,
        published -> Bool,
        published_date -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        passwd -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
