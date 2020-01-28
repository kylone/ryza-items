extern crate yaml_rust;
use std::collections::HashSet;
use yaml_rust::{ScanError, Yaml, YamlLoader};
use std::fmt::Write;


use regex::Regex;

pub struct ValidationResults {
    pub valid: bool,
    pub pass_messages: Vec<String>,
    pub fail_messages: Vec<String>,
}

impl ValidationResults {
    /// combine one set of results into another.  
    /// 
    /// The intent here is to have each type of validation to add messages to the total result
    /// while providing a means to reach to a particular result's `valid` value.
    fn include(&mut self, other: ValidationResults){
        // need to make other mutable to move vec contents
        let mut other = other;

        self.valid = self.valid && other.valid;
        self.pass_messages.append(&mut other.pass_messages);
        self.fail_messages.append(&mut other.fail_messages);
    }

    fn new() -> ValidationResults{
        ValidationResults{
            valid:true,
            pass_messages: Vec::new(),
            fail_messages: Vec::new(),
        }
    }
}

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

fn validate_key_exists(yaml: &Yaml, key: &str, optional: bool) -> ValidationResults {
    let mut results = ValidationResults::new(); 

    let value = &yaml[key];

    // there's a whole bunch of .unwrap() calls with the term crate
    // if we can't work with the terminal, just panic


    if value.is_badvalue() && !optional {
        results.fail_messages.push(format!("'{}' key is missing", key));
        results.valid = false;
    } else if !optional {

        let mut pass_message = String::new();
        write!(&mut pass_message,"{} is present", key ).unwrap();

        if let Some(value) = value.as_str() {
            write!(&mut pass_message,": {}", value).unwrap();
        } else if let Some(value) = value.as_i64() {
            write!(&mut pass_message,": {}", value).unwrap();
        } 
        results.pass_messages.push(pass_message);
        true; // validation successful
    }
    results
}

fn _validate_materials(
    yaml: &Yaml,
    validation_set: &HashSet<String>,
    key: &str,
    only_output_invalid: bool,
) -> bool {
    let value = &yaml[key];
    let mut terminal = term::stdout().unwrap();

    lazy_static! {
        static ref CATEGORY_REGEX: Regex = Regex::new(r#"\(\w+\)"#).unwrap();
    }
    if let Some(list) = value.as_vec() {
        let mut all_values_valid = true;
        terminal.fg(term::color::BRIGHT_RED).unwrap();
        for value in list {
            if let Some(value) = value.as_str() {
                if CATEGORY_REGEX.is_match(value) && !validation_set.contains(value) {
                    all_values_valid = false;
                    println!("{}: {} is not a valid value", key, value);
                }
            }
        }
        terminal.reset().unwrap();
        if all_values_valid && !only_output_invalid {
            terminal.fg(term::color::BRIGHT_GREEN).unwrap();
            println!("{} values are valid", key);
            terminal.reset().unwrap();
        }
        true
    } else {
        false // not a yaml list (a vector)
    }
}

fn validate_list_values(
    yaml: &Yaml,
    validation_set: &HashSet<String>,
    key: &str,
    optional:bool,
) -> ValidationResults {
    let mut results = ValidationResults::new();

    let value = &yaml[key];

    if let Some(list) = value.as_vec() {
        results.valid = true;
        for value in list {
            if let Some(value) = value.as_str() {
                if !validation_set.contains(value) {
                    results.valid = false;
                    results.fail_messages.push(
                        format!("{}: {} is not a valid value", key, value));
                }
            }
            if let Some(value) = value.as_hash() {
                for (hash_key, _) in value {
                    if let Some(hash_key) = hash_key.as_str() {
                        if !validation_set.contains(hash_key) {
                            results.valid = false;
                            results.fail_messages.push(
                                format!("{}: {} is not a valid value", key, hash_key));
                        }
                    }
                }
            }
        }

        if results.valid {
            results.pass_messages.push(format!("{} values are valid", key));
        }
        
    } else {
        if !optional { 
        results.valid = false; // not a yaml list (a vector)
        results.fail_messages.push(format!("{} is not a list", key));
        }
    }
    results
}

fn validate_list( yaml: &Yaml,
    validation_set: &HashSet<String>,
    key: &str,
    optional:bool) -> ValidationResults{
    let mut results = validate_key_exists(yaml, key, optional);

    if results.valid  {
        results.include(validate_list_values(
            yaml,
            validation_set,
            key,
            optional,
             ));
    }
    results
}

pub fn validate_item_contents(
    contents: &str,
    item_validation_sets: &ItemValidationSets,
) -> Result<ValidationResults, ScanError> {

    let mut results = ValidationResults::new();

    let docs = YamlLoader::load_from_str(contents)?;
    // YAML files can actually contain multiple files inside, we want the first one
    let doc = &docs[0];
    results.include(validate_key_exists(&doc, "Name", false));
    results.include(validate_key_exists(&doc, "Item Number", false));
    results.include(validate_key_exists(&doc, "Level", false));
    results.include(validate_list(&doc,&item_validation_sets.categories, "Category", false));
    results.include(validate_list(&doc,&item_validation_sets.classifications, "Classifications", false ));
    results.include(validate_list(&doc,&item_validation_sets.elements, "Element", false ));
    
    results.include(validate_list(&doc,&item_validation_sets.categories, "Materials", true));
    
    Ok(results)
}
