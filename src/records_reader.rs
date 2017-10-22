use csv;
use std::io;
use errors::{Result, ResultExt};
use std::collections::HashMap;
#[derive(Debug)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub raw: HashMap<String, String>,
}

pub struct RecordIter<'r, R: io::Read + 'r> {
    iter: csv::StringRecordsIter<'r, R>,
    heading_id: String,
    heading_name: String,
    headers: csv::StringRecord,
}

impl<'r, R: io::Read + 'r> RecordIter<'r, R> {
    fn new(r: &'r mut csv::Reader<R>, heading_id: &str, heading_name: &str) -> csv::Result<Self> {
        let headers = r.headers()?.clone();

        Ok(RecordIter {
            iter: r.records(),
            heading_id: heading_id.to_string(),
            heading_name: heading_name.to_string(),
            headers,
        })
    }
}

impl<'r, R: io::Read + 'r> Iterator for RecordIter<'r, R> {
    type Item = csv::Result<Record>;
    fn next(&mut self) -> Option<Self::Item> {
        fn get<'a>(record: &'a HashMap<String, String>, column: &str) -> String {
            record.get(column).unwrap().to_string()
        }

        self.iter.next().map(|r| {
            r.and_then(|r| {
                let rec: HashMap<String, String> = r.deserialize(Some(&self.headers))?;

                let id = get(&rec, &self.heading_id);
                let name = get(&rec, &self.heading_name);
                Ok(Record { id, name, raw: rec })
            })
        })
    }
}

pub fn new_record_iter<'r, R: io::Read + 'r>(
    r: &'r mut csv::Reader<R>,
    heading_id: &str,
    heading_name: &str,
) -> Result<(RecordIter<'r, R>, csv::StringRecord)> {
    let headers = r.headers()
        .chain_err(|| "Can't find headers in input file")?
        .clone();
    let rec_iter = RecordIter::new(r, heading_id, heading_name)
        .chain_err(|| "Can't find needed fields in the header of input file")?;

    Ok((rec_iter, headers))
}
