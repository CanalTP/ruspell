use regex_processor;
use ispell_wrapper;
use param;

pub enum Processor {
    Fixedcase(regex_processor::FixedcaseProcessor),
    RegexReplace(regex_processor::RegexReplace),
    Ispell(ispell_wrapper::SpellCheck),
    Decode(param::Decode),
    SnakeCase,
    FirstLetterUppercase,
    LogSuspicious(regex_processor::LogSuspicious),
}
