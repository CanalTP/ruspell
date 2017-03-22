use csv;
use std::io;
use std::iter::FilterMap;
use errors::{Result, ResultExt};


#[derive(Debug)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub raw: Vec<String>,
}

pub struct RecordIter<'a, R: io::Read + 'a> {
    iter: csv::StringRecords<'a, R>,
    id_pos: usize,
    name_pos: usize,
}
impl<'a, R: io::Read + 'a> RecordIter<'a, R> {
    fn new(r: &'a mut csv::Reader<R>, heading_id: &str, heading_name: &str) -> csv::Result<Self> {
        let headers = r.headers()?;

        let get_optional_pos = |name| headers.iter().position(|s| s == name);
        let get_pos = |field| {
            get_optional_pos(field).ok_or_else(|| {
                csv::Error::Decode(format!("Invalid file, cannot find column '{}'", field))
            })
        };

        Ok(RecordIter {
            iter: r.records(),
            id_pos: get_pos(heading_id)?,
            name_pos: get_pos(heading_name)?,
        })
    }
}
impl<'a, R: io::Read + 'a> Iterator for RecordIter<'a, R> {
    type Item = csv::Result<Record>;
    fn next(&mut self) -> Option<Self::Item> {
        fn get(record: &[String], pos: usize) -> csv::Result<&str> {
            match record.get(pos) {
                Some(s) => Ok(s),
                None => Err(csv::Error::Decode(format!("Failed accessing record '{}'.", pos))),
            }
        }

        self.iter.next().map(|r| {
            r.and_then(|r| {
                let id = get(&r, self.id_pos)?.to_string();
                let name = get(&r, self.name_pos)?.to_string();
                Ok(Record {
                    id: id,
                    name: name,
                    raw: r,
                })
            })
        })
    }
}


type CompleteRecordIterator<'a, R> = FilterMap<RecordIter<'a, R>,
                                               fn(csv::Result<Record>) -> Option<Record>>;

pub fn new_record_iter<'a, R: io::Read + 'a>
    (r: &'a mut csv::Reader<R>,
     heading_id: &str,
     heading_name: &str)
     -> Result<(CompleteRecordIterator<'a, R>, Vec<String>, usize)> {
    fn reader_handler(rc: csv::Result<Record>) -> Option<Record> {
        rc.map_err(|e| println!("error at csv line decoding : {}", e)).ok()
    }
    let headers = r.headers().chain_err(|| "Can't find headers in input file")?;
    let rec_iter = RecordIter::new(r, heading_id, heading_name)
        .chain_err(|| "Can't find needed fields in the header of input file")?;
    let pos = rec_iter.name_pos;

    Ok((rec_iter.filter_map(reader_handler), headers, pos))
}
