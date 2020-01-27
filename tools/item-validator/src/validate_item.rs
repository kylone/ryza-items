extern crate yaml_rust;
use std::collections::HashSet;
use yaml_rust::{ScanError, Yaml, YamlLoader};

extern crate term;

pub struct ItemValidationSets {
    pub elements: HashSet<String>,
    pub categories: HashSet<String>,
    pub classifications: HashSet<String>,
    pub gathering_tools: HashSet<String>,
}

impl ItemValidationSets {
    pub fn new() -> ItemValidationSets {
        ItemValidationSets {
            elements: HashSet::new(),
            categories: HashSet::new(),
            classifications: HashSet::new(),
            gathering_tools: HashSet::new(),
        }
    }
}

pub fn build_item_validation_sets(contents: &str) -> Result<ItemValidationSets, ScanError> {
    let docs = YamlLoader::load_from_str(contents)?;
    // YAML files can actually contain multiple files inside, we want the first one
    let doc = &docs[0];
    let mut validation_sets = ItemValidationSets::new();
    add_to_set(&doc, "Item Categories", &mut validation_sets.categories);
    add_to_set(
        &doc,
        "Item Classifications",
        &mut validation_sets.classifications,
    );
    add_to_set(&doc, "Elements", &mut validation_sets.elements);
    add_to_set(
        &doc,
        "Gathering Tools",
        &mut validation_sets.gathering_tools,
    );
    Ok(validation_sets)
}

fn add_to_set(yaml: &Yaml, key: &str, set: &mut HashSet<String>) {
    let value = &yaml[key];

    if let Some(list) = value.as_vec() {
        for value in list {
            if let Some(value) = value.as_str() {
                set.insert(value.to_string());
            }
        }
    }
    set.shrink_to_fit();
}

fn validate_key_exists(yaml: &Yaml, key: &str, only_output_invalid: bool) -> bool {
    let value = &yaml[key];

    // there's a whole bunch of .unwrap() calls with the term crate
    // if we can't work with the terminal, just panic

    let mut terminal = term::stdout().unwrap();

    if value.is_badvalue() {
        terminal.fg(term::color::BRIGHT_RED).unwrap();
        println!("'{}' key is missing", key);
        terminal.reset().unwrap();
        false // validation unsuccessful
    } else {
        if !only_output_invalid {
            terminal.fg(term::color::BRIGHT_GREEN).unwrap();

            print!("  {} is present", key);
            terminal.reset().unwrap();

            if let Some(value) = value.as_str() {
                println!(": {}", value)
            } else if let Some(value) = value.as_i64() {
                println!(": {}", value)
            } else {
                println!();
            }
        }

        true // validation successful
    }
}

fn validate_list_values(
    yaml: &Yaml,
    validation_set: &HashSet<String>,
    key: &str,
    only_output_invalid: bool,
) -> bool {
    let value = &yaml[key];
    let mut terminal = term::stdout().unwrap();

    if let Some(list) = value.as_vec() {
        let mut all_values_valid = true;
        terminal.fg(term::color::BRIGHT_RED).unwrap();
        for value in list {
            if let Some(value) = value.as_str() {
                if !validation_set.contains(value) {
                    all_values_valid = false;
                    println!("  {}: {} is not a valid value", key, value);
                }
            }
            if let Some(value) = value.as_hash() {
                for (hash_key, _) in value {
                    if let Some(hash_key) = hash_key.as_str() {
                        if !validation_set.contains(hash_key) {
                            all_values_valid = false;
                            println!("  {}: {} is not a valid value", key, hash_key);
                        }
                    }
                }
            }
        }
        terminal.reset().unwrap();
        if all_values_valid && !only_output_invalid {
            terminal.fg(term::color::BRIGHT_GREEN).unwrap();
            println!("  {} values are valid", key);
            terminal.reset().unwrap();
        }
        true
    } else {
        false // not a yaml list (a vector)
    }
}

pub fn validate_item_contents(
    contents: &str,
    item_validation_sets: &ItemValidationSets,
    only_output_invalid: bool,
) -> Result<(), ScanError> {
    let docs = YamlLoader::load_from_str(contents)?;
    // YAML files can actually contain multiple files inside, we want the first one
    let doc = &docs[0];
    validate_key_exists(&doc, "Name", only_output_invalid);
    validate_key_exists(&doc, "Item Number", only_output_invalid);
    validate_key_exists(&doc, "Level", only_output_invalid);
    if validate_key_exists(&doc, "Category", only_output_invalid) {
        validate_list_values(
            &doc,
            &item_validation_sets.categories,
            "Category",
            only_output_invalid,
        );
    }
    if validate_key_exists(&doc, "Classifications", only_output_invalid) {
        validate_list_values(
            &doc,
            &item_validation_sets.classifications,
            "Classifications",
            only_output_invalid,
        );
    }
    if validate_key_exists(&doc, "Element", only_output_invalid) {
        validate_list_values(
            &doc,
            &item_validation_sets.elements,
            "Element",
            only_output_invalid,
        );
    }
    Ok(())
}
