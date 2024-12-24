use std::collections::HashMap;

use beacon_calculator::{
    calculate_color_from_panes_default, find_combination_custom, find_combination_default,
    get_standard_colors, Panes, PreciseRGB,
};
use rocket::serde::json::Json;

use rocket::request::{self, FromRequest, Request};
use rocket::Route;
use rocket::{http::Status, outcome::Outcome};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};

pub fn get_routes() -> Vec<Route> {
    routes![
        beacon,
        approx,
        approx_cust,
        panes,
        colors,
        options_approx_cust
    ]
}

struct DepthCutoffHeaders {
    depth: u8,
    cutoff: u8,
}

#[derive(Deserialize, Debug)]
struct Colors {
    x: HashMap<String, [u8; 3]>,
}
impl Serialize for Colors {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_map(Some(self.x.len()))?;
        for (k, v) in &self.x {
            seq.serialize_entry(&k.to_string(), &v)?;
        }
        seq.end()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DepthCutoffHeaders {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let depth_header = req.headers().get_one("depth");
        let cutoff_header = req.headers().get_one("cutoff");

        if let (Some(depth), Some(cutoff)) = (depth_header, cutoff_header) {
            if let (Ok(depth), Ok(cutoff)) = (depth.parse::<u8>(), cutoff.parse::<u8>()) {
                return Outcome::Success(Self { depth, cutoff });
            }
        }
        Outcome::Error((Status::BadRequest, ()))
    }
}
// Define a struct to hold the query parameters
#[derive(FromForm)]
struct ColorQuery {
    r: u8,
    g: u8,
    b: u8,
}

#[get("/")]
const fn beacon() -> &'static str {
    "Welcome to the Beacon Zone 😎"
}

#[get("/approximation?<color..>")]
fn approx(color: ColorQuery) -> Json<Panes> {
    let rgb = [color.r, color.g, color.b];
    Json(find_combination_default(rgb).unwrap())
}
#[get("/approximation/custom?<color..>")]
fn approx_cust(color: ColorQuery, headers: DepthCutoffHeaders) -> Json<Panes> {
    let rgb = [color.r, color.g, color.b];
    Json(find_combination_custom(rgb, headers.depth, headers.cutoff).unwrap())
}
#[options("/approximation/custom")]
fn options_approx_cust() -> Status {
    Status::Ok
}

#[get("/panes?<panes..>")]
fn panes(panes: String) -> Json<PreciseRGB> {
    let mut panes1 = Vec::new();
    for entry in panes.split(',') {
        panes1.push(entry.trim().to_string());
    }
    dbg!(&panes);
    Json(calculate_color_from_panes_default(&panes1))
}

#[get("/colors")]
fn colors() -> Json<Colors> {
    let x = get_standard_colors();
    Json(Colors { x })
}
