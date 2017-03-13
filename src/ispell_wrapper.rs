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
        if let Ok(aspell_checker) =
            SpellLauncher::new()
                .timeout(1000)
                .aspell()
                .dictionary("fr")
                .launch() {
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

    pub fn check(&mut self, name: String) -> String {

        let errors_res = self.aspell.check(&name);
        if let Err(e) = errors_res {
            print!("{:?}", e);
            println!(" ({})", name);
            self.nb_error += 1;
            return name;
            //errors_res = self.aspell.check(&name);
        }

        let misspelt_errors = errors_res.unwrap();
        /*if misspelt_errors.is_empty() {
            println!("all is ok ({})", name);
        }*/

        for e in misspelt_errors {
            //println!("'{}' (pos: {}) is misspelled!", &e.misspelled, e.position);
            if !e.suggestions.is_empty() {
                //println!("Maybe you meant : '{:?}'?", &e.suggestions[0]);
            } else {
                //println!("Nothing better to offer...");
            }

            if !&e.suggestions.is_empty() {
                let normed_miss: String = e.misspelled
                    .nfkd()
                    .filter(|c| !is_combining_mark(*c))
                    .flat_map(char::to_lowercase)
                    .collect();
                let normed_sugg: String = e.suggestions[0]
                    .nfkd()
                    .filter(|c| !is_combining_mark(*c))
                    .flat_map(char::to_lowercase)
                    .collect();
                // valid
                // IF original word has no accent
                //   AND normalized versions are the same
                //   AND suggestion adds accent
                if e.misspelled.len() == normed_miss.len() && normed_miss == normed_sugg &&
                   e.misspelled.len() < e.suggestions[0].len() {
                    println!("REPLACE VALID  : {} > {} ({})",
                             &e.misspelled,
                             &e.suggestions[0],
                             name);
                    self.nb_replace += 1;
                }
                /*let mut m_c = e.misspelled.chars();
                for s_c in e.suggestions[0].chars() {
                    if s_c >= 128 as char
                }*/
                /*if &e.misspelled.chars().count() == &e.suggestions[0].chars().count() {
                    for (m, s) in &e.misspelled.chars().iter.zip(&e.suggestions[0]
                                                                      .chars()
                                                                      .iter) {}

                }*/
            }

            /*if !&e.suggestions.is_empty() && &e.misspelled.len() == &e.suggestions[0].len() &&
               &e.misspelled.chars().zip(&e.suggestions[0].chars()).all(|(m, s)| {
                                                                            s >= 128 as char ||
                                                                            m == s
                                                                        }) {
                println!("REPLACE VALID: {} > {}", &e.misspelled, &e.suggestions[0]);
            }*/
        }
        name
    }
}



/*use std::thread::sleep;
fn ispell(name: String) -> String {
    let mut checker = SpellLauncher::new()
        .aspell()
        .dictionary("fr_FR")
        .launch()
        .unwrap();
    let errors = checker.check(&name).unwrap();
    for e in errors {
        println!("'{}' (pos: {}) is misspelled!", &e.misspelled, e.position);
        if !e.suggestions.is_empty() {
            println!("Maybe you meant '{}'?", &e.suggestions[0]);
        }
    }
    sleep(std::time::Duration::new(1, 0));
    name
}*/

