// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Ispell(::ispell::Error);
        Csv(::csv::Error);
        Regex(::regex::Error);
        Fmt(::std::fmt::Error);
    }

    errors {
        ColumnNotFound(t: String) {
            description("column not found")
            display("column {} not found", t)
        }
    }
}
