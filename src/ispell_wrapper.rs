use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
use ispell::SpellLauncher;

pub struct SpellCheck {
    aspell: ::ispell::SpellChecker,
    nb_replace: u32,
    nb_error: u32,
}
impl SpellCheck {
    pub fn new() -> ::std::result::Result<Self, String> {
        if let Ok(aspell_checker) = SpellLauncher::new().aspell().dictionary("fr").launch() {
            Ok(SpellCheck {
                   aspell: aspell_checker,
                   nb_replace: 0,
                   nb_error: 0,
               })
        } else {
            Err("Impossible to launch aspell".to_string())
        }
    }

    pub fn nb_error(&self) -> u32 {
        self.nb_error
    }

    pub fn nb_replace(&self) -> u32 {
        self.nb_replace
    }

    pub fn add_word(&mut self, new_word: &str) -> ::ispell::Result<()> {
        self.aspell.add_word(new_word)
    }

    pub fn check(&mut self, mut name: String) -> String {

        let errors_res = self.aspell.check(&name);
        if let Err(e) = errors_res {
            print!("{:?}", e);
            println!(" ({})", name);
            self.nb_error += 1;
            return name;
        }

        let misspelt_errors = errors_res.unwrap();

        for e in misspelt_errors {
            /*println!("'{}' (pos: {}) is misspelled!", &e.misspelled, e.position);
            if !e.suggestions.is_empty() {
                println!("Maybe you meant : '{:?}'?", &e.suggestions[0]);
            } else {
                println!("Nothing better to offer...");
            }*/
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
                println!("Ambiguous suggestions for {} : {:?}",
                         e.misspelled,
                         valid_suggestions);
            }
        }
        name
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
    // valid IF original word has no accent
    //   AND normalized versions are the same
    //   AND suggestion adds accent
    if original.len() == normed_orig.len() && normed_orig == normed_sugg &&
       original.len() < suggestion.len() {
        /*println!("REPLACE VALID  : {} > {} ({})",
                             &original,
                             &suggestion,
                             name);*/
        true
    } else {
        false
    }
}
