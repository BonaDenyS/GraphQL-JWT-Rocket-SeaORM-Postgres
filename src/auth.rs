use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AuthUser(pub Option<i32>);

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

pub fn create_jwt(user_id: i32, secret: &str) -> String {
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;

    let claims = Claims { sub: user_id, exp };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

pub fn validate_jwt(token: &str, secret: &str) -> Option<i32> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;

    Some(data.claims.sub)
}
