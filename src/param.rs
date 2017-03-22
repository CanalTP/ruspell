use std::fs::File;
use workers::worker;
use workers::regex_processor as rp;
use errors::{Result, ResultExt};
use serde_yaml;

use workers::ispell_wrapper;
use workers::bano_reader;

// define params file structure
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ProcessSequence {
    processes: Vec<NameProcessor>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum NameProcessor {
    Decode(Decode),
    FirstLetterUppercase,
    SnakeCase,
    LowercaseWord(FixedcaseWord),
    UppercaseWord(FixedcaseWord),
    RegexReplace(RegexReplace),
    IspellCheck(IspellCheck),
    LogSuspicious(LogSuspicious),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Decode {
    pub from_encoding: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FixedcaseWord {
    words: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct RegexReplace {
    from: String,
    to: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct IspellCheck {
    bano_files: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct LogSuspicious {
    regex: String,
}

pub fn read_param(param_file: &str) -> Result<Vec<worker::Processor>> {
    use self::NameProcessor::*;
    use workers::worker::Processor as WP;

    let param_rdr = File::open(param_file).chain_err(|| "Could not open param file")?;

    let sequence: ProcessSequence =
        serde_yaml::from_reader(param_rdr).chain_err(|| "Problem while reading param file")?;

    sequence.processes
        .into_iter()
        .map(|a| match a {
            LowercaseWord(lcw) => rp::FixedcaseProcessor::new(&lcw.words, true).map(WP::Fixedcase),
            UppercaseWord(ucw) => rp::FixedcaseProcessor::new(&ucw.words, false).map(WP::Fixedcase),
            IspellCheck(i) => {
                let mut ispell =
                ispell_wrapper::SpellCheck::new().chain_err(|| "Could not create ispell manager")?;
                bano_reader::populate_dict_from_files(&i.bano_files, &mut ispell)?;
                Ok(WP::Ispell(ispell))
            }
            RegexReplace(re) => rp::RegexReplace::new(&re.from, &re.to).map(WP::RegexReplace),
            LogSuspicious(l) => rp::LogSuspicious::new(&l.regex).map(WP::LogSuspicious),
            Decode(d) => Ok(WP::Decode(d)),
            SnakeCase => Ok(WP::SnakeCase),
            FirstLetterUppercase => Ok(WP::FirstLetterUppercase),
        })
        .collect()
}
