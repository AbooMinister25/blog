// use diesel::backend::Backend;
// use diesel::deserialize::FromSql;
// use diesel::serialize::ToSql;
// use diesel::sql_types::Text;

// #[derive(Debug)]
// pub struct Tags(serde_json::Value);

// impl<DB> FromSql<Text, DB> for Tags
// where
//     DB: Backend,
//     String: FromSql<Text, DB>,
// {
//     fn from_sql(
//         bytes: Option<&<DB as diesel::backend::Backend>::RawValue>,
//     ) -> diesel::deserialize::Result<Self> {
//         let t = String::from_sql(bytes)?;
//         Ok(Self(serde_json::from_str(&t)?))
//     }
// }

// impl<DB> ToSql<Text, DB> for Tags
// where
//     DB: Backend,
//     String: ToSql<Text, DB>,
// {
//     fn to_sql<W: std::io::Write>(
//         &self,
//         out: &mut diesel::serialize::Output<W, DB>,
//     ) -> diesel::serialize::Result {
//         let s = serde_json::to_string(&self.0)?;
//         String::to_sql(&s, out)
//     }
// }


// #[derive(Queryable)]
// pub struct Post {
//     pub id: i32,
//     pub title: String,
//     pub content: String,
// }
