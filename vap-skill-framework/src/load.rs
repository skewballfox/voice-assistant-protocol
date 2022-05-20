use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use toml::de::from_str;
use unic_langid::LanguageIdentifier;
use vap_common_skill::structures::{
    msg_register_intents::{
        NluData, NluDataEntity, NluDataEntityData, NluDataIntent, NluDataIntentUtterance,
        NluDataSlot,
    },
    Language,
};

pub fn list_langs<P>(intents: P) -> Vec<LanguageIdentifier> where P: AsRef<Path> {
    let folder = intents.as_ref();
    folder.read_dir().unwrap().into_iter().filter_map(|r| {
        r.unwrap().file_name().to_str().unwrap().parse().ok()
    }).collect()
}

pub fn load_intents<P>(langs: &[&LanguageIdentifier], intents: P) -> Vec<NluData>
where
    P: AsRef<Path>,
{
    let folder = intents.as_ref();
    folder.read_dir().unwrap().into_iter().filter_map(|r| {
        let r = r.unwrap();
        let t = r.file_type().unwrap();
        if t.is_file() {
            let i: LanguageIdentifier = r.file_name().to_str().unwrap().parse().unwrap();

            if langs.contains(&&i) {
                let l: LangData = from_str(&fs::read_to_string(r.path()).unwrap()).unwrap();
                Some(l.into_nlu_data(i.into()))
            }
            else {
                None
            }
        } else {
            None
        }
    }).collect()
}

type ScopeData = HashMap<String, IntentData>;

#[derive(Debug, Deserialize)]
struct LangData {
    #[serde(rename = "intents")]
    scopes: HashMap<String, ScopeData>,

    entities: HashMap<String, EntityData>,
}

impl LangData {
    pub fn into_nlu_data(self, language: Language) -> NluData {
        // Just one scope right now
        let intents = self.scopes["main"]
            .clone()
            .into_iter()
            .map(|(n, i)| i.into_vap(n))
            .collect();
        let entities = self
            .entities
            .into_iter()
            .map(|(n, e)| e.into_vap(n))
            .collect();
        NluData {
            language,
            intents,
            entities,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct IntentData {
    utterances: Vec<String>,
    slots: HashMap<String, String>,
}

impl IntentData {
    fn into_vap(self, name: String) -> NluDataIntent {
        let utterances = self
            .utterances
            .into_iter()
            .map(|u| NluDataIntentUtterance { text: u })
            .collect();

        let slots = self
            .slots
            .into_iter()
            .map(|(n, e)| NluDataSlot { name: n, entity: e })
            .collect();

        NluDataIntent {
            name,
            utterances,
            slots,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EntityData {
    data: Vec<NluDataEntityData>,
}

impl EntityData {
    pub fn into_vap(self, name: String) -> NluDataEntity {
        NluDataEntity {
            name,
            strict: false,
            data: self.data,
        }
    }
}
