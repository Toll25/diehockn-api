use std::env;
use std::num::ParseIntError;
use std::process::{Command, Stdio};

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
#[get("/")]
const fn beacon() -> &'static str {
    "Welcome to the BBBeacon Zone 😎"
}

#[post("/website", data = "<payload>")]
fn update_website(a: Token, payload: &str) -> Result<String, String> {
    if verify(payload, a.0.as_str(), "GITHUB_WEBSITE_SECRET_KEY").is_err() {
        return Err("Bad Request".to_string());
    }
    //let work_dir = "/home/elena/Documents/Projects/diehockn.com/";
    let work_dir = "/app/diehockn.com";
    let script_path = "/app/update_script.sh";

    println!("Updating Website");

    match Command::new("bash")
        .arg(script_path)
        .arg(work_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(_) => {
            // Return immediately to satisfy the webhook response time requirement
            Ok("Deployment started successfully.".to_string())
        }
        Err(e) => {
            // Return an error if the script couldn't be started
            Err(format!("Failed to start deployment: {}", e))
        }
    }
}
#[post("/api", data = "<payload>")]
fn update_api(a: Token, payload: &str) -> Result<String, String> {
    if verify(payload, a.0.as_str(), "GITHUB_API_SECRET_KEY").is_err() {
        return Err("Bad Request".to_string());
    }
    //let work_dir = "/home/elena/Documents/Projects/diehockn.com/";
    let work_dir = "/app/diehockn-api";
    let script_path = "/app/update_script.sh";

    println!("Updating API");

    match Command::new("bash")
        .arg(script_path)
        .arg(work_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(_) => {
            // Return immediately to satisfy the webhook response time requirement
            Ok("Deployment started successfully.".to_string())
        }
        Err(e) => {
            // Return an error if the script couldn't be started
            Err(format!("Failed to start deployment: {}", e))
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CORS)
        .mount("/", routes![index])
        .mount("/beacon", routes![beacon])
        .mount("/update", routes![update_api, update_website])
}

fn verify(payload: &str, sig: &str, secret_env: &str) -> Result<(), MacError> {
    type HmacSha256 = Hmac<Sha256>;
    let secret = env::var(secret_env).expect("GITHUB_SECRET_KEY must be set in the environment");
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
