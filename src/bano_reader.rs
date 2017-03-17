use std::collections::HashMap;
use std::iter::FilterMap;
use std::io;
use csv;
use utils;
use errors::{Result, ResultExt};

pub fn populate_dict_from_file(file: &str,
                               ispell: &mut ::ispell_wrapper::SpellCheck)
                               -> Result<()> {
    println!("Reading street and city names from {}", file);

    let mut rdr =
        csv::Reader::from_file(file).chain_err(|| "Could not open BANO file")?.double_quote(true);
    let banos = new_bano_iter(&mut rdr);

    // This map is built as follows :
    // map_normed["napoleon"] = map_napo
    // map_napo["Napoléon"] = 42 (occurences)
    // map_napo["Napoleon"] = 2 (occurences)
    let mut map_normed = HashMap::new();

    for b in banos {
        for w in b.street.split_whitespace().chain(b.city.split_whitespace()) {
            // do not consider full-uppercase word or word containing a digit
            if w.chars().all(|c| !c.is_lowercase()) || w.chars().any(|c| c.is_numeric()) {
                continue;
            }
            let map = map_normed.entry(utils::normed(w)).or_insert_with(HashMap::new);
            *map.entry(w.to_string()).or_insert(0) += 1;
        }
    }

    let mut nb_added = 0;
    for map in map_normed.values() {
        if let Some(interesting_word) = get_interesting_word(map) {
            if utils::has_accent(&interesting_word) {
                let _ = ispell.add_word(&interesting_word); // ignore the error
                nb_added += 1;
            }
        }
    }
    println!("Added {} words to dictionnary", nb_added);
    Ok(())
}

fn get_interesting_word(map: &HashMap<String, u32>) -> Option<String> {
    let mut map_iter = map.iter();
    let mut first_max_w = map_iter.next().expect("This map should never be empty");
    let mut second_max_count = 0;

    for i in map_iter {
        if i.1 > first_max_w.1 {
            second_max_count = *first_max_w.1;
            first_max_w = i;
        } else if *i.1 > second_max_count {
            second_max_count = *i.1;
        }
    }

    // first max contains the forms appearing more and its occurences
    // second max reports the occurences of the second form appearing more
    // if the first form appears 4 times (empirical) more than the second one, it is qualified
    if second_max_count == 0 || (*first_max_w.1 / second_max_count) >= 4 {
        return Some(first_max_w.0.clone());
    }
    None
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct Bano {
    pub street: String,
    pub city: String,
}

struct BanoIter<'a, R: io::Read + 'a> {
    iter: csv::StringRecords<'a, R>,
    street_pos: usize,
    city_pos: usize,
}
impl<'a, R: io::Read + 'a> BanoIter<'a, R> {
    fn new(r: &'a mut csv::Reader<R>) -> csv::Result<Self> {
        Ok(BanoIter {
               iter: r.records(),
               street_pos: 2,
               city_pos: 4,
           })
    }
}
impl<'a, R: io::Read + 'a> Iterator for BanoIter<'a, R> {
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
                let street = get(&r, self.street_pos)?.to_string();
                let city = get(&r, self.city_pos)?.to_string();
                Ok(Bano {
                       street: street,
                       city: city,
                   })
            })
        })
    }
}

type CompleteBanoIterator<'a, R> = FilterMap<BanoIter<'a, R>,
                                             fn(csv::Result<Bano>) -> Option<Bano>>;

fn new_bano_iter<R: io::Read>(r: &mut csv::Reader<R>) -> CompleteBanoIterator<R> {
    fn reader_handler(rc: csv::Result<Bano>) -> Option<Bano> {
        rc.map_err(|e| println!("error at csv line decoding : {}", e)).ok()
    }
    let rec_iter = BanoIter::new(r).expect("Can't find needed fields in the header.");

    rec_iter.filter_map(reader_handler)
}
