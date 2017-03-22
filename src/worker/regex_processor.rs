use regex::{Regex, RegexBuilder};
use utils;
use std::fmt::Write;
use errors::{Result, ResultExt};

pub enum CaseSpecifier {
    Lower,
    Upper,
}
pub struct FixedcaseProcessor {
    regex: Regex,
    case: CaseSpecifier,
}
impl FixedcaseProcessor {
    pub fn new(words: &[String], case: CaseSpecifier) -> Result<Self> {
        let mut regex_str = "^(".to_string();
        for w in words {
            write!(&mut regex_str, "{}|", w)?;
        }
        regex_str.pop();
        regex_str.push_str(")$");
        Ok(FixedcaseProcessor {
            regex: RegexBuilder::new(&regex_str).case_insensitive(true)
                .build()
                .chain_err(|| format!("Problem building the Regex from {}", regex_str))?,
            case: case,
        })
    }
    pub fn process(&self, name: &str) -> String {
        let mut new_name = String::new();
        for word in utils::get_words(name) {
            if self.regex.is_match(word) {
                match self.case {
                    CaseSpecifier::Lower => {
                        new_name.push_str(&word.to_lowercase());
                    }
                    CaseSpecifier::Upper => {
                        new_name.push_str(&word.to_uppercase());
                    }
                }
            } else {
                new_name.push_str(word);
            }
        }
        new_name
    }
}


pub struct RegexReplace {
    from: Regex,
    to: String,
}
impl RegexReplace {
    pub fn new(from: &str, to: &str) -> Result<Self> {
        Ok(RegexReplace {
            from: RegexBuilder::new(from).case_insensitive(true)
                .build()
                .chain_err(|| format!("Problem building the Regex from {}", from))?,
            to: to.to_string(),
        })
    }
    pub fn process(&self, name: &str) -> String {
        self.from.replace_all(name, self.to.as_str()).into_owned()
    }
}


pub struct LogSuspicious {
    regex: Regex,
}
impl LogSuspicious {
    pub fn new(regex: &str) -> Result<Self> {
        Ok(LogSuspicious {
            regex: RegexBuilder::new(regex).case_insensitive(true)
                .build()
                .chain_err(|| format!("Problem building the Regex from {}", regex))?,
        })
    }
    pub fn process(&self, name: &str) {
        for m in self.regex.find_iter(name) {
            println!("Warning: suspicious match {} in name {}", m.as_str(), name);
        }
    }
}
