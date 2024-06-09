use std::path::{Path, PathBuf};
use rocket::Responder;
use rocket::request::{Outcome, Request, FromRequest};
use rocket::response::Redirect;
use rocket::http::{CookieJar, Status};
use rocket::fs::{NamedFile, relative};
use rad_report::models::*;
use rad_report::establish_connection;
use diesel::prelude::*;
use rocket::serde::{Serialize, Deserialize, json::Json};
use chrono::Utc;
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Algorithm, Header, Validation};
use std::env;

#[derive(Responder, Debug)]
pub enum NetworkResponse {
    #[response(status = 201)]
    Created(String),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 409)]
    Conflict(String),
}

#[derive(Serialize)]
pub enum ResponseBody {
    Message(String),
    AuthToken(String),
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub subject_id: i32,
    exp: usize
}

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims
}

#[rocket::async_trait]
impl <'r> FromRequest<'r> for JWT {
    type Error = NetworkResponse;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, NetworkResponse> {
        fn is_valid(key: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
            Ok(decode_jwt(String::from(key))?)
        }

        match req.headers().get_one("authorization") {
            None => {
                Outcome::Forward(Status::Unauthorized)
            }
            Some(key) => match is_valid(key) {
                Ok(claims) => Outcome::Success(JWT { claims }),
                _ => Outcome::Forward(Status::Unauthorized)
            }

                    // jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    //     let response = LoginResponse { body: ResponseBody::Message(String::from("Expired login")) };
                    //     Outcome::Error((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
                    // },
                    // jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    //     let response = LoginResponse { body: ResponseBody::Message(String::from("Incorrect login or misformed authorization token")) };
                    //     Outcome::Error((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
                    // },
                    // _ => {
                    //     let response = LoginResponse { body: ResponseBody::Message(String::from(format!("Error validating token: {}", err))) };
                    //     Outcome::Error((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
                    // }
        }
    }
}

fn create_jwt(id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("Failed to calculate expiration time for JWT")
        .timestamp();
    let claims = Claims {
        subject_id: id,
        exp: expiration as usize
    };
    let header = Header::new(Algorithm::HS512);

    encode(&header, &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn decode_jwt(token: String) -> Result<Claims, jsonwebtoken::errors::ErrorKind> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = token.trim_start_matches("Bearer").trim();

    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(token) => Ok(token.claims),
        Err(err) => Err(err.kind().to_owned())
    }
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