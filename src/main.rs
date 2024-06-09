#[macro_use] extern crate rocket;

use rocket::response::Redirect;
use rocket::fs::{NamedFile, relative};
use rocket_dyn_templates::Template;

mod login;
mod dashboard;

#[get("/")]
fn index() -> Redirect {
   Redirect::to(uri!("/login"))  // TODO: Implement token checking logic to decide whether to redirect to login or content
}

#[get("/static/favicon.png")]
async fn get_favicon() -> Option<NamedFile> {
    NamedFile::open(relative!("/static/favicon.png")).await.ok()
}

#[launch]
fn rocket() -> _ {
   rocket::build()
   .mount("/", routes![index, get_favicon])
   .mount("/login", login::routes())
   .mount("/dashboard", dashboard::routes())
   .attach(Template::fairing())
}
