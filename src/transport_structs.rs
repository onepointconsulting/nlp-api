use serde::{Serialize, Deserialize};
use rust_bert::pipelines::sequence_classification::Label;

pub(crate) struct ErrorCodes;

impl ErrorCodes {
    pub const STATUS_OK: &'static str = "OK";
    pub const STATUS_FAILED: &'static str = "Failed";
}

#[derive(Deserialize)]
pub(crate) struct TranslationRequest {
    pub(crate) orig_text: String,
    pub(crate) language: String,
    pub(crate) source_language: Option<String>
}

#[derive(Deserialize)]
pub(crate) struct ZeroShotRequest {
    pub(crate) orig_text: String,
    pub(crate) split: bool,
    pub(crate) labels: Option<Vec<String>>
}

#[derive(Deserialize)]
pub(crate) struct KeywordExtractionRequest {
    pub(crate) orig_text: String,
    pub(crate) split: bool
}

#[derive(Serialize)]
pub(crate) struct ZeroShotResponse {
    pub(crate) sentences: Vec<String>,
    pub(crate) responses: Vec<Vec<Label>>,
    pub(crate) status: String
}

#[derive(Deserialize,Serialize)]
pub(crate) struct TranslationResponse {
    pub(crate) orig_text: String,
    pub(crate) translation: String
}

#[derive(Serialize)]
pub(crate) struct Info {
    pub(crate) message: String,
    pub(crate) timestamp: String,
}

#[derive(Serialize)]
pub(crate) struct ExtractionResponse {
    pub(crate) results: Vec<Vec<ExtractionKeyword>>,
    pub(crate) status: String
}

#[derive(Serialize)]
pub(crate) struct ExtractionKeyword {
    /// Keyword
    pub(crate) text: String,
    /// Similarity score for the keyword
    pub(crate) score: f32,
}

#[derive(Deserialize)]
pub(crate) struct SummarizationRequest {
    pub(crate) orig_text: String,
    pub(crate) model: Option<String>
}

#[derive(Deserialize)]
pub(crate) struct DialogueRequest {
    pub(crate) question: String,
    pub(crate) model: Option<String>
}

#[derive(Serialize)]
pub(crate) struct SimpleTextResponse {
    pub(crate) text: String,
    pub(crate) status: String
}
