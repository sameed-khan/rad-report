use rocket::Route;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::http::Status;
use rocket_dyn_templates::{Template, context};
use rad_report::models::*;
use diesel::prelude::*;
use rad_report::establish_connection;

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

#[get("/")]
fn index(user_id: UserID) -> Template {
    use rad_report::schema::users::dsl::*;

    let query_id = user_id.0;
    let user_name: String = users
        .select(User::as_select())
        .filter(id.eq(&query_id))
        .first::<User>(&mut establish_connection())
        .expect("User not found, cookie authentication failed!")
        .username;

    Template::render("dashboard", context! {
        username: user_name,
        id: query_id
    })
}

#[get("/", rank = 2)]
fn dashboard_redirect() -> Redirect {
    Redirect::to(uri!("/login"))
}

pub fn routes() -> Vec<Route> {
    routes![index, dashboard_redirect]
}