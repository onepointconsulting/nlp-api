use std::str::FromStr;
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use rust_bert::RustBertError;
use std::thread;

pub(crate) enum SupportedLanguage {
    Fr,
    Hi,
    Pt,
    En,
    De
}

impl FromStr for SupportedLanguage {

    type Err = ();

    fn from_str(input: &str) -> Result<SupportedLanguage, Self::Err> {
        match input {
            "fr" => Ok(SupportedLanguage::Fr),
            "pt" => Ok(SupportedLanguage::Pt),
            "hi" => Ok(SupportedLanguage::Hi),
            "de" => Ok(SupportedLanguage::De),
            "en" => Ok(SupportedLanguage::En),
            _      => Err(()),
        }
    }
}

pub(crate) async fn translate_input(language: SupportedLanguage, input: String) -> Result<String, RustBertError> {

    return thread::spawn(move || {
        let selected_lang = match language {
            SupportedLanguage::Fr => Language::French,
            SupportedLanguage::Hi => Language::Hindi,
            SupportedLanguage::Pt => Language::Portuguese,
            SupportedLanguage::En => Language::English,
            SupportedLanguage::De => Language::German
        };

        if selected_lang == Language::English {
            return Ok(input.to_string())
        }

        let model = TranslationModelBuilder::new()
            .with_source_languages(vec![Language::English])
            .with_target_languages(vec![selected_lang])
            .create_model()?;
        let output = model.translate(&[input], None,
                                     selected_lang)?;

        Ok(output.join("\n"))
    }).join().expect("Failed");

}
