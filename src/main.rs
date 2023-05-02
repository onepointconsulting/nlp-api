mod nlp;

use std::str::FromStr;
use dotenv::dotenv;
use ::config::Config;
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, web, get, post};
use chrono::{Datelike, Timelike, Utc};
use rust_bert::pipelines::sequence_classification::Label;
use crate::config::MainConfig;
use serde::{Serialize, Deserialize};
use crate::nlp::{keyword_extraction, SupportedLanguage, translate_input, zero_shot_classification};

const STATUS_OK: &'static str = "OK";
const STATUS_FAILED: &'static str = "Failed";

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
    language: String,
    source_language: Option<String>
}

#[derive(Deserialize)]
struct ZeroShotRequest {
    orig_text: String,
    split: bool,
    labels: Option<Vec<String>>
}

#[derive(Deserialize)]
struct KeywordExtractionRequest {
    orig_text: String,
    split: bool
}

#[derive(Serialize)]
struct ZeroShotResponse {
    sentences: Vec<String>,
    responses: Vec<Vec<Label>>,
    status: String
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


#[derive(Serialize)]
struct ExtractionResponse {
    results: Vec<Vec<ExtractionKeyword>>,
    status: String
}


#[derive(Serialize)]
struct ExtractionKeyword {
    /// Keyword
    pub text: String,
    /// Similarity score for the keyword
    pub score: f32,
}

#[post("/translate")]
async fn translate(info: web::Json<TranslationRequest>) -> impl Responder {
    let orig_text = &info.orig_text;
    let souce_language = &info.language;
    let supported_language_res = SupportedLanguage::from_str(souce_language.as_str());
    let supported_language = supported_language_res.unwrap_or(SupportedLanguage::En);
    // If the source language is not found then default to English, otherwise return none and also defaults to English
    let source_language = match &info.source_language {
        Some(l) => SupportedLanguage::from_str(l.as_str()).unwrap_or(SupportedLanguage::En),
        None => SupportedLanguage::En
    };
    let res = translate_input(
        supported_language,
        source_language,
        info.orig_text.clone());
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

#[post("/zero_shot")]
async fn zero_shot_classification_service(request: web::Json<ZeroShotRequest>) -> impl Responder {
    let labels = &request.labels.clone()
        .unwrap_or(vec!["politics", "public health", "economics", "sports", "arts"].iter()
            .map(|s| s.to_string()).collect());
    let res = zero_shot_classification(
        request.orig_text.clone(),
        request.split,
        labels
    );
    match res.await {
        Ok(vecs) => {
            let (sentences, responses) = vecs;
            HttpResponse::Ok().json(ZeroShotResponse {
                sentences,
                responses,
                status: String::from(STATUS_OK)
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ZeroShotResponse {
                sentences: vec!(request.orig_text.clone()),
                responses: vec!(),
                status: String::from(STATUS_FAILED)
            })
        }
    }
}

#[post("/keyword_extraction")]
async fn keyword_extraction_service(request: web::Json<KeywordExtractionRequest>) -> impl Responder {
    let res = keyword_extraction(request.orig_text.clone(), request.split);
    match res.await {
        Ok(vec) => {
            let keyword_res = vec.iter()
                .map(|child| child.iter()
                    .map(|k| ExtractionKeyword { text: k.text.clone(), score: k.score}).collect()).collect();
            let extraction_keyword = ExtractionResponse{
                results: keyword_res,
                status: String::from(STATUS_OK)
            };
            HttpResponse::Ok().json(extraction_keyword)
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ExtractionResponse {
                results: vec![],
                status: String::from(STATUS_FAILED)
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
            .service(zero_shot_classification_service)
            .service(keyword_extraction_service)
    })
        .bind(server_addr)?
        .run()
        .await
}
