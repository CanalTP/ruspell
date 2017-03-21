use std::fs::File;
use worker;
use regex_processor;
use errors::{Result, ResultExt};
use serde_yaml;

use ispell_wrapper;
use bano_reader;

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
    let mut processors = vec![];
    let param_rdr = File::open(param_file).chain_err(|| "Could not open param file")?;

    let sequence: ProcessSequence =
        serde_yaml::from_reader(param_rdr).chain_err(|| "Problem while reading param file")?;
    for a in sequence.processes {
        match a {
            NameProcessor::LowercaseWord(lcw) => {
                processors.push(
                    worker::Processor::Fixedcase(
                        regex_processor::FixedcaseProcessor::new(&lcw.words, true)?
                    )
                );
            }
            NameProcessor::UppercaseWord(ucw) => {
                processors.push(
                    worker::Processor::Fixedcase(
                        regex_processor::FixedcaseProcessor::new(&ucw.words, false)?
                    )
                );
            }
            NameProcessor::IspellCheck(i) => {
                let mut ispell = ispell_wrapper::SpellCheck::new()
                .chain_err(|| "Could not create ispell manager")?;
                bano_reader::populate_dict_from_files(&i.bano_files, &mut ispell)?;
                processors.push(worker::Processor::Ispell(ispell));
            }
            NameProcessor::RegexReplace(re) => {
                processors.push(
                    worker::Processor::RegexReplace(
                        regex_processor::RegexReplace::new(&re.from, &re.to)?
                    )
                )
            }
            NameProcessor::LogSuspicious(l) => {
                processors.push(
                    worker::Processor::LogSuspicious(
                        regex_processor::LogSuspicious::new(&l.regex)?
                    )
                );
            }
            NameProcessor::Decode(d) => {
                processors.push(worker::Processor::Decode(d));
            }
            NameProcessor::SnakeCase => {
                processors.push(worker::Processor::SnakeCase);
            }
            NameProcessor::FirstLetterUppercase => {
                processors.push(worker::Processor::FirstLetterUppercase);
            }
        }
    }

    Ok(processors)
}
