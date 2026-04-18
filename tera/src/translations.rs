use std::{collections::HashMap, fmt::Display, sync::Arc};

use crate::{Error, Kwargs, State, TeraResult, Value};

/// Weather the translator can handle a given message
pub enum TranslationMessageAvailability {
    NotAvailable,
    OnlyWithoutArgs,
    OnlyWithArgs,
    WithAndWithoutArgs,
    Unknown,
}

/// The translation function type definition
pub trait Translator: Sync + Send + 'static {
    /// The translation function call
    fn call(&self, message: &str, kwargs: Option<Kwargs>, state: &State) -> TeraResult<Value>;

    /// Returns weather the translator can handle a given message
    fn is_message_available(&self, _messsage: &str) -> TranslationMessageAvailability {
        TranslationMessageAvailability::Unknown
    }

    /// Whether the current translators's output should be treated as safe, defaults to `false`
    fn is_safe(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub(crate) struct StoredTranslator {
    translator: Arc<dyn Translator>,
    is_safe: bool,
}

impl std::fmt::Debug for StoredTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredFilter")
            .field("is_safe", &self.is_safe)
            .finish_non_exhaustive()
    }
}

impl StoredTranslator {
    pub fn new<Trans>(t: Trans) -> Self
    where
        Trans: Translator,
    {
        let is_safe = Translator::is_safe(&t);

        StoredTranslator {
            translator: Arc::new(t),
            is_safe,
        }
    }

    pub fn call(&self, message: &str, kwargs: Option<Kwargs>, state: &State) -> TeraResult<Value> {
        self.translator.call(message, kwargs, state)
    }

    pub fn is_message_available(&self, message: &str) -> TranslationMessageAvailability {
        self.translator.is_message_available(message)
    }

    pub fn is_safe(&self) -> bool {
        self.is_safe
    }
}

impl<V> Translator for HashMap<String, V>
where
    V: Display + Sync + Send + 'static + Into<Value>,
    Value: for<'a> From<&'a V>,
{
    fn call(&self, message: &str, _kwargs: Option<Kwargs>, _state: &State) -> TeraResult<Value> {
        // TODO: Improve error handling
        self.get(message).map(|s| s.into()).ok_or_else(|| {
            Error::message(format!(
                "Translation failed: message id {message:?} not found"
            ))
        })
    }

    fn is_message_available(&self, message: &str) -> TranslationMessageAvailability {
        if self.contains_key(message) {
            TranslationMessageAvailability::OnlyWithoutArgs
        } else {
            TranslationMessageAvailability::NotAvailable
        }
    }
}
