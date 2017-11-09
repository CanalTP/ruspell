use std::collections::BTreeMap;
use std::io;
use csv;
use utils;
use super::ispell_wrapper::SpellCheck;
use errors::{Result, ResultExt};
use std::path::Path;

pub fn populate_dict_from_files(
    files: &[String],
    ispell: &mut SpellCheck,
    conf_path: &Path,
) -> Result<()> {
    // This map is built as follows :
    // map_normed["napoleon"] = map_napo
    // map_napo["Napoléon"] = 42 (occurences)
    // map_napo["Napoleon"] = 2 (occurences)
    let mut map_normed = BTreeMap::new();
    for f in files {
        let mut file_path = conf_path.join(f);
        file_path = file_path.canonicalize().chain_err(|| {
            format!("Could not read {}", file_path.display())
        })?;
        println!("Reading street and city names from {}", file_path.display());

        let mut rdr = csv::Reader::from_file(&file_path)
            .chain_err(|| {
                format!("Could not open BANO file {}", file_path.display())
            })?
            .double_quote(true)
            .has_headers(false);
        let banos = new_bano_iter(&mut rdr);

        for res_b in banos {
            let b = res_b.chain_err(|| {
                format!("error at line csv decoding: {}", file_path.display())
            })?;
            for w in b.street.split_whitespace().chain(b.city.split_whitespace()) {
                // do not consider full-uppercase word or word containing a digit
                if w.chars().all(|c| !c.is_lowercase()) || w.chars().any(|c| c.is_numeric()) {
                    continue;
                }
                let map = map_normed.entry(utils::normed(w)).or_insert_with(
                    BTreeMap::new,
                );
                *map.entry(w.to_string()).or_insert(0) += 1;
            }
        }
    }

    let corpus_size: u32 = map_normed.values().flat_map(|m| m.values()).sum();
    println!("BANO corpus size = {}", corpus_size);
    let mut nb_added = 0;
    for map in map_normed.values() {
        if let Some(interesting_word) = get_interesting_word(map) {
            if (map[&interesting_word] >= corpus_size / 100000 ||
                    !ispell.has_competitor_word(&interesting_word)?) &&
                !ispell.has_same_accent_word(&interesting_word)?
            {
                let _ = ispell.add_word(&interesting_word); // ignore the error
                nb_added += 1;
            }
        }
    }
    println!("Added {} words to dictionnary", nb_added);
    Ok(())
}

fn get_interesting_word(map: &BTreeMap<String, u32>) -> Option<String> {
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
                None => Err(csv::Error::Decode(
                    format!("Failed accessing record '{}'.", pos),
                )),
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

fn new_bano_iter<'a, R: io::Read>(r: &'a mut csv::Reader<R>) -> BanoIter<'a, R> {
    BanoIter::new(r).expect("Can't find needed fields in the header.")
}
