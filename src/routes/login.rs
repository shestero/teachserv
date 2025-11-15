use serde::Deserialize;

use time::OffsetDateTime;
use crate::captcha_secret;
use base64::Engine;
use sha2::Sha256;
use sha2::Digest;
use hmac::{Hmac, Mac};
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct Login {
    pub login: String,
    password: String,
    token: Option<String>,
    captcha: Option<String>,
}

impl Login {
    pub fn check_password(&self, password: &String) -> bool {
        self.password == *password
    }

    /// Проверяет подписанный токен и сравнивает с user_answer.
    /// Возвращает Ok(()) если подпись валидна, токен не просрочен и хэш ответа совпадает.
    pub fn verify_signed_token(token_b64: &str, user_answer: &str) -> Result<(), String> {
        let token_bytes = base64::engine::general_purpose::STANDARD
            .decode(token_b64)
            .map_err(|_| "invalid token base64".to_string())?;

        // payload size: nonce(8) + expiry(8) + answer_hash(32) = 48
        if token_bytes.len() < 48 + 32 { // 32 — сигнатура
            return Err("token too short".into());
        }
        let payload_len = 8 + 8 + 32;
        let (payload, signature) = token_bytes.split_at(payload_len);

        // проверка подписи
        let mut mac = HmacSha256::new_from_slice(&*captcha_secret).map_err(|_| "hmac error")?;
        mac.update(payload);
        mac.verify_slice(signature).map_err(|_| "invalid signature".to_string())?;

        // извлекаем expiry и answer_hash
        let expiry_bytes = &payload[8..16];
        let expiry_unix = i64::from_be_bytes(expiry_bytes.try_into().unwrap());
        let expiry = OffsetDateTime::from_unix_timestamp(expiry_unix)
            .map_err(|_| "invalid expiry".to_string())?;

        if OffsetDateTime::now_utc() > expiry {
            return Err("token expired".into());
        }

        let stored_answer_hash = &payload[16..48];
        let user_hash = sha2::Sha256::digest(user_answer.as_bytes());

        if stored_answer_hash != user_hash.as_slice() {
            return Err("wrong answer".into());
        }

        Ok(())
    }

    // None means ok
    pub fn check_captcha(&self) -> Option<String> {
        if let Some(token) = &self.token && let Some(captcha) = &self.captcha {
            Self::verify_signed_token(&token, &captcha).err()
        } else {
            Some("Captcha required!".to_string())
        }
        /*
        match (self.token.clone(), self.captcha.clone()) {
            (Some(token), Some(captcha)) =>
                Self::verify_signed_token(&token, &captcha).err(),
            _ =>
                Some("Captcha required!".to_string())
        }
        */
    }
}