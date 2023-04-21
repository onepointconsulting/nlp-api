mod nlp;

use std::str::FromStr;
use dotenv::dotenv;
use ::config::Config;
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, web, get, post};
use chrono::{Datelike, Timelike, Utc};
use crate::config::MainConfig;
use serde::{Serialize, Deserialize};
use crate::nlp::{SupportedLanguage, translate_input};

mod config {
    use serde::Deserialize;

    #[derive(Debug, Default, Deserialize, Clone)]
    pub struct MainConfig {
        pub server_addr: String
    }
}

#[derive(Deserialize)]
struct TranslationRequest {
    orig_text: String,
    language: String
}

#[derive(Deserialize,Serialize)]
struct TranslationResponse {
    orig_text: String,
    translation: String
}

#[derive(Serialize)]
struct Info {
    message: String,
    timestamp: String,
}

#[post("/translate")]
async fn translate(info: web::Json<TranslationRequest>) -> impl Responder {
    let orig_text = &info.orig_text;
    let supported_language_res = SupportedLanguage::from_str(info.language.as_str());
    let supported_language = supported_language_res.unwrap_or(SupportedLanguage::En);
    let res = translate_input(supported_language, info.orig_text.clone());
    match res.await {
        Ok(s) => {
            HttpResponse::Ok().json(TranslationResponse {
                orig_text: orig_text.clone(),
                translation: s
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(TranslationResponse {
                orig_text: orig_text.clone(),
                translation: format!("{:?}", e)
            })
        }
    }
}

fn create_timestamp() -> String {
    let now = Utc::now();
    let (hour, day, month) = (now.hour(), now.day(), now.month());
    let (_, year) = now.year_ce();
    let timestamp = format!("{}-{}-{} {:02}:{:02}:{:02}", year, month, day, hour, now.minute(), now.second());
    timestamp
}

#[get("/")]
async fn index() -> impl Responder {
    let timestamp = create_timestamp();
    HttpResponse::Ok().json(Info {
        message: "Welcome to NLP API!".to_string(),
        timestamp,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();

    let config: MainConfig = config_.try_deserialize().unwrap();

    let server_addr = config.server_addr.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(config.clone()))
            .service(index)
            .service(translate)
    })
        .bind(server_addr)?
        .run()
        .await
}
