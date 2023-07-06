mod nlp;
mod transport_structs;

use std::future::Future;
use std::str::FromStr;
use std::sync::mpsc::channel;
use dotenv::dotenv;
use ::config::Config;
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, web, get, post};
use chrono::{Datelike, Timelike, Utc};
use rust_bert::RustBertError;
use threadpool::ThreadPool;
use crate::config::MainConfig;
use crate::nlp::{dialogue, keyword_extraction, summarization, SupportedLanguage, translate_input, zero_shot_classification};
use crate::transport_structs::{DialogueRequest, ErrorCodes, ExtractionKeyword, ExtractionResponse, Info, KeywordExtractionRequest, SummarizationRequest, SimpleTextResponse, TranslationRequest, TranslationResponse, ZeroShotRequest, ZeroShotResponse};

mod config {
    use serde::Deserialize;

    #[derive(Debug, Default, Deserialize, Clone)]
    pub struct MainConfig {
        pub server_addr: String,
    }
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
                status: String::from(ErrorCodes::STATUS_OK)
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ZeroShotResponse {
                sentences: vec!(request.orig_text.clone()),
                responses: vec!(),
                status: String::from(ErrorCodes::STATUS_FAILED)
            })
        }
    }
}

#[post("/keyword_extraction")]
async fn keyword_extraction_service(request: web::Json<KeywordExtractionRequest>) -> impl Responder {
    let res = keyword_extraction(request);
    match res.await {
        Ok(vec) => {
            let keyword_res = vec.iter()
                .map(|child| child.iter()
                    .map(|k| ExtractionKeyword { text: k.text.clone(), score: k.score}).collect()).collect();
            let extraction_keyword = ExtractionResponse{
                results: keyword_res,
                status: String::from(ErrorCodes::STATUS_OK)
            };
            HttpResponse::Ok().json(extraction_keyword)
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ExtractionResponse {
                results: vec![],
                status: String::from(ErrorCodes::STATUS_FAILED)
            })
        }
    }
}

#[post("/summarization")]
async fn summarization_service(request: web::Json<SummarizationRequest>, pool: web::Data<ThreadPool>) -> impl Responder {
    let model_option = &request.model;
    let res = summarization(
        request.orig_text.clone(), model_option, pool
    );
    process_simple_text_response(res).await
}

fn create_simple_text_error(e: RustBertError) -> HttpResponse {
    HttpResponse::InternalServerError().json(SimpleTextResponse {
        text: format!("{:?}", e),
        status: ErrorCodes::STATUS_FAILED.to_string()
    })
}

#[post("/dialogue")]
async fn dialogue_service(request: web::Json<DialogueRequest>, pool: web::Data<ThreadPool>) -> impl Responder {
    let res = dialogue(
        request.question.clone(), pool
    );
    return process_simple_text_response(res).await;
}

async fn process_simple_text_response(res: impl Future<Output=Result<String, RustBertError>>) -> HttpResponse {
    match res.await {
        Ok(s) => {
            HttpResponse::Ok().json(SimpleTextResponse {
                text: s,
                status: ErrorCodes::STATUS_OK.to_string()
            })
        }
        Err(e) => {
            create_simple_text_error(e)
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
async fn index(config: web::Data<MainConfig>, pool: web::Data<ThreadPool>) -> impl Responder {
    let timestamp = create_timestamp();
    let (tx, rx) = channel();
    pool.execute(move || {
        tx.send("Welcome to NLP API!".to_string()).expect("channel will be there waiting for the pool")
    });
    let msg = rx.recv().unwrap();
    HttpResponse::Ok().json(Info {
        message: msg,
        server_address: config.server_addr.clone(),
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
        let num_workers = 4;
        let pool = ThreadPool::new(num_workers);
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool))
            .service(index)
            .service(summarization_service)
            .service(translate)
            .service(zero_shot_classification_service)
            .service(keyword_extraction_service)
            .service(dialogue_service)
    })
        .bind(server_addr)?
        .run()
        .await
}
