use std::collections::HashMap;
use regex::{Regex, RegexSet, Captures};
use utils;
use errors::Result;

pub struct RegexProcessor {
    fixed_case_word: HashMap<String, String>,
}
impl RegexProcessor {
    pub fn new() -> Self {
        RegexProcessor { fixed_case_word: HashMap::new() }
    }
    pub fn add_fixed_case(&mut self, fixed: &str) -> Result<()> {
        let fixed_lower = fixed.to_lowercase();
        if self.fixed_case_word.contains_key(&fixed_lower) {
            return Err(format!("Cannot add multiple fixed case words with \
                                same lowercase value : \"{}\" and \"{}\"",
                               self.fixed_case_word[&fixed_lower],
                               fixed)
                               .into());
        }
        self.fixed_case_word.insert(fixed_lower, fixed.to_string());
        Ok(())
    }
    pub fn fix_case(&self, lower_processed: &str, processed: &str, push_on: &mut String) -> () {
        if let Some(fixed) = self.fixed_case_word.get(lower_processed) {
            push_on.push_str(fixed);
        } else {
            push_on.push_str(processed);
        }
    }
}

fn must_be_lower(text: &str) -> bool {
    lazy_static! {
        static ref RE: RegexSet =
            RegexSet::new(&[
                r"(?i)^(\d+([eè]me|[eè]re?|nde?))$",
                ]).unwrap();
    }
    RE.is_match(text)
}

fn must_be_upper(text: &str) -> bool {
    lazy_static! {
        static ref RE: RegexSet =
            RegexSet::new(&[
                r"(?i)^((XL|X{0,3})(IX|IV|V?I{0,3})|\w*\d\w*|RN\d*|RD\d*)$",
                ]).unwrap();
    }
    RE.is_match(text)
}

pub fn fixed_case_word(name: &str, regex: &RegexProcessor) -> String {
    let mut new_name = String::new();
    for word in utils::get_words(name) {
        let lower_word = word.to_lowercase();
        if must_be_lower(word) {
            new_name.push_str(&word.to_lowercase());
        } else if must_be_upper(word) {
            new_name.push_str(&word.to_uppercase());
        } else {
            regex.fix_case(&lower_word, word, &mut new_name);
        }
    }
    new_name
}


pub fn sed_whole_name_before(name: &str) -> String {
    lazy_static! {
        static ref RE_SAINT: Regex =
            Regex::new(r"(?i)(^|\W)s(?:ain)?t(e?)\W+").unwrap();
    }
    let res = RE_SAINT.replace_all(name, "${1}Saint${2}-");

    lazy_static! {
        static ref RE_AVENUE: Regex =
            Regex::new(r"(?i)(^|\W)ave?\.?(\W|$)").unwrap();
    }
    let res = RE_AVENUE.replace_all(&res, "${1}Avenue${2}");

    lazy_static! {
        static ref RE_PLACE: Regex =
            Regex::new(r"(?i)(^|\W)pl\.?(\W|$)").unwrap();
    }
    let res = RE_PLACE.replace_all(&res, "${1}Place${2}");

    lazy_static! {
        static ref RE_BOULEVARD: Regex =
            Regex::new(r"(?i)(^|\W)bl?v?d\.?(\W|$)").unwrap();
    }
    let res = RE_BOULEVARD.replace_all(&res, "${1}Boulevard${2}");

    lazy_static! {
        static ref RE_ROUTE: Regex =
            Regex::new(r"(?i)(^|\W)rte(\W|$)").unwrap();
    }
    let res = RE_ROUTE.replace_all(&res, "${1}Route${2}");

    lazy_static! {
        static ref RE_ND: Regex =
            Regex::new(r"(?i)(^|\W)n(?:otre)?[ -]*d(?:ame)?(\W|$)").unwrap();
    }
    let res = RE_ND.replace_all(&res, "${1}Notre-Dame${2}");

    lazy_static! {
        static ref RE_HOTEL_DE_VILLE: Regex =
            Regex::new(r"(?i)(^|\W)hdv(\W|$)").unwrap();
    }
    let res = RE_HOTEL_DE_VILLE.replace_all(&res, "${1}Hôtel de Ville${2}");

    lazy_static! {
        static ref RE_ROND_POINT: Regex =
            Regex::new(r"(?i)(^|\W)ro?n?d[ \.-]?po?i?n?t(\W|$)").unwrap();
    }
    let res = RE_ROND_POINT.replace_all(&res, "${1}Rond-Point ");

    res.into_owned()
}


pub fn sed_whole_name_after(name: &str) -> String {
    lazy_static! {
        static ref RE_DU_NUM: Regex =
            Regex::new(r"(?i)(^|\W)(du|de la) (\d+)e(\W|$)").unwrap();
    }
    let res = RE_DU_NUM.replace_all(name, "${1}${2} ${3}ème${4}");
    lazy_static! {
        static ref RE_DU_PREMIER: Regex =
            Regex::new(r"(?i)(^|\W)du 1ème(\W|$)").unwrap();
    }
    let res = RE_DU_PREMIER.replace_all(&res, "${1}du 1er${2}");
    lazy_static! {
        static ref RE_DE_LA_PREMIERE: Regex =
            Regex::new(r"(?i)(^|\W)de la 1ème(\W|$)").unwrap();
    }
    let res = RE_DE_LA_PREMIERE.replace_all(&res, "${1}de la 1ère${2}");

    lazy_static! {
        static ref RE_A: Regex =
            Regex::new(r"(?i) a ").unwrap();
    }
    let res = RE_A.replace_all(&res, " à ");

    lazy_static! {
        static ref RE_BACKQUOTE: Regex =
            Regex::new(r"(?i)’").unwrap();
    }
    let res = RE_BACKQUOTE.replace_all(&res, "'");

    lazy_static! {
        static ref RE_SPACES: Regex =
            Regex::new(r"(?i)[_ ]+").unwrap();
    }
    let res = RE_SPACES.replace_all(&res, " ");

    lazy_static! {
        static ref RE_SPACE_DASH: Regex =
            Regex::new(r"(?i)(^|[^ ])(?: -|- )([^ ]|$)").unwrap();
    }
    let res = RE_SPACE_DASH.replace_all(&res, "${1} - ${2}");

    lazy_static! {
        static ref RE_GENERAL: Regex =
            Regex::new(r"(?i)(^|\W)gal(\W|$)").unwrap();
    }
    let res = RE_GENERAL.replace_all(&res, "${1}Général${2}");

    lazy_static! {
        static ref RE_MARECHAL: Regex =
            Regex::new(r"(?i)(^|\W)mal(\W|$)").unwrap();
    }
    let res = RE_MARECHAL.replace_all(&res, "${1}Maréchal${2}");

    lazy_static! {
        static ref RE_QUOTE_H: Regex =
            Regex::new(r"(?i)(^|\W)([ld])[ ']+(h[aiîouyeéèê]|[aiîouyéèê]|et[^ ]|e[^t].)")
            .unwrap();
    }
    let res = RE_QUOTE_H.replace_all(&res, |caps: &Captures| {
        format!("{}{}'{}", &caps[1], &caps[2].to_lowercase(), &caps[3])
    });

    lazy_static! {
        static ref RE_QUOTE_DE: Regex =
            Regex::new(r"(?i)(^|\W)([ld])e[ ']+([aiîouyéèê]|et[^ ]|e[^t].)").unwrap();
    }
    let res = RE_QUOTE_DE.replace_all(&res, |caps: &Captures| {
        format!("{}{}'{}", &caps[1], &caps[2].to_lowercase(), &caps[3])
    });

    lazy_static! {
        static ref RE_DE_LE: Regex =
            Regex::new(r"(?i)(^|\W)de le(\W|$)").unwrap();
    }
    let res = RE_DE_LE.replace_all(&res, "${1}du${2}");
    lazy_static! {
        static ref RE_DE_LES: Regex =
            Regex::new(r"(?i)(^|\W)de les(\W|$)").unwrap();
    }
    let res = RE_DE_LES.replace_all(&res, "${1}des${2}");

    res.into_owned()
}


pub fn log_suspicious(name: &str) {
    lazy_static! {
        static ref RE_SUSPICIOUS: Regex =
            Regex::new(r"(?i)[^\w '-/\(\)\.]").unwrap();
    }
    for m in RE_SUSPICIOUS.find_iter(name) {
        println!("Warning: suspicious character {} in name {}",
                 m.as_str(),
                 name);
    }
    if name.contains(',') {
        println!("Warning: suspicious character , in name {}", name);
    }
}
