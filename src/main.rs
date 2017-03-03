extern crate csv;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i", help = "CSV file to be processed \
                                                     (typically a GTFS stops.txt file)")]
    input: String,

    #[structopt(long = "output", short = "o", default_value = "rules.csv",
                help = "Fusio rules.csv file")]
    output: String,

    #[structopt(long = "id", short = "d", default_value = "stop_id",
                help = "The heading name of the column that is the unique id of the record")]
    heading_id: String,

    #[structopt(long = "name", short = "s", default_value = "stop_name",
                help = "The heading name of the column that needs a spell_check.")]
    heading_name: String,
}


#[derive(Debug)]
struct Record {
    id: String,
    name: String,
}

struct RecordIter<'a, R: std::io::Read + 'a> {
    iter: csv::StringRecords<'a, R>,
    id_pos: usize,
    name_pos: usize,
}
impl<'a, R: std::io::Read + 'a> RecordIter<'a, R> {
    fn new(r: &'a mut csv::Reader<R>, heading_id: &str, heading_name: &str) -> csv::Result<Self> {
        let headers = try!(r.headers());

        let get_optional_pos = |name| headers.iter().position(|s| s == name);
        let get_pos = |field| {
            get_optional_pos(field).ok_or_else(|| {
                csv::Error::Decode(format!("Invalid file, cannot find column '{}'", field))
            })
        };

        Ok(RecordIter {
            iter: r.records(),
            id_pos: try!(get_pos(heading_id)),
            name_pos: try!(get_pos(heading_name)),
        })
    }
}
impl<'a, R: std::io::Read + 'a> Iterator for RecordIter<'a, R> {
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
                let id = try!(get(&r, self.id_pos));
                let name = try!(get(&r, self.name_pos));
                Ok(Record {
                    id: id.to_string(),
                    name: name.to_string(),
                })
            })
        })
    }
}

fn main() {
    let args = Args::from_args();

    let mut rdr = csv::Reader::from_file(args.input)
        .unwrap()
        .double_quote(true);

    let record_list: Vec<Record> = RecordIter::new(&mut rdr, &args.heading_id, &args.heading_name)
        .expect("Can't find needed fields in the header.")
        .filter_map(|rc| {
            rc.map_err(|e| println!("error at csv line decoding : {}", e))
                .ok()
        })
        .collect();

    let mut wtr = csv::Writer::from_file(args.output).unwrap();
    wtr.encode(("id", "old_name", "new_name"))
        .unwrap();

    for rec in &record_list {
        wtr.encode((&rec.id, &rec.name, &rec.name))
            .unwrap();
    }
}
