use std::fs::File;
use worker::{self, ispell_wrapper, bano_reader, regex_processor as rp};
use errors::{Result, ResultExt};
use serde_yaml;
use std::path::Path;

// define config file structure
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
    dictionnary: String,
    bano_files: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct LogSuspicious {
    regex: String,
}

pub fn read_conf(conf_file: &str) -> Result<Vec<worker::Processor>> {
    use self::NameProcessor::*;
    use worker::Processor as WP;

    let conf_rdr = File::open(conf_file).chain_err(
        || "Could not open config file",
    )?;

    let sequence: ProcessSequence = serde_yaml::from_reader(conf_rdr).chain_err(
        || "Problem while reading config file",
    )?;

    sequence
        .processes
        .into_iter()
        .map(|a| match a {
            LowercaseWord(lcw) => {
                rp::FixedcaseProcessor::new(&lcw.words, rp::CaseSpecifier::Lower)
                    .chain_err(|| "Could not create LowercaseWord manager")
                    .map(WP::Fixedcase)
            }
            UppercaseWord(ucw) => {
                rp::FixedcaseProcessor::new(&ucw.words, rp::CaseSpecifier::Upper)
                    .chain_err(|| "Could not create UppercaseWord manager")
                    .map(WP::Fixedcase)
            }
            IspellCheck(i) => {
                // the conf_file is already valid, thus this can't fail
                let conf_path = Path::new(conf_file).parent().unwrap();
                let mut ispell = ispell_wrapper::SpellCheck::new(&i.dictionnary).chain_err(
                    || "Could not create ispell manager",
                )?;
                bano_reader::populate_dict_from_files(&i.bano_files, &mut ispell, conf_path)?;
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
