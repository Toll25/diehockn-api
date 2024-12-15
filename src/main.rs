use std::env;
use std::num::ParseIntError;
use std::process::Command;

use hmac::{digest::MacError, Hmac, Mac};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::request::{self, FromRequest, Request};
use rocket::Response;
use rocket::{http::Status, outcome::Outcome};
use sha2::Sha256;

#[macro_use]
extern crate rocket;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

struct Token(String);

#[derive(Debug)]
enum ApiTokenError {
    Missing,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ApiTokenError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = req.headers().get_one("X-Hub-Signature-256");
        token.map_or(
            Outcome::Error((Status::Unauthorized, ApiTokenError::Missing)),
            |token| Outcome::Success(Self(token.to_string())),
        )
    }
}

#[get("/")]
const fn index() -> &'static str {
    "Welcome to the Bone Zone 😎"
}

#[post("/update_website", data = "<payload>")]
fn update(a: Token, payload: &str) -> String {
    if verify(payload, a.0.as_str()).is_err() {
        "Bad Request".to_string();
    }
    //let work_dir = "/home/elena/Documents/Projects/diehockn.com/";
    let work_dir = "/app/diehockn";
    let output = Command::new("git")
        .args(["pull", "origin", "main"]) // Modify branch if needed
        .current_dir(work_dir)
        .output()
        .unwrap();

    if !output.status.success() {
        return "Pull failed".to_string();
    }
    println!(
        "Git pull successful:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let output = Command::new("docker")
        .args(["build", "-t", "diehockn", "."])
        .current_dir(work_dir)
        .output()
        .unwrap();

    if !output.status.success() {
        return "Build failed".to_string();
    }
    println!(
        "Docker build successful:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let output = Command::new("docker-compose")
        .args(["down"])
        .current_dir(work_dir)
        .output()
        .unwrap();

    if !output.status.success() {
        return "Compose down failed".to_string();
    }
    println!(
        "Docker Compose down successful:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let output = Command::new("docker-compose")
        .args(["up", "-d"])
        .current_dir(work_dir)
        .output()
        .unwrap();

    if !output.status.success() {
        return "Compose up failed".to_string();
    }
    println!(
        "Docker Compose stack redeployed:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    "Successfull deployment".to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CORS)
        .mount("/", routes![index, update])
}

fn verify(payload: &str, sig: &str) -> Result<(), MacError> {
    type HmacSha256 = Hmac<Sha256>;
    let secret =
        env::var("GITHUB_SECRET_KEY").expect("GITHUB_SECRET_KEY must be set in the environment");
    let secret = secret.trim();
    let sig = sig.trim_start_matches("sha256=");

    //dbg!(&sig);
    //dbg!(&payload);
    //dbg!(&secret);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());

    let val_sig = decode_hex(sig).unwrap();
    mac.verify_slice(&val_sig[..])
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
