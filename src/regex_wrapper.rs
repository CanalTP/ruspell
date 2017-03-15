use regex::{Regex, RegexSet, Captures};
use utils;


fn must_be_lower(text: &str) -> bool {
    lazy_static! {
        static ref RE: RegexSet =
            RegexSet::new(&[
                r"(?i)^(en|sur|et|sous|de|du|des|le|la|les|au|aux|un|une)$",
                r"(?i)^(à|\d+([eè]me|[eè]re?|nde?))$",
                ]).unwrap();
    }
    RE.is_match(text)
}

fn must_be_upper(text: &str) -> bool {
    lazy_static! {
        static ref RE: RegexSet =
            RegexSet::new(&[
                r"(?i)^(RER|CDG|CES|ASPTT|PTT|EDF|GDF|INRIA|INRA|CRC|HEC|SNCF|RATP|HLM|CHR|CHU)$",
                r"(?i)^(KFC|MJC|IME|CAT|DDE|LEP|EGB|SNECMA|DGAT|VVF)$",
                r"(?i)^(ZA|ZAC|ZI|RPA|CFA|CEA|CC|IUT|TGV|CCI|UFR|CPAM|ANPE|\w*\d\w*|RN\d*|RD\d*)$",
                r"(?i)^(XL|X{0,3})(IX|IV|V?I{0,3})$",
                ]).unwrap();
    }
    RE.is_match(text)
}

pub fn fixed_case_word(name: String) -> String {
    let mut new_name = String::new();
    for word in utils::get_words(&name) {
        if must_be_lower(&word) {
            new_name.push_str(&word.to_lowercase());
        } else if must_be_upper(&word) {
            new_name.push_str(&word.to_uppercase());
        } else {
            new_name.push_str(word);
        }
    }
    new_name
}


pub fn sed_whole_name(name: String) -> String {
    lazy_static! {
        static ref RE_SAINT: Regex =
            Regex::new(r"(?i)(^|\W)s(?:ain)?t(e?)\W+").unwrap();
    }
    let res = RE_SAINT.replace_all(&name, "${1}Saint${2}-");

    lazy_static! {
        static ref RE_ND: Regex =
            Regex::new(r"(?i)(^|\W)n(?:otre)?[ -]*d(?:ame)?(\W|$)").unwrap();
    }
    let res = RE_ND.replace_all(&res, "${1}Notre-Dame${2}");

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
        static ref RE_AVENUE: Regex =
            Regex::new(r"(?i)(^|\W)ave?\.?(\W|$)").unwrap();
    }
    let res = RE_AVENUE.replace_all(&res, "${1}Avenue${2}");

    lazy_static! {
        static ref RE_ROUTE: Regex =
            Regex::new(r"(?i)(^|\W)rte(\W|$)").unwrap();
    }
    let res = RE_ROUTE.replace_all(&res, "${1}Route${2}");

    lazy_static! {
        static ref RE_DU_NUM: Regex =
            Regex::new(r"(?i)(^|\W)(du|de la) (\d+)e(\W|$)").unwrap();
    }
    let res = RE_DU_NUM.replace_all(&res, "${1}${2} ${3}ème${4}");
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
        static ref RE_HOTEL_DE_VILLE: Regex =
            Regex::new(r"(?i)(^|\W)hdv(\W|$)").unwrap();
    }
    let res = RE_HOTEL_DE_VILLE.replace_all(&res, "${1}Hôtel de Ville${2}");

    lazy_static! {
        static ref RE_A: Regex =
            Regex::new(r"(?i) a ").unwrap();
    }
    let res = RE_A.replace_all(&res, " à ");

    lazy_static! {
        static ref RE_ROND_POINT: Regex =
            Regex::new(r"(?i)(^|\W)ro?n?d[ \.-]?po?i?n?t(\W|$)").unwrap();
    }
    let res = RE_ROND_POINT.replace_all(&res, "${1}Rond-Point ");

    lazy_static! {
        static ref RE_SPACES: Regex =
            Regex::new(r"(?i)  +").unwrap();
    }
    let res = RE_SPACES.replace_all(&res, " ");

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
            Regex::new(r"(?i)(^|\W)([ld])[ '](h[aiouye]|[aiouy]|et[^ ]|e[^t].)").unwrap();
    }
    let res = RE_QUOTE_H.replace_all(&res, |caps: &Captures| {
        format!("{}{}'{}", &caps[1], &caps[2].to_lowercase(), &caps[3])
    });

    lazy_static! {
        static ref RE_QUOTE_DE: Regex =
            Regex::new(r"(?i)(^|\W)([ld])e[ ']([aiouye]|[aiouy]|et[^ ]|e[^t].)").unwrap();
    }
    let res = RE_QUOTE_DE.replace_all(&res, |caps: &Captures| {
        format!("{}{}'{}", &caps[1], &caps[2].to_lowercase(), &caps[3])
    });

    res.into_owned()
}
