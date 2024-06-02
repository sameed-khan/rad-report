#[macro_use] extern crate rocket;

use rocket::response::Redirect;
use rocket::fs::{FileServer, relative};

#[get("/")]
fn index() -> Redirect {
   Redirect::to(uri!("/public/login"))  // TODO: Implement token checking logic to decide whether to redirect to login or content
}

// #[get("/login")]
// async fn login() -> () {
//     println!("Login page requested");
// }

#[launch]
fn rocket() -> _ {
   rocket::build()
   .mount("/", routes![index])
   .mount("/public", FileServer::from(relative!("static")))
}
