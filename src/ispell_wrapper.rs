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

    pub fn check(&mut self, name: &str) -> Result<String> {

        let errors_res = self.aspell.check(name);
        let mut new_name = name.to_string();

        let misspelt_errors = errors_res.chain_err(|| "Could not perform check using aspell")?;

        for e in misspelt_errors {
            let valid_suggestions: Vec<_> = e.suggestions
                .iter()
                .filter(|s| is_suggestion_qualified(&e.misspelled, s))
                .collect();
            if valid_suggestions.len() == 1 {
                self.nb_replace += 1;
                new_name = new_name.replace(&e.misspelled, &valid_suggestions[0]);
            } else if valid_suggestions.len() > 1 {
                println!("Aspell ambiguous suggestions for {} : {:?}",
                         e.misspelled,
                         valid_suggestions);
            }
        }
        Ok(new_name)
    }
}

fn is_suggestion_qualified(original: &str, suggestion: &str) -> bool {
    !utils::has_accent(original) && utils::normed(original) == utils::normed(suggestion) &&
    utils::has_accent(suggestion)
}
