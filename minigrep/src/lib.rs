use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

pub struct Config {
    pub query: String,
    pub filename: String,
    pub case_sensitive: bool,
    pub search_fn: for <'r, 's> fn(&'r str, &'s str) -> Vec<&'s str>,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };
        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file name"),
        };

        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();
        let search_fn: for<'r, 's> fn(&'r str, &'s str) -> Vec<&'s str>;
        search_fn = if case_sensitive
            { search } else { search_case_insensitive };

        Ok(Config { query, filename, case_sensitive, search_fn })
    }
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let mut f = File::open(config.filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let results = (config.search_fn)(&config.query, &contents);
        
    for line in results {
        println!("{}", line);
    }

    Ok(())
}

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines()
        .filter(|line| line.contains(query))
        .collect()
}

fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = &query.to_lowercase();

    contents.lines()
        .filter(|line| line.to_lowercase().contains(query))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    static CONTENTS: &'static str = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.
It's all duct tape and baloney.";

    #[test]
    fn one_result() {
        let query = "fast";
        
        assert_eq!(
            vec!["safe, fast, productive."],
            search(query, CONTENTS));
    }

    #[test]
    fn case_sensitive() {
        let query = "Pick";

        assert_eq!(
            vec!["Pick three."],
            search(query, CONTENTS));
    }

    #[test]
    fn case_insensitive() {
        let query = "ruSt";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, CONTENTS));
    }
}

