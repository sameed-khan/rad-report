use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub firstname: String,
    pub lastname: String,
    pub npi: String,
    pub degree: String,
    pub training_year: i32
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User, foreign_key = id))]
#[diesel(table_name = crate::schema::cases)]
pub struct Case {
    pub id: i32,
    pub read_at: NaiveDateTime,
    pub npi: String,
    pub exam_name: String,
    pub modality: String,
    pub subspecialty: String,
    pub is_child: bool,
    pub facility_name: String
}