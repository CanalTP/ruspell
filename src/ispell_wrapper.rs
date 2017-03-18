use ispell;
use errors::{Result, ResultExt};
use utils;

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
    pub fn new() -> Result<Self> {
        Ok(SpellCheck {
               aspell: ispell::SpellLauncher::new().aspell()
                   .dictionary("fr")
                   .launch()?,
               cache: None,
           })
    }

    pub fn add_word(&mut self, new_word: &str) -> Result<()> {
        Ok(self.aspell.add_word(new_word)?)
    }

    fn check_cache(&mut self, word: &str) -> Result<&Vec<ispell::IspellError>> {
        if self.cache.is_none() ||
           self.cache
               .as_ref()
               .unwrap()
               .name != word {
            self.cache = Some(SpellCache::new(&mut self.aspell, word)?);
        }
        Ok(&self.cache
                .as_ref()
                .unwrap()
                .errors)
    }

    // check for the presence of the same word, no matter the case
    pub fn has_same_accent_word(&mut self, word: &str) -> Result<bool> {
        let misspelt_errors = self.check_cache(word)?;

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
        let misspelt_errors = self.check_cache(word)?;

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

    pub fn check(&mut self, name: &str) -> Result<String> {
        let misspelt_errors = self.check_cache(name)?;

        let mut new_name = name.to_string();

        for e in misspelt_errors.iter().filter(|e| !utils::has_accent(&e.misspelled)) {
            let normed_miss = utils::normed(&e.misspelled);
            let valid_suggestions: Vec<_> = e.suggestions
                .iter()
                .filter(|s| normed_miss == utils::normed(s))
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
