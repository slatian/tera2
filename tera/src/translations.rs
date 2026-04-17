use std::{collections::HashMap, fmt::Display, sync::Arc};

use crate::{Error, Kwargs, State, TeraResult, Value, value::FunctionResult};


/// The translation function type definition
pub trait Translator<Res>: Sync + Send + 'static {

	/// The translation function call
    fn call(&self, message: &str, kwargs: Option<Kwargs>, state: &State) -> Res;

    /// Whether the current translators's output should be treated as safe, defaults to `false`
    fn is_safe(&self) -> bool {
        false
    }

}

type TranslationFunc = dyn Fn(&str, Option<Kwargs>, &State) -> TeraResult<Value> + Sync + Send + 'static;

#[derive(Clone)]
pub(crate) struct StoredTranslator {
    func: Arc<TranslationFunc>,
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
    pub fn new<Trans, Res>(t: Trans) -> Self
    where
        Trans: Translator<Res> + for<'a> Translator<Res>,
        Res: FunctionResult,
    {
        let is_safe = Translator::<Res>::is_safe(&t);
        let closure = move |message: &str, kwargs, state: &State| -> TeraResult<Value> {
            t.call(message, kwargs, state).into_result()
        };

        StoredTranslator {
            func: Arc::new(closure),
            is_safe,
        }
    }

    pub fn call(&self, message: &str, kwargs: Option<Kwargs>, state: &State) -> TeraResult<Value> {
        (self.func)(message, kwargs, state)
    }

    pub fn is_safe(&self) -> bool {
        self.is_safe
    }
}


impl<V> Translator<TeraResult<String>> for HashMap<String, V>
where V: Display + Sync + Send + 'static {
    fn call(&self, message: &str, _kwargs: Option<Kwargs>, _state: &State) -> TeraResult<String> {
	    // TODO: Improve error handling
        self.get(message)
	        .map(|s| s.to_string())
	        .ok_or_else(|| Error::message(format!("Translation failed: message id {message:?} not found")))
    }
}
