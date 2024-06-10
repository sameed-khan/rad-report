// @generated automatically by Diesel CLI.

diesel::table! {
    cases (id) {
        id -> Int4,
        read_at -> Timestamp,
        #[max_length = 10]
        npi -> Varchar,
        #[max_length = 50]
        exam_name -> Varchar,
        #[max_length = 50]
        modality -> Varchar,
        #[max_length = 50]
        subspecialty -> Varchar,
        is_child -> Bool,
        #[max_length = 50]
        facility_name -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        #[max_length = 20]
        firstname -> Varchar,
        #[max_length = 20]
        lastname -> Varchar,
        #[max_length = 10]
        npi -> Varchar,
        #[max_length = 20]
        degree -> Varchar,
        training_year -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cases,
    users,
);
