extern crate csv;
extern crate encoding;
#[macro_use]
extern crate error_chain;
extern crate ispell;
extern crate regex;
extern crate rustc_serialize;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate unicode_normalization;

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

    #[structopt(long = "config", short = "c", help = "Path to configuration file to be read.")]
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
fn process_record(
    rec: &Record,
    processors: &mut [worker::Processor],
) -> Result<Option<RecordRule>> {
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
            new_name,
            debug: format!("{:?}", modifications),
        }))
    }
}

#[derive(Debug, Serialize)]
struct RecordRule {
    id: String,
    old_name: String,
    new_name: String,
    debug: String,
}

fn run() -> Result<()> {
    let args = Args::from_args();

    let mut rdr_stops = csv::ReaderBuilder::new()
        .from_path(&args.input)
        .chain_err(|| "Could not open input file")?;
    let (records, headers) =
        records_reader::new_record_iter(&mut rdr_stops, &args.heading_id, &args.heading_name)?;

    // producing rules to be applied to re-spell names
    let mut wtr_rules =
        csv::Writer::from_path(&args.rules).chain_err(|| "Could not open rules file")?;
    wtr_rules
        .serialize(&["id", "old_name", "new_name", "debug"])
        .chain_err(|| "Could not write header of rules file")?;

    // producing output and replacing names only if requested (wtr_stops is an Option)
    let mut wtr_stops = match args.output {
        Some(ref f) => Some(csv::Writer::from_path(f).chain_err(|| "Could not open output file")?),
        None => None,
    };
    wtr_stops
        .as_mut()
        .map_or(Ok(()), |w| w.write_record(&headers))
        .chain_err(|| "Could not write header of output file")?;

    //creating processor vector from config
    let mut processors = conf::read_conf(&args.config).chain_err(|| "Could not read config file")?;

    for res_rec in records {
        let mut rec = res_rec.chain_err(|| format!("error at csv line decoding: {}", &args.input))?;
        if let Some(rule) = process_record(&rec, &mut processors)? {
            *rec.raw.get_mut(&args.heading_name).unwrap() = rule.new_name.clone();

            wtr_rules
                .serialize(&rule)
                .chain_err(|| "Could not write into rules file")?;
        }

        let mut stop_record: Vec<&str> = Vec::with_capacity(headers.len());
        for h in &headers {
            stop_record.push(&rec.raw[h]);
        }

        wtr_stops
            .as_mut()
            .map_or(Ok(()), |w| w.serialize(&stop_record))
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
