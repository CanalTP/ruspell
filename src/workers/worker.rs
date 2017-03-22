use workers::regex_processor;
use workers::ispell_wrapper;
use param;
use utils;
use errors::Result;

pub enum Processor {
    Fixedcase(regex_processor::FixedcaseProcessor),
    RegexReplace(regex_processor::RegexReplace),
    Ispell(ispell_wrapper::SpellCheck),
    Decode(param::Decode),
    SnakeCase,
    FirstLetterUppercase,
    LogSuspicious(regex_processor::LogSuspicious),
}
impl Processor {
    pub fn apply(&mut self, name: &str) -> Result<String> {
        match *self {
            Processor::Fixedcase(ref p) => Ok(p.process(name)),
            Processor::RegexReplace(ref p) => Ok(p.process(name)),
            Processor::Ispell(ref mut p) => p.process(name),
            Processor::Decode(ref d) => utils::decode(name, &d.from_encoding),
            Processor::SnakeCase => Ok(utils::snake_case(name)),
            Processor::FirstLetterUppercase => Ok(utils::first_upper(name)),
            Processor::LogSuspicious(ref l) => {
                l.process(name);
                Ok(name.to_string())
            }
        }
    }
}
