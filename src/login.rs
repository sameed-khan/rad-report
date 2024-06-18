use std::path::{Path, PathBuf};
use rocket::Responder;
use rocket::response::Redirect;
use rocket::http::CookieJar;
use rocket::fs::{NamedFile, relative};
use rad_report::models::*;
use rad_report::establish_connection;
use diesel::prelude::*;
use rocket::serde::{Serialize, Deserialize, json::Json};
use std::env;

#[derive(Responder, Debug)]
pub enum NetworkResponse {
    // #[response(status = 201)]
    // Created(String),
    // #[response(status = 400)]
    // BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    // #[response(status = 404)]
    // NotFound(String),
    // #[response(status = 409)]
    // Conflict(String),
}

#[derive(Serialize)]
pub enum ResponseBody {
    Message(String),
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub body: ResponseBody,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String
}

#[post("/", format = "json", data = "<login>")]
async fn post_login(login: Json<LoginRequest>, jar: &CookieJar<'_>) -> Result<Redirect, NetworkResponse>{
    use rad_report::schema::users::dsl::*;

    let user: LoginRequest = login.into_inner();
    let user: User = match users
        .select(User::as_select())
        .filter(username.eq(&user.username))
        .filter(password_hash.eq(&user.password))
        .first::<User>(&mut establish_connection()) {
            Ok(user) => user,
            Err(err) => match err {
                diesel::result::Error::NotFound => {
                    let response = LoginResponse { 
                        body: ResponseBody::Message(format!("Error - wrong username or password for user {}", &user.username)) 
                    };
                    return Err(NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap()));
                },
                _ => {
                    panic!("Error: {:?}", err);
                }
            }
        };

    jar.add_private(("user_id", user.id.to_string()));
    Ok(Redirect::to(uri!("/dashboard")))
}

#[get("/<file..>", rank = 2)]
async fn get_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/login/").join(file)).await.ok()
}

#[get("/")]
async fn get() -> Option<NamedFile> {
    NamedFile::open(relative!("static/login/index.html")).await.ok()
}

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_files, post_login]
}