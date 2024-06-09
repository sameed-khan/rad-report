// @generated automatically by Diesel CLI.

diesel::table! {
    cases (id) {
        id -> Int4,
        read_at -> Nullable<Timestamp>,
        #[max_length = 50]
        npi -> Nullable<Varchar>,
        #[max_length = 50]
        exam_name -> Nullable<Varchar>,
        #[max_length = 50]
        modality -> Nullable<Varchar>,
        #[max_length = 50]
        subspecialty -> Nullable<Varchar>,
        is_child -> Nullable<Bool>,
        #[max_length = 50]
        facility_name -> Nullable<Varchar>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        #[max_length = 20]
        firstname -> Nullable<Varchar>,
        #[max_length = 20]
        lastname -> Nullable<Varchar>,
        #[max_length = 20]
        degree -> Nullable<Varchar>,
        training_year -> Nullable<Int4>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cases,
    users,
);
