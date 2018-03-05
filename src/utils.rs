use encoding::label::encoding_from_whatwg_label;
use encoding::EncoderTrap;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
use errors::Result;

pub fn decode(name: &str, encoding: &str) -> Result<String> {
    let enc = encoding_from_whatwg_label(encoding);
    if enc.is_none() {
        return Err(format!("Could not find encoding from {}", encoding).into());
    }
    if let Ok(Ok(res)) = enc.unwrap()
        .encode(name, EncoderTrap::Strict)
        .map(String::from_utf8)
    {
        return Ok(res);
    }
    Ok(name.to_string())
}

// split into words (based on non-alphanumeric chars)
pub fn get_words(name: &str) -> Vec<&str> {
    let mut words = Vec::<&str>::new();
    let mut index_start_word = 0;
    let mut is_current_alpha = name.chars()
        .next()
        .map(char::is_alphanumeric)
        .unwrap_or(true);
    for c in name.char_indices() {
        if c.1.is_alphanumeric() != is_current_alpha {
            words.push(&name[index_start_word..c.0]);
            is_current_alpha = c.1.is_alphanumeric();
            index_start_word = c.0;
        }
    }
    words.push(&name[index_start_word..]);
    words
}

// Force the first char uppercase
pub fn first_upper(name: &str) -> String {
    let mut chars = name.chars();
    let mut new_name = String::new();
    new_name.extend(chars.next().map(|c| c.to_uppercase().collect::<String>()));
    new_name.extend(chars);
    new_name
}

/// MUSEE dE La GARE sncf > Musee de la gare de lyon
pub fn first_upper_all_lower(name: &str) -> String {
    let mut chars = name.chars();
    let mut new_name = String::new();
    new_name.extend(chars.next().map(|c| c.to_uppercase().collect::<String>()));
    new_name.extend(chars.flat_map(char::to_lowercase));
    new_name
}

/// every word becomes Mmmmmm
pub fn snake_case(name: &str) -> String {
    let mut new_name = String::new();
    for word in get_words(name) {
        new_name.push_str(&first_upper_all_lower(word));
    }
    new_name
}

// normalize a word (remove accents, lowercase, ...)
pub fn normed(word: &str) -> String {
    word.nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn has_accent(word: &str) -> bool {
    word.nfkd().any(is_combining_mark)
}
