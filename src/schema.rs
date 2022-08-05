table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        summary -> Text,
        published -> Bool,
        published_at -> Timestamp,
        tags -> Nullable<Array<Text>>,
    }
}
