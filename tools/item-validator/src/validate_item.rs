extern crate yaml_rust;
use yaml_rust::{ScanError, Yaml, YamlLoader};

extern crate term;

fn validate_yaml_key(yaml: &Yaml, key: &str) {
    let value = &yaml[key];

    // there's a whole bunch of .unwrap() calls with the term crate
    // if we can't work with the terminal, just panic

    let mut terminal = term::stdout().unwrap();

    if value.is_badvalue() {
        terminal.fg(term::color::BRIGHT_RED).unwrap();
        println!("'{}' key is missing", key);
        terminal.reset().unwrap();
    } else {
        terminal.fg(term::color::BRIGHT_GREEN).unwrap();

        print!("  {} is present", key);
        terminal.reset().unwrap();

        if let Some(value) = value.as_str() {
            println!(": {}", value)
        } else if let Some(value) = value.as_i64() {
            println!(": {}", value)
        } else {
            println!("");
        }
    }
}

pub fn validate_item_contents(contents: &str) -> Result<(), ScanError> {
    let docs = YamlLoader::load_from_str(contents)?;
    // YAML files can actually contain multiple files inside, we want the first one
    let doc = &docs[0];
    validate_yaml_key(&doc, "Name");
    validate_yaml_key(&doc, "Item Number");
    validate_yaml_key(&doc, "Level");
    validate_yaml_key(&doc, "Category");
    Ok(())
}
