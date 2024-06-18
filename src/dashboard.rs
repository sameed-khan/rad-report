use std::path::{Path, PathBuf};
use rocket::Route;
use rocket::fs::NamedFile;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_dyn_templates::{Template, context};
use rad_report::models::*;
use diesel::prelude::*;
use diesel::dsl::{count, min, max, sql_query};
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

fn capitalize_degrees(degree: &str) -> String {
    let degrees: Vec<&str> = degree.split("_").collect();
    let mut capitalized_degrees = Vec::new();

    for degree in degrees {
        let capitalized_degree = match degree.trim().to_lowercase().as_str() {
            "md" => "MD".to_string(),
            "do" => "DO".to_string(),
            "mbbs" => "MBBS".to_string(),
            "ms" => "MS".to_string(),
            "phd"=> "PhD".to_string(),
            "mba" => "MBA".to_string(),
            "dphil" => "DPhil".to_string(),
            _ => degree.to_uppercase() 
        };
        capitalized_degrees.push(capitalized_degree);
    }

    capitalized_degrees.join(", ")
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
        degree: capitalize_degrees(&user.degree),
        training_year: user.training_year,
        start_date: stardate,
        end_date: endate,
        modcounts: modality_counts,
        subspec_counts: subspecialty_counts,
        total_cases: total_cases
    })
}

#[derive(QueryableByName, Debug)]
struct DonutGraphQueryResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    category_out: String,
    #[diesel(sql_type = diesel::sql_types::Float)]
    pct: f32,
}

#[derive(Serialize)]
struct DonutGraphData {
     subspecialty: GraphDataResponse,
     modality: GraphDataResponse
}

#[derive(Serialize)]
struct GraphDataResponse {
    data: Vec<f32>,
    categories: Vec<String>
}

// This is not clean, but I'm practicing learning how to use the type system
impl From<Vec<DonutGraphQueryResult>> for GraphDataResponse {
    fn from(results: Vec<DonutGraphQueryResult>) -> Self {
        let data = results.iter().map(|x| x.pct as f32).collect();
        let categories = results.iter().map(|x| x.category_out.clone()).collect();
        GraphDataResponse { data, categories }
    }
}

#[get("/donuts")]
fn get_donut_graph_data (user_id: UserID) -> Json<DonutGraphData> {
    let conn = &mut establish_connection();
    let query_id = user_id.0;
    let user: User = rad_report::schema::users::dsl::users
        .select(User::as_select())
        .filter(rad_report::schema::users::dsl::id.eq(&query_id))
        .get_result(conn)
        .expect("Diesel error: user query unsuccessful");

    const QUERY_STRING: &str =
    r#" SELECT (((SUM(tbl1.cat_count) / tbl2.total) * 100)::real) AS pct, tbl1.{column} AS category_out
        FROM users
        INNER JOIN (
            SELECT {column}, COUNT(id) AS cat_count, npi
            FROM cases
            GROUP BY {column}, npi
        ) AS tbl1
        ON tbl1.npi = users.npi INNER JOIN (
            SELECT COUNT(id) AS total, npi
            FROM cases
            GROUP BY npi
        ) AS tbl2
        ON tbl2.npi = users.npi
        WHERE users.npi = $1
        GROUP BY users.npi, tbl1.{column}, tbl2.total"#;

    let subspecialty_data: GraphDataResponse = sql_query(QUERY_STRING.replace("{column}", "subspecialty"))
        .bind::<diesel::sql_types::Text, _>(&user.npi)
        .load::<DonutGraphQueryResult>(conn)
        .expect("Diesel error: subspecialty donut graph query unsuccessful")
        .into();

    let modality_data: GraphDataResponse = sql_query(QUERY_STRING.replace("{column}", "modality"))
        .bind::<diesel::sql_types::Text, _>(&user.npi)
        .load::<DonutGraphQueryResult>(conn)
        .expect("Diesel error: modality donut graph query unsuccessful")
        .into();

    let donut_data_response = DonutGraphData {
        subspecialty: subspecialty_data,
        modality: modality_data
    };

    Json(donut_data_response)
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
    routes![index, dashboard_redirect, get_files, get_donut_graph_data]
}