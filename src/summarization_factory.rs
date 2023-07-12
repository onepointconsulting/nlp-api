use rust_bert::pipelines::common::{ModelResource, ModelType};
use rust_bert::resources::RemoteResource;
use rust_bert::bart::{BartConfigResources, BartMergesResources, BartModelResources, BartVocabResources};
use rust_bert::pipelines::summarization::SummarizationConfig;
use rust_bert::pegasus::{PegasusConfigResources, PegasusModelResources, PegasusVocabResources};
use rust_bert::prophetnet::{ProphetNetConfigResources, ProphetNetModelResources, ProphetNetVocabResources};
use rust_bert::longt5::{LongT5ConfigResources, LongT5ModelResources, LongT5VocabResources};

pub struct SummarizationConfigFactory;

impl SummarizationConfigFactory {
    pub fn distil_bart() -> SummarizationConfig {
        SummarizationConfig::new(
            ModelType::Bart,
            ModelResource::Torch(Box::new(RemoteResource::from_pretrained(
                BartModelResources::DISTILBART_CNN_6_6,
            ))),
            RemoteResource::from_pretrained(BartConfigResources::DISTILBART_CNN_6_6),
            RemoteResource::from_pretrained(BartVocabResources::DISTILBART_CNN_6_6),
            Some(RemoteResource::from_pretrained(BartMergesResources::DISTILBART_CNN_6_6)),
        )
    }
    pub fn pegasus() -> SummarizationConfig {
        SummarizationConfig::new(
            ModelType::Pegasus,
            ModelResource::Torch(Box::new(RemoteResource::from_pretrained(
                PegasusModelResources::CNN_DAILYMAIL,
            ))),
            RemoteResource::from_pretrained(PegasusConfigResources::CNN_DAILYMAIL),
            RemoteResource::from_pretrained(PegasusVocabResources::CNN_DAILYMAIL),
            None)
    }
    pub fn prophetnet() -> SummarizationConfig {
        SummarizationConfig::new(
            ModelType::ProphetNet,
            ModelResource::Torch(Box::new(RemoteResource::from_pretrained(
                ProphetNetModelResources::PROPHETNET_LARGE_UNCASED,
            ))),
            RemoteResource::from_pretrained(ProphetNetConfigResources::PROPHETNET_LARGE_UNCASED),
            RemoteResource::from_pretrained(ProphetNetVocabResources::PROPHETNET_LARGE_UNCASED),
            None)
    }
    pub fn long_t5() -> SummarizationConfig {
        SummarizationConfig::new(
            ModelType::LongT5,
            ModelResource::Torch(Box::new(RemoteResource::from_pretrained(
                LongT5ModelResources::TGLOBAL_BASE_BOOK_SUMMARY,
            ))),
            RemoteResource::from_pretrained(LongT5ConfigResources::TGLOBAL_BASE_BOOK_SUMMARY),
            RemoteResource::from_pretrained(LongT5VocabResources::TGLOBAL_BASE_BOOK_SUMMARY),
            None)
    }
}