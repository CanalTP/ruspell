use ispell;
use errors::{Result, ResultExt};
use utils;
use std::collections::BTreeSet;

struct SpellCache {
    name: String,
    errors: Vec<ispell::IspellError>,
}
impl SpellCache {
    fn new(checker: &mut ispell::SpellChecker, name: &str) -> Result<Self> {
        Ok(SpellCache {
            name: name.to_string(),
            errors: checker.check(name).chain_err(|| "Could not perform check using aspell")?,
        })
    }
}

pub struct SpellCheck {
    aspell: ispell::SpellChecker,
    cache: Option<SpellCache>,
}
impl SpellCheck {
    pub fn new(dict: &str) -> Result<Self> {
        Ok(SpellCheck {
            aspell: ispell::SpellLauncher::new().aspell()
                .dictionary(dict)
                .timeout(10000)
                .launch()?,
            cache: None,
        })
    }

    pub fn add_word(&mut self, new_word: &str) -> Result<()> {
        Ok(self.aspell.add_word(new_word)?)
    }

    fn get_ispell_errors(&mut self, word: &str) -> Result<&[ispell::IspellError]> {
        if self.cache.as_ref().map_or(true, |cache| cache.name != word) {
            self.cache = Some(SpellCache::new(&mut self.aspell, word)?);
        }
        Ok(&self.cache
            .as_ref()
            .unwrap()
            .errors)
    }

    // check for the presence of the same word, no matter the case
    pub fn has_same_accent_word(&mut self, word: &str) -> Result<bool> {
        let misspelt_errors = self.get_ispell_errors(word)?;

        if misspelt_errors.is_empty() {
            return Ok(true);
        }

        let lower_case_w = word.to_lowercase();
        for e in misspelt_errors {
            if e.suggestions.iter().any(|s| lower_case_w == s.to_lowercase()) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // check for the presence of the same word, no matter accent or case
    pub fn has_competitor_word(&mut self, word: &str) -> Result<bool> {
        let misspelt_errors = self.get_ispell_errors(word)?;

        if misspelt_errors.is_empty() {
            return Ok(true);
        }

        let normed_w = utils::normed(word);
        for e in misspelt_errors {
            if e.suggestions.iter().any(|s| normed_w == utils::normed(s)) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn process(&mut self, name: &str) -> Result<String> {
        let misspelt_errors = self.get_ispell_errors(name)?;

        let mut new_name = name.to_string();

        for e in misspelt_errors.iter().filter(|e| !utils::has_accent(&e.misspelled)) {
            let normed_miss = utils::normed(&e.misspelled);
            // set_lowercase just helps ignoring concurrence between
            // suggestions differing just by case
            let mut set_lowercase = BTreeSet::new();
            let valid_suggestions: Vec<_> = e.suggestions
                .iter()
                .filter(|s| {
                    normed_miss == utils::normed(s) && set_lowercase.insert(s.to_lowercase())
                })
                .collect();
            if valid_suggestions.len() == 1 && utils::has_accent(valid_suggestions[0]) {
                new_name = new_name.replace(&e.misspelled, valid_suggestions[0]);
            } else if valid_suggestions.len() > 1 {
                println!("Aspell ambiguous suggestions for {} : {:?}",
                         e.misspelled,
                         valid_suggestions);
            }
        }
        Ok(new_name)
    }
}
