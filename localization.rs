use std::{collections::HashMap, fs, path::Path};
use serde::Deserialize;
use regex::Regex;
use crate::registries::ID;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TranslationID {
    pub namespace: String,
    pub category: String,
    pub name: String,
}

impl TranslationID {
    pub fn new(namespace: &str, category: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            category: category.to_string(),
            name: name.to_string(),
        }
    }

    pub fn from_id(id: &ID, c: &str) -> Self {
        // ID:  "namespace:name"
        // TID: "namespace:category.name"
        let parts: Vec<&str> = id.to_string().splitn(2, ':').collect();
        return Self {
            namespace: parts[0].to_string(),
            category: c.to_string(),
            name: parts[1].to_string(),
        };
    }
}

impl From<&str> for TranslationID {
    /// Format: "namespace:category.name"
    fn from(value: &str) -> Self {
        let parts: Vec<&str> = value.splitn(2, ':').collect();
        if parts.len() == 2 {
            let namespace = parts[0].to_string();
            let category_name: Vec<&str> = parts[1].splitn(2, '.').collect();
            if category_name.len() == 2 {
                return Self {
                    namespace,
                    category: category_name[0].to_string(),
                    name: category_name[1].to_string(),
                };
            }
        }
        panic!("Invalid TranslationID format: '{}'. Expected format: 'namespace:category.name'", value);
    }
}

impl From<String> for TranslationID {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct Language {
    pub name: String,
    pub code: String,
}

#[derive(Clone, Debug)]
pub struct LanguageList {
    pub languages: Vec<Language>,
}

impl LanguageList {
    pub fn new() -> Self {
        Self {
            languages: Vec::new(),
        }
    }

    pub fn is_valid_code(code: &str) -> bool {
        // Regex: zwei kleine Buchstaben, dann '_', dann zwei große Buchstaben
        // Beispiel: "de_DE", "en_US"
        let re = Regex::new(r"^[a-z]{2}_[A-Z]{2}$").unwrap();
        re.is_match(code)
    }

    pub fn add(&mut self, name: &str, code: &str) {
        // Überprüfen, ob die Sprache bereits existiert
        if self.languages.iter().any(|lang| lang.code == code) {
            eprintln!("⚠ Language with code '{}' already exists!", code);
            return;
        }

        self.languages.push(Language {
            name: name.to_string(),
            code: code.to_string(),
        });
    }

    pub fn get(&self, code: &str) -> Option<&Language> {
        if !Self::is_valid_code(code) {
            eprintln!("⚠ Language code '{}' is not valid! Expected format: <xx_XX> (2 lowercase + '_' + 2 uppercase letters)", code);
            return None;
        }
        self.languages.iter().find(|lang| lang.code == code)
    }
}

#[derive(Debug)]
pub struct Translator {
    pub language: Language,
    pub translations: HashMap<TranslationID, String>,
}

impl Translator {
    pub fn is_valid_identifier(identifier: &str) -> bool {
        // Regex: Erlaubt Buchstaben, Zahlen und Unterstriche, muss mit Buchstaben beginnen
        let re = Regex::new(r"^[a-z]{1,16}:[a-z_]{1,16}.[a-z_]{1,64}$").unwrap();
        re.is_match(identifier)
    }

    pub fn load<P: AsRef<Path>>(language: Language, path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        // Kompakte flache Map: key = "namespace.category:name"
        let raw_yaml: HashMap<String, String> = serde_yaml::from_str(&content)?;

        let mut translations = HashMap::new();

        for (key, translation) in raw_yaml {
            if Self::is_valid_identifier(&key) {
                translations.insert(TranslationID::from(key.as_str()), translation);
            } else {
                // Ungültiges Format, überspringen oder Fehler?
                // Hier überspringen:
                eprintln!("Ungültiger Key in Übersetzungen: {}", key);
            }
        }

        Ok(Self { language, translations })
    }

    pub fn translate(&self, id: &TranslationID, vars: Option<&HashMap<&str, &str>>) -> String {
        if let Some(translation) = self.translations.get(id) {
            if let Some(vars) = vars {
                // Platzhalter ersetzen
                let mut result = translation.clone();
                for (key, value) in vars {
                    // Regex für Platzhalter: {key}
                    let re = Regex::new(&format!(r"\{{{}\}}", key)).unwrap();
                    result = re.replace_all(&result, *value).to_string();
                }
                result
            } else {
                translation.clone()
            }
        } else {
            // Fallback: "namespace:category.name" oder "item.category.name"
            format!("{}:{}.{}", id.namespace, id.category, id.name)
        }
    }
}