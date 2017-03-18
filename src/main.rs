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
#[macro_use]
extern crate lazy_static;

mod utils;
mod regex_processor;
mod ispell_wrapper;
mod bano_reader;
mod records_reader;
mod errors;

use structopt::StructOpt;
use ispell_wrapper::SpellCheck;
use records_reader::Record;
use regex_processor::RegexProcessor;
use std::io;
use errors::{Result, ResultExt};


#[derive(StructOpt)]
struct Args {
    #[structopt(long = "input", short = "i",
                help = "Path to input CSV file to be processed \
                        (typically a GTFS stops.txt file).")]
    input: String,

    #[structopt(long = "bano", short = "b",
                help = "Path to input BANO file to be read \
                        (street and city names for dictionnary).")]
    bano: Option<String>,

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ProcessSequence {
    actions: Vec<Action>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Action {
    FixedCaseWord(FixedCaseWord),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FixedCaseWord {
    words: Vec<String>,
}

#[derive(Debug, RustcEncodable)]
struct RecordRule {
    id: String,
    old_name: String,
    new_name: String,
}


/// management of all processing applied to names
fn process_record(rec: &Record,
                  ispell: &mut SpellCheck,
                  regex: &RegexProcessor)
                  -> Result<Option<RecordRule>> {

    let mut new_name = utils::decode(&rec.name.clone());
    new_name = regex_processor::sed_whole_name_before(&new_name);
    new_name = ispell.check(&new_name)?;
    new_name = utils::snake_case(&new_name);
    new_name = regex_processor::fixed_case_word(&new_name, regex);
    new_name = regex_processor::sed_whole_name_after(&new_name);
    new_name = utils::first_upper(&new_name);

    regex_processor::log_suspicious(&new_name);

    if rec.name == new_name {
        Ok(None)
    } else {
        Ok(Some(RecordRule {
                    id: rec.id.clone(),
                    old_name: rec.name.clone(),
                    new_name: new_name,
                }))
    }
}


fn run() -> Result<()> {
    use bano_reader;

    let args = Args::from_args();

    let mut rdr_stops = csv::Reader::from_file(args.input)
        .chain_err(|| "Could not open input file")?
        .double_quote(true);
    let (records, headers, name_pos) =
        records_reader::new_record_iter(&mut rdr_stops, &args.heading_id, &args.heading_name)?;

    // producing rules to be applied to re-spell names
    let mut wtr_rules =
        csv::Writer::from_file(args.rules).chain_err(|| "Could not open rules file")?;
    wtr_rules.encode(("id", "old_name", "new_name"))
        .chain_err(|| "Could not write header of rules file")?;

    // producing output and replacing names only if requested (wtr_stops is an Option)
    let mut wtr_stops = match args.output {
        Some(ref f) => Some(csv::Writer::from_file(f).chain_err(|| "Could not open output file")?),
        None => None,
    };
    wtr_stops.as_mut()
        .map_or(Ok(()), |w| w.encode(headers))
        .chain_err(|| "Could not write header of output file")?;

    //creating regex wrapper from params
    let mut regex = RegexProcessor::new();
    let fcw = FixedCaseWord {
        words: vec!["en".to_string(),
                    "sur".to_string(),
                    "et".to_string(),
                    "sous".to_string(),
                    "de".to_string(),
                    "du".to_string(),
                    "des".to_string(),
                    "la".to_string(),
                    "les".to_string(),
                    "lès".to_string(),
                    "au".to_string(),
                    "aux".to_string(),
                    "un".to_string(),
                    "une".to_string(),
                    "à".to_string(),
                    "le".to_string(),
                    "RER".to_string(),
                    "CDG".to_string(),
                    "CES".to_string(),
                    "ASPTT".to_string(),
                    "PTT".to_string(),
                    "EDF".to_string(),
                    "GDF".to_string(),
                    "INRIA".to_string(),
                    "INRA".to_string(),
                    "CRC".to_string(),
                    "HEC".to_string(),
                    "SNCF".to_string(),
                    "RATP".to_string(),
                    "HLM".to_string(),
                    "CHR".to_string(),
                    "CHU".to_string(),
                    "KFC".to_string(),
                    "MJC".to_string(),
                    "IME".to_string(),
                    "CAT".to_string(),
                    "DDE".to_string(),
                    "LEP".to_string(),
                    "EGB".to_string(),
                    "SNECMA".to_string(),
                    "DGAT".to_string(),
                    "VVF".to_string(),
                    "ZA".to_string(),
                    "ZAC".to_string(),
                    "ZI".to_string(),
                    "RPA".to_string(),
                    "CFA".to_string(),
                    "CEA".to_string(),
                    "CC".to_string(),
                    "IUT".to_string(),
                    "TGV".to_string(),
                    "CCI".to_string(),
                    "UFR".to_string(),
                    "CPAM".to_string(),
                    "ANPE".to_string()],
    };
    let process = ProcessSequence { actions: vec![Action::FixedCaseWord(fcw)] };
    let s = serde_yaml::to_string(&process).unwrap();
    println!("{}", s);
    for a in process.actions {
        match a {
            Action::FixedCaseWord(fcw) => {
                for w in fcw.words {
                    regex.add_fixed_case(&w)?;
                }
            }
        }
    }

    // creating ispell manager (and populate dictionnary if requested)
    let mut ispell = SpellCheck::new().chain_err(|| "Could not create ispell manager")?;
    if let Some(bano_file) = args.bano {
        bano_reader::populate_dict_from_file(&bano_file, &mut ispell)?;
    }

    for mut rec in records {
        if let Some(rule) = process_record(&rec, &mut ispell, &regex)? {
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
