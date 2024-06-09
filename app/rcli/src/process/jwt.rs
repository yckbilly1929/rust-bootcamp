use std::{
    io::Read,
    ops::Add,
    time::{Duration, SystemTime},
};

use anyhow::{Ok, Result};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    sub: String,
    aud: String,
    exp: u64,
}

pub fn process_jwt_sign(
    reader: &mut dyn Read,
    alg: Algorithm,
    sub: &str,
    aud: &str,
    exp: Duration,
) -> Result<String> {
    let mut key_buf = Vec::new();
    reader.read_to_end(&mut key_buf)?;

    let key = match alg {
        Algorithm::HS256 => EncodingKey::from_secret(key_buf.as_slice()),
        Algorithm::EdDSA => EncodingKey::from_ed_pem(key_buf.as_slice())?,
        _ => panic!("algorithm not yet supported"),
    };

    let exp_after = SystemTime::now()
        .add(exp)
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("already expired")
        .as_secs();

    let claims = Claim {
        sub: sub.to_string(),
        aud: aud.to_string(),
        exp: exp_after,
    };

    let token = jsonwebtoken::encode(&Header::new(alg), &claims, &key)?;
    Ok(token)
}

pub fn process_jwt_verify(
    reader: &mut dyn Read,
    alg: Algorithm,
    token: &str,
    sub: &str,
    aud: &str,
) -> Result<()> {
    let mut key_buf = Vec::new();
    reader.read_to_end(&mut key_buf)?;

    let key = match alg {
        Algorithm::HS256 => DecodingKey::from_secret(key_buf.as_slice()),
        Algorithm::EdDSA => DecodingKey::from_ed_pem(key_buf.as_slice())?,
        _ => panic!("algorithm not yet supported"),
    };

    // TODO: validate sub, aud with set_required_spec_claims, set_audience
    let mut validation = Validation::new(alg);
    if !sub.is_empty() {
        validation.sub = Some(sub.to_string());
    }
    if aud.is_empty() {
        validation.validate_aud = false;
    } else {
        validation.set_audience(&[aud]);
    }

    let verified_token = jsonwebtoken::decode::<Claim>(token, &key, &validation)?;
    eprintln!("claims={:?}", verified_token.claims);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use humantime::parse_duration;

    use super::*;

    const PRIV_KEY: &[u8] = include_bytes!("../../fixtures/ed25519.pem");
    const PUB_KEY: &[u8] = include_bytes!("../../fixtures/ed25519_pub.pem");
    const SECRET_KEY: &[u8] = include_bytes!("../../fixtures/blake3.txt");

    #[test]
    fn test_process_jwt_encode_verify_ed25519_pkcs8() -> Result<()> {
        let mut priv_key = Cursor::new(PRIV_KEY);
        let mut pub_key = Cursor::new(PUB_KEY);
        let alg = Algorithm::EdDSA;
        let sub = "acme";
        let aud = "device1";
        let exp = parse_duration("14d")?;

        let token = process_jwt_sign(&mut priv_key, alg, sub, aud, exp)?;
        process_jwt_verify(&mut pub_key, alg, &token, sub, aud)?;

        Ok(())
    }

    #[test]
    fn test_process_jwt_encode_verify_hs256() -> Result<()> {
        let mut secret_key = Cursor::new(SECRET_KEY);
        let mut secret_key_clone = secret_key.clone();
        let alg = Algorithm::HS256;
        let sub = "acme";
        let aud = "device1";
        let exp = parse_duration("14d")?;

        let token = process_jwt_sign(&mut secret_key, alg, sub, aud, exp)?;
        process_jwt_verify(&mut secret_key_clone, alg, &token, sub, aud)?;

        Ok(())
    }
}
