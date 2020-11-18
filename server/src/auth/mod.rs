use chrono::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    #[serde(with = "jwt_numeric_date")]
    pub exp: DateTime<Utc>,
}

impl Claims {
    pub fn new(sub: String, company: String, exp: DateTime<Utc>) -> Self {
        let exp = exp
            .date()
            .and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0);
        Self { sub, company, exp }
    }
}

mod jwt_numeric_date {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};
    /// Serializes a DateTime<Utc> to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
            .single() // If there are multiple or no valid DateTimes from timestamp, return None
            .ok_or_else(|| serde::de::Error::custom("invalid Unix timestamp value"))
    }
}

pub fn validate_token(token: &String) -> Result<bool, Box<dyn std::error::Error>> {
    let secret = &std::env::var("JWT_SECRET").expect("JWT SECRET not set");

    let _token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(true)
}

pub fn decode_token(token: &String) -> String {
    let secret = &std::env::var("JWT_SECRET").expect("JWT SECRET not set");

    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .unwrap();
    token_data.claims.company
}

pub fn create_token(user: &String, duration: i64) -> Result<String, Box<dyn std::error::Error>> {
    let sub = "user".to_string();
    let company = user.to_string();
    let exp = Utc::now() + chrono::Duration::days(duration);
    let secret = &std::env::var("JWT_SECRET").expect("JWT SECRET not set");

    let claims = Claims::new(sub, company, exp);

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(token)
}
