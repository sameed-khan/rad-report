use std::path::{Path, PathBuf};
use rocket::Route;
use rocket::fs::NamedFile;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::http::Status;
use rocket_dyn_templates::{Template, context};
use rad_report::models::*;
use rad_report::schema::{cases, users};
use diesel::prelude::*;
use diesel::dsl::{count, min, max};
use rad_report::establish_connection;
use serde::Serialize;
use chrono::NaiveDateTime;

#[derive(Debug)]
struct UserID(i32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserID {
    type Error = std::convert::Infallible; // TODO: change to NetworkResponse

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<UserID, Self::Error> {
        request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse::<i32>().ok())
            .map(UserID)
            .or_forward(Status::Unauthorized)
    }
}

fn capitalize_first_letter(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str()
    }
}

#[derive(Serialize, Debug)]
struct AggregationResult {
    // #[serde(flatten)]
    category: String,
    count: i64
}

#[get("/")]
fn index(user_id: UserID) -> Template {
    use rad_report::schema::users::dsl::*;
    use rad_report::schema::cases::dsl::*;

    let conn = &mut establish_connection();
    let query_id = user_id.0;
    let user: User = users
        .select(User::as_select())
        .filter(rad_report::schema::users::dsl::id.eq(&query_id))
        .get_result(conn)
        .expect("Diesel error: user query unsuccessful");

    let modality_counts = cases
        .filter(rad_report::schema::cases::dsl::npi.eq(&user.npi))
        .group_by(rad_report::schema::cases::modality)
        .select((rad_report::schema::cases::modality, count(rad_report::schema::cases::dsl::id)))
        .load::<(String, i64)>(conn)
        .expect("Diesel error: modality group by query unsuccessful")
        .into_iter()
        .map(|(cat, count)| AggregationResult { category: cat, count: count })
        .collect::<Vec<AggregationResult>>();

    let subspecialty_counts = cases
        .filter(rad_report::schema::cases::dsl::npi.eq(&user.npi))
        .group_by(rad_report::schema::cases::subspecialty)
        .select((rad_report::schema::cases::subspecialty, count(rad_report::schema::cases::dsl::id)))
        .load::<(String, i64)>(conn)
        .expect("Diesel error: subspecialty group by query unsuccessful")
        .into_iter()
        .map(|(cat, count)| AggregationResult { category: cat, count: count })
        .collect::<Vec<AggregationResult>>();

    let (stardate, endate) = cases
        .filter(rad_report::schema::cases::dsl::npi.eq(&user.npi))
        .select((min(rad_report::schema::cases::dsl::read_at), max(rad_report::schema::cases::dsl::read_at)))
        // .select(min(rad_report::schema::cases::dsl::read_at))
        .load::<(Option<NaiveDateTime>, Option<NaiveDateTime>)>(conn)
        .expect("Diesel error: datestring query unsuccessful")
        .into_iter()
        .map(|(a, b)| (
            a.unwrap().format("%B %d, %Y").to_string(),
            b.unwrap().format("%B %d, %Y").to_string(),
        ))
        .next()
        .expect("No datestrings found");

    let total_cases = cases
        .filter(rad_report::schema::cases::dsl::npi.eq(&user.npi))
        .select(count(rad_report::schema::cases::dsl::id))
        .first::<i64>(conn)
        .expect("Diesel error: total_cases query unsuccessful");

    Template::render("dashboard", context! {
        firstname: capitalize_first_letter(&user.firstname),
        lastname: capitalize_first_letter(&user.lastname),
        degree: user.degree.replace("_", " ").to_uppercase(),
        training_year: user.training_year,
        start_date: stardate,
        end_date: endate,
        modcounts: modality_counts,
        subspec_counts: subspecialty_counts,
        total_cases: total_cases
    })
}


#[get("/", rank = 2)]
fn dashboard_redirect() -> Redirect {
    Redirect::to(uri!("/login"))
}

#[get("/<file..>")]
async fn get_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/dashboard/").join(file)).await.ok()
}

pub fn routes() -> Vec<Route> {
    routes![index, dashboard_redirect, get_files]
}