extern crate rustc_serialize;
extern crate csv;
extern crate structopt;
extern crate encoding;
extern crate regex;
extern crate ispell;
extern crate unicode_normalization;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate structopt_derive;

mod utils;
mod worker;
mod records_reader;
mod errors;
mod conf;


use structopt::StructOpt;
use records_reader::Record;
use std::io;
use errors::{Result, ResultExt};


#[derive(StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i",
                help = "Path to input CSV file to be processed \
                        (typically a GTFS stops.txt file).")]
    input: String,

    #[structopt(long = "config", short = "c",
                help = "Path to configuration file to be read.")]
    config: String,

    #[structopt(long = "output", short = "o",
                help = "Path to output CSV file after processing \
                        (same as input, <name> column processed).")]
    output: Option<String>,

    #[structopt(long = "rules", short = "r", default_value = "rules.csv",
                help = "Path to output rules.csv file \
                        (modifications description).")]
    rules: String,

    #[structopt(long = "id", short = "I", default_value = "stop_id",
                help = "The heading name of the column that is the unique id of the record.")]
    heading_id: String,

    #[structopt(long = "name", short = "N", default_value = "stop_name",
                help = "The heading name of the column that needs a spell_check.")]
    heading_name: String,
}


/// management of all processing applied to names
/// returns None if no change was applied,
/// Some modified name otherwise
fn process_record(rec: &Record,
                  processors: &mut [worker::Processor])
                  -> Result<Option<RecordRule>> {

    let mut new_name = rec.name.clone();
    let mut modifications = vec![];
    for (i, p) in processors.iter_mut().enumerate() {
        let modified_name = p.apply(&new_name)?;
        if modified_name != new_name {
            modifications.push((i, modified_name.clone()));
        }
        new_name = modified_name;
    }

    if rec.name == new_name && modifications.is_empty() {
        Ok(None)
    } else {
        Ok(Some(RecordRule {
            id: rec.id.clone(),
            old_name: rec.name.clone(),
            new_name: new_name,
            debug: format!("{:?}", modifications),
        }))
    }
}


#[derive(Debug, RustcEncodable)]
struct RecordRule {
    id: String,
    old_name: String,
    new_name: String,
    debug: String,
}


fn run() -> Result<()> {
    let args = Args::from_args();

    let mut rdr_stops = csv::Reader::from_file(&args.input)
        .chain_err(|| "Could not open input file")?
        .double_quote(true);
    let (records, headers, name_pos) =
        records_reader::new_record_iter(&mut rdr_stops, &args.heading_id, &args.heading_name)?;

    // producing rules to be applied to re-spell names
    let mut wtr_rules =
        csv::Writer::from_file(&args.rules).chain_err(|| "Could not open rules file")?;
    wtr_rules.encode(("id", "old_name", "new_name", "debug"))
        .chain_err(|| "Could not write header of rules file")?;

    // producing output and replacing names only if requested (wtr_stops is an Option)
    let mut wtr_stops = match args.output {
        Some(ref f) => Some(csv::Writer::from_file(f).chain_err(|| "Could not open output file")?),
        None => None,
    };
    wtr_stops.as_mut()
        .map_or(Ok(()), |w| w.encode(headers))
        .chain_err(|| "Could not write header of output file")?;

    //creating processor vector from config
    let mut processors = conf::read_conf(&args.config).chain_err(|| "Could not read config file")?;

    for res_rec in records {
        let mut rec = res_rec.chain_err(|| format!("error at csv line decoding: {}", &args.input))?;
        if let Some(rule) = process_record(&rec, &mut processors)? {
            rec.raw[name_pos] = rule.new_name.clone();
            wtr_rules.encode(&rule).chain_err(|| "Could not write into rules file")?;
        }
        wtr_stops.as_mut()
            .map_or(Ok(()), |w| w.encode(&rec.raw))
            .chain_err(|| "Could not write into output file")?;
    }
    Ok(())
}


fn main() {
    if let Err(ref e) = run() {
        use io::Write;
        let stderr = &mut io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}
