#[macro_use]
extern crate rocket;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use ed25519_dalek::SigningKey;
use rocket::{request::FromParam, State};
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex};

use rocket::serde::json::Json;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Username(String);

impl<'r> FromParam<'r> for Username {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(Username(param.to_string()))
    }
}

struct AppState {
    db: Mutex<HashMap<Username, SigningKey>>,
}

#[derive(Default, Debug, Serialize)]
pub struct KeyDetails {
    pub verification: String,
    pub signing: String,
}

#[derive(Default, Debug, Serialize)]
pub struct SignerDetails {
    pub verification_url: String,
    pub verification_key_b64: String,
    pub metadata: HashMap<String, String>,
}

#[get("/generate_key/<username>")]
fn generate_key(username: Username, state: &State<AppState>) -> Result<Json<KeyDetails>, String> {
    let mut csprng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let user_exists = state.db.lock().unwrap().contains_key(&username);
    if user_exists {
        return Err(format!("Username {:?} already exists", username.0));
    }
    state
        .db
        .lock()
        .unwrap()
        .insert(username, signing_key.clone());

    let verification_b64 = URL_SAFE.encode(signing_key.verifying_key().to_bytes());
    let signing_b64 = URL_SAFE.encode(signing_key.to_bytes());

    Ok(Json(KeyDetails {
        verification: verification_b64,
        signing: signing_b64,
    }))
}

#[get("/provenance/<username>")]
fn provenance(username: Username, state: &State<AppState>) -> Result<Json<SignerDetails>, String> {
    let base_url = "http://localhost:8000";

    let binding = state.db.lock().unwrap();
    let Some(signing_key) = binding.get(&username) else {
        return Err(format!("Username {:?} not found", username.0));
    };
    let verification_key_b64 = URL_SAFE.encode(signing_key.verifying_key().to_bytes());

    let mut metadata: HashMap<String, String> = HashMap::new();
    metadata.insert("username".to_string(), username.clone().0);

    Ok(Json(SignerDetails {
        verification_url: format!("{base_url}/{}/provenance", username.0),
        verification_key_b64,
        metadata,
    }))
}

#[launch]
fn rocket() -> _ {
    let db = Mutex::new(HashMap::new());

    // Keep a constant base64 signing key for the user beyarkay for testing purposes
    let base64_signing_key = "-5TaFC0xFOj_hf7mlvVaLKKpVFTaXUrLDzRqaaf7gFw=";
    println!("Signing key for user `beyarkay`: {:?}", base64_signing_key);
    // Decode the base64 string
    let binding: Vec<u8> = URL_SAFE.decode(base64_signing_key.as_bytes()).unwrap();
    // Convert the vec to a slice
    let decoded_base64_slice: &[u8] = binding.as_slice();
    // Convert the unknown-length slice into a correct-length slice
    let correct_length_slice = decoded_base64_slice.try_into().unwrap();
    // Convert the correct-length slice into a SigningKey
    let signing_key: SigningKey = SigningKey::from_bytes(correct_length_slice);
    // Add the signing key to the DB
    db.lock()
        .unwrap()
        .insert(Username("beyarkay".to_string()), signing_key.clone());

    let state = AppState { db };

    rocket::build()
        .manage(state)
        .mount("/", routes![provenance, generate_key])
}
