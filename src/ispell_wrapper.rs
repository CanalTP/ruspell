use ispell;
use errors::{Result, ResultExt};
use utils;

pub struct SpellCheck {
    aspell: ispell::SpellChecker,
    nb_replace: u32,
    nb_error: u32,
}
impl SpellCheck {
    pub fn new() -> Result<Self> {
        Ok(SpellCheck {
               aspell: ispell::SpellLauncher::new().aspell()
                   .dictionary("fr")
                   .launch()?,
               nb_replace: 0,
               nb_error: 0,
           })
    }

    pub fn nb_error(&self) -> u32 {
        self.nb_error
    }

    pub fn nb_replace(&self) -> u32 {
        self.nb_replace
    }

    pub fn add_word(&mut self, new_word: &str) -> Result<()> {
        Ok(self.aspell.add_word(new_word)?)
    }

    // check for the presence of the same word, no matter the case
    pub fn has_same_accent_word(&mut self, word: &str) -> Result<bool> {
        let misspelt_errors = self.aspell
            .check(word)
            .chain_err(|| "Could not perform check using aspell")?;

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
        let misspelt_errors = self.aspell
            .check(word)
            .chain_err(|| "Could not perform check using aspell")?;

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
        let misspelt_errors = self.aspell
            .check(name)
            .chain_err(|| "Could not perform check using aspell")?;

        let mut new_name = name.to_string();

        for e in misspelt_errors.iter().filter(|e| !utils::has_accent(&e.misspelled)) {
            let normed_miss = utils::normed(&e.misspelled);
            let valid_suggestions: Vec<_> = e.suggestions
                .iter()
                .filter(|s| normed_miss == utils::normed(s))
                .collect();
            if valid_suggestions.len() == 1 && utils::has_accent(valid_suggestions[0]) {
                self.nb_replace += 1;
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
