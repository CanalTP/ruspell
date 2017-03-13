use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
use std::collections::HashMap;

pub fn populate_dict_from_bano_file(file: &str, ispell: &mut ::ispell_wrapper::SpellCheck) {
    println!("Reading street and city names from {}", file);

    let mut rdr = csv::Reader::from_file(file).unwrap().double_quote(true);
    let banos = new_bano_iter(&mut rdr);
    let mut map_normed = HashMap::new();

    for b in banos {
        for w in b.street.split_whitespace().chain(b.city.split_whitespace()) {
            if w.chars().all(|c| !c.is_lowercase()) || w.chars().any(|c| c.is_numeric()) {
                continue;
            }
            let normed_w: String = w.nfkd()
                .filter(|c| !is_combining_mark(*c))
                .flat_map(char::to_lowercase)
                .collect();
            let map = map_normed.entry(normed_w).or_insert(HashMap::new());
            let counter = map.entry(w.to_string()).or_insert(0);
            *counter += 1;
        }
    }

    let mut nb_added = 0;
    for (normed_w, map) in map_normed {
        if let Some(interesting_word) = get_interesting_word(map) {
            if interesting_word.len() > normed_w.len() {
                ispell.add_word(&interesting_word).unwrap();
                nb_added += 1;
            }
        }
    }
    println!("Added {} words to dictionnary", nb_added);
}

fn get_interesting_word(map: HashMap<String, u32>) -> Option<String> {
    let mut map_iter = map.iter();
    let mut first_max_w = map_iter.next().unwrap();
    if map.len() == 1 {
        return Some(first_max_w.0.clone());
    }
    let mut second_max_w = map_iter.next().unwrap();
    if second_max_w.1 > first_max_w.1 {
        std::mem::swap(&mut second_max_w, &mut first_max_w);
    }
    for i in map_iter {
        if i.1 > first_max_w.1 {
            std::mem::swap(&mut second_max_w, &mut first_max_w);
            first_max_w = i;
        } else if i.1 > second_max_w.1 {
            second_max_w = i;
        }
    }
    if (first_max_w.1 / second_max_w.1) >= 4 {
        return Some(first_max_w.0.clone());
    }
    None
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct Bano {
    pub street: String,
    pub city: String,
}

use ::*;
struct BanoIter<'a, R: std::io::Read + 'a> {
    iter: csv::StringRecords<'a, R>,
    street_pos: usize,
    city_pos: usize,
}
impl<'a, R: std::io::Read + 'a> BanoIter<'a, R> {
    fn new(r: &'a mut csv::Reader<R>) -> csv::Result<Self> {
        Ok(BanoIter {
               iter: r.records(),
               street_pos: 2,
               city_pos: 4,
           })
    }
}
impl<'a, R: std::io::Read + 'a> Iterator for BanoIter<'a, R> {
    type Item = csv::Result<Bano>;
    fn next(&mut self) -> Option<Self::Item> {
        fn get(record: &[String], pos: usize) -> csv::Result<&str> {
            match record.get(pos) {
                Some(s) => Ok(s),
                None => Err(csv::Error::Decode(format!("Failed accessing record '{}'.", pos))),
            }
        }

        self.iter.next().map(|r| {
            r.and_then(|r| {
                let street = try!(get(&r, self.street_pos)).to_string();
                let city = try!(get(&r, self.city_pos)).to_string();
                Ok(Bano {
                       street: street,
                       city: city,
                   })
            })
        })
    }
}

fn new_bano_iter<'a, R: std::io::Read + 'a>
    (r: &'a mut csv::Reader<R>)
     -> std::iter::FilterMap<BanoIter<'a, R>, fn(csv::Result<Bano>) -> Option<Bano>> {
    fn reader_handler(rc: csv::Result<Bano>) -> Option<Bano> {
        rc.map_err(|e| println!("error at csv line decoding : {}", e)).ok()
    }
    let rec_iter = BanoIter::new(r).expect("Can't find needed fields in the header.");

    rec_iter.filter_map(reader_handler)
}

