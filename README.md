# ruspell

Executable to perform a spell-check on csv file
(first aimed at french transportation stop-signs data)


## Usage

Compile
```bash
cargo build --release
```

You can use ruspell as follows
```bash
target/release/ruspell -i stops.txt -c config.yml -r rules.csv -o stops_out.txt
```
> You may find files examples (including config file) in `tests/data` directory.


## Requirements

You probably want to install aspell before use
(especially if you activate `IspellChecker` in conf).

On debian (here for french spellcheck):
```bash
sudo apt-get install aspell-fr
```


## Tests

Ruspell is tested running on a fr-idf file.

Simply do:
```bash
sh tests/fr-idf.sh
```
> If diffs are displayed, tests failed.


## Configuration

The configuration file allows management of the processing sequence to be applied to the csv file.
The order of the sequence in conf is respected (and matters most of the time).

Processors available are:

### Decode:
Decode double-encoded files.
You can specify any encoding to decode from within the list being matched by
[rust_encoding](https://github.com/lifthrasiir/rust-encoding/blob/master/src/label.rs).

Ex:
```yaml
  - Decode:
      from_encoding: iso_8859-15 # latin9
```


### RegexReplace:
Replace all matches of a `from` regular expression by the provided `to` expression.
By default the matching is case-insensitive (use `(?-i)` to specify otherwise).
You can use the whole [rust regex-syntax](https://doc.rust-lang.org/regex/regex/index.html#syntax)
in your expressions.

Ex:
```yaml
  - RegexReplace:
      from: "(^|\\W)pl\\.?(\\W|$)"
      to: "${1}Place${2}"
```
> This will replace any `pl` or `pl.` preceded and followed by non-alphanumeric character
> by `Place` (`${1}` and `${2}` are just pasting matched previous and following characters).


### IspellCheck:
Perform a spellcheck using aspell (and its dictionnary).
Replaces a word only if normed version (no accent, case-insensitive) of the word is the same.
You can provide a list of bano CSV files so that
street and city names are added to aspell dictionary.

Ex:
```yaml
  - IspellCheck:
      dictionnary: "fr"
      bano_files:
        - "bano/bano-75.csv"
        - "bano/bano-77.csv"
```


### SnakeCase:
Change case to snake-case on whole name (all lowercase, first letter of each word uppercase).

Ex:
```yaml
  - SnakeCase
```
> This will change `HELLO i'm A Random naME` to `Hello I'M A Random Name`.


### UppercaseWord:
Change case of all word matching (case-insensitive) one of the regex in the list,
so that word is full uppercase.

Example:
```yaml
  - UppercaseWord:
      words:
        - RER
        - "\\w*\\d\\w*" # words containing a digit
```
> This will change `rER` to `RER` and `Rn25bis` to `RN25BIS`.


### LowercaseWord:
Change case of all word matching (case-insensitive) one of the regex in the list,
so that word is full lowercase.

Example:
```yaml
  - UppercaseWord:
      words:
        - de
        - "\\d+([eè]me|[eè]re?|nde?)" # manage "2ème" (has to be after uppercase management)
```
> This will change `dE` to `de` and `2NDE` to `2nde`.


### FirstLetterUppercase:
Change case to upper only for the first letter of the name.

Example:
```yaml
  - FirstLetterUppercase
```
> This will change `hello. i'M a meSsage - random` to `Hello. i'M a meSsage - random`


### LogSuspicious:
Output a warning log for each match with the provided regex.

Example:
```yaml
  - LogSuspicious:
      regex: "[^\\w \\(\\)]"
```
> This will output a warning for each character that
> is none of alphanumeric, space or parenthesis.
