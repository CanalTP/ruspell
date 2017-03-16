use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
use ispell;
use errors::{Result, ResultExt};

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

    pub fn check(&mut self, mut name: String) -> Result<String> {

        let errors_res = self.aspell.check(&name);
        if let Err(e) = errors_res {
            print!("{:?}", e);
            println!(" ({})", name);
            self.nb_error += 1;
            return Ok(name);
        }

        let misspelt_errors = errors_res.chain_err(|| "Could not perform check using aspell")?;

        for e in misspelt_errors {
            let mut valid_suggestions = vec![];

            for s in e.suggestions {
                if is_suggestion_qualified(&e.misspelled, &s) {
                    valid_suggestions.push(s);
                }
            }
            if valid_suggestions.len() == 1 {
                self.nb_replace += 1;
                name = name.replace(&e.misspelled, &valid_suggestions[0]);
            } else if valid_suggestions.len() > 1 {
                println!("Aspell ambiguous suggestions for {} : {:?}",
                         e.misspelled,
                         valid_suggestions);
            }
        }
        Ok(name)
    }
}

fn is_suggestion_qualified(original: &str, suggestion: &str) -> bool {
    let normed_orig: String = original.nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect();
    let normed_sugg: String = suggestion.nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect();
    // valid IF original word has no accent AND
    //   normalized versions are the same AND
    //   suggestion adds accent
    if original.len() == normed_orig.len() && normed_orig == normed_sugg &&
       original.len() < suggestion.len() {
        true
    } else {
        false
    }
}
