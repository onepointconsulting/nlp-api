use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;

use actix_web::web;
use rust_bert::pipelines::conversation::{ConversationManager, ConversationModel};
use rust_bert::pipelines::keywords_extraction::{Keyword, KeywordExtractionConfig, KeywordExtractionModel, KeywordScorerType};
use rust_bert::pipelines::sentence_embeddings::{SentenceEmbeddingsConfig, SentenceEmbeddingsModelType};
use rust_bert::pipelines::sequence_classification::Label;
use rust_bert::pipelines::summarization::{SummarizationModel};
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;
use rust_bert::RustBertError;
use threadpool::ThreadPool;

use crate::KeywordExtractionRequest;
use crate::summarization_factory::SummarizationConfigFactory;

#[derive(Debug)]
pub(crate) enum SupportedLanguage {
    Fr,
    Hi,
    Pt,
    En,
    De,
    Nl,
}

impl FromStr for SupportedLanguage {
    type Err = ();

    fn from_str(input: &str) -> Result<SupportedLanguage, Self::Err> {
        match input {
            "fr" => Ok(SupportedLanguage::Fr),
            "pt" => Ok(SupportedLanguage::Pt),
            "hi" => Ok(SupportedLanguage::Hi),
            "de" => Ok(SupportedLanguage::De),
            "nl" => Ok(SupportedLanguage::Nl),
            "en" => Ok(SupportedLanguage::En),
            _ => Err(()),
        }
    }
}

pub(crate) async fn translate_input(target_language: SupportedLanguage,
                                    source_language: SupportedLanguage,
                                    input: String) -> Result<String, RustBertError> {
    println!("Converting from {:?} to {:?}", source_language, target_language);

    return thread::spawn(move || {
        let source_lang = convert_language(source_language);
        let selected_lang = convert_language(target_language);

        let model = TranslationModelBuilder::new()
            .with_source_languages(vec![source_lang])
            .with_target_languages(vec![selected_lang])
            .create_model()?;
        let splits = split_text(input.clone());
        let output = model.translate(&splits, None,
                                     selected_lang)?;

        Ok(output.join(""))
    }).join().expect("Failed");
}

fn split_text(input: String) -> Vec<String> {
    let mut vec: Vec<String> = Vec::new();
    let string_bytes = input.as_bytes();
    let mut start = 0;
    for (i, c) in input.char_indices() {
        if (c == '.' && (i < input.len() - 1 && (string_bytes[i + 1] as char) != '.' && !(string_bytes[i + 1] as char).is_numeric())) ||
            (i == input.len() - 1) {
            let slice = &input[start..i + 1].trim().to_string();
            println!("{}", slice);
            vec.push(slice.clone());
            start = i + 1;
        }
    }
    if vec.len() == 0 {
        vec.push(input);
    }
    return vec;
}

pub async fn zero_shot_classification(input: String, split: bool, labels: &Vec<String>)
                                      -> Result<(Vec<String>, Vec<Vec<Label>>), RustBertError> {
    let label_copy: Vec<String> = labels.iter().map(|l| l.clone()).collect();
    return thread::spawn(move || {
        let vec = handle_split(input, split);
        let splits: Vec<&str> = vec.iter().map(|s| s.as_str()).collect();

        let sequence_classification_model = ZeroShotClassificationModel::new(Default::default())?;

        let candidate_labels: Vec<&str> = label_copy.iter().map(|s| s.as_str()).collect();
        let output = sequence_classification_model.predict_multilabel(
            &splits,
            candidate_labels,
            None,
            128,
        );
        return match output {
            Ok(vecs) => {
                let orig = splits.iter().map(|s| s.to_string()).collect();
                let success = Ok((orig, vecs));
                success
            }
            Err(e) => {
                let error = Err(e);
                error
            }
        };
    }).join().expect("Failed zero shot classification");
}

fn handle_split(input: String, split: bool) -> Vec<String> {
    let vec = if split { split_text(input.clone()) } else { vec!(input.clone()) };
    vec
}

struct KeywordConfigFactory;

impl KeywordConfigFactory {
    fn variable_keyword_number<'a>(how_many: usize) -> KeywordExtractionConfig<'a> {
        let sentence_embeddings_config =
            SentenceEmbeddingsConfig::from(SentenceEmbeddingsModelType::AllMiniLmL6V2);

        KeywordExtractionConfig {
            sentence_embeddings_config,
            tokenizer_stopwords: None,
            tokenizer_pattern: None,
            scorer_type: KeywordScorerType::CosineSimilarity,
            ngram_range: (1, 1),
            num_keywords: how_many,
            diversity: None,
            max_sum_candidates: None,
        }
    }

    fn variable_keyword_number_ngram<'a>(how_many: usize, ngram_range: (usize, usize)) -> KeywordExtractionConfig<'a> {
        let sentence_embeddings_config =
            SentenceEmbeddingsConfig::from(SentenceEmbeddingsModelType::AllMiniLmL6V2);

        KeywordExtractionConfig {
            sentence_embeddings_config,
            tokenizer_stopwords: None,
            tokenizer_pattern: None,
            scorer_type: KeywordScorerType::CosineSimilarity,
            ngram_range,
            num_keywords: how_many,
            diversity: None,
            max_sum_candidates: None,
        }
    }
}

pub async fn keyword_extraction(
    request: web::Json<KeywordExtractionRequest>, pool: web::Data<ThreadPool>) ->
Result<Vec<Vec<Keyword>>, RustBertError> {
    let input: String = request.orig_text.clone();
    let split: bool = request.split;
    let how_many_option: Option<usize> = request.how_many;
    let ngram_range: Option<(usize, usize)> = request.ngram_range;

    let (tx, rx) = channel();
    const DEFAULT_HOW_MANY: usize = 5;
    pool.execute(move || {
        let vec = handle_split(input, split);
        let splits: Vec<&str> = vec.iter().map(|s| s.as_str()).collect();
        let keyword_extraction_model = if how_many_option.is_some() && ngram_range.is_some() {
            KeywordConfigFactory::variable_keyword_number_ngram(
                how_many_option.unwrap(), ngram_range.unwrap())
        } else if how_many_option.is_some() {
            KeywordConfigFactory::variable_keyword_number(how_many_option.unwrap())
        } else if ngram_range.is_some() {
            KeywordConfigFactory::variable_keyword_number_ngram(
                DEFAULT_HOW_MANY, ngram_range.unwrap())
        } else {
            Default::default()
        };
        let keyword_extraction_mode_resl =
            KeywordExtractionModel::new(keyword_extraction_model);
        match keyword_extraction_mode_resl {
            Ok(keyword_extraction_model) => {
                let output_result = keyword_extraction_model.predict(&splits);
                match output_result {
                    Ok(output) => {
                        let _ = tx.send(Ok(output));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    });
    rx.recv().unwrap()
}

pub async fn summarization(input_str: String, model_option: &Option<String>, pool: web::Data<ThreadPool>)
    -> Result<String, RustBertError> {
    let model_option_clone = model_option.clone();
    let (tx, rx) = channel();
    pool.execute(move || {
        let config = match model_option_clone {
            Some(s) => {
                match s.as_str() {
                    "distilbart" => SummarizationConfigFactory::distil_bart(),
                    "pegasus" => SummarizationConfigFactory::pegasus(),
                    "prophetnet" => SummarizationConfigFactory::prophetnet(),
                    "long_t5" => SummarizationConfigFactory::long_t5(),
                    _ => Default::default()
                }
            }
            None => {
                Default::default()
            }
        };
        let summarization_model_result = SummarizationModel::new(config);
        match summarization_model_result {
            Ok(summarization_model) => {
                let output = summarization_model.summarize(&[input_str]);
                let _ = tx.send(Ok(output.join(" ").clone()));
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }

    });
    rx.recv().unwrap()
}

pub async fn dialogue(input_str: String, pool: web::Data<ThreadPool>)
    -> Result<String, RustBertError> {
    let (tx, rx) = channel();
    pool.execute(move || {
        let conversation_model_res = ConversationModel::new(Default::default());
        match conversation_model_res {
            Ok(conversation_model) => {
                let mut conversation_manager = ConversationManager::new();
                let conversation_id = conversation_manager.create(input_str.as_str());
                let map = conversation_model.generate_responses(&mut conversation_manager);
                let string_list = map.iter().map(|kv| kv.1.to_string()).collect::<Vec<String>>();
                let _ = tx.send(Ok(string_list.join(" ")));
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    });
    rx.recv().unwrap()
}

fn convert_language(language: SupportedLanguage) -> Language {
    return match language {
        SupportedLanguage::Fr => Language::French,
        SupportedLanguage::Hi => Language::Hindi,
        SupportedLanguage::Pt => Language::Portuguese,
        SupportedLanguage::En => Language::English,
        SupportedLanguage::De => Language::German,
        SupportedLanguage::Nl => Language::Dutch
    };
}
