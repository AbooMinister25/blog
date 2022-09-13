// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Nullable<Integer>,
        title -> Text,
        content -> Text,
        summary -> Text,
        tags -> Text,
        published -> Bool,
        published_at -> Timestamp,
    }
}
