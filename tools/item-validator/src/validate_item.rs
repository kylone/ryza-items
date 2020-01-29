extern crate yaml_rust;
use std::collections::HashSet;
use yaml_rust::{ScanError, Yaml, YamlLoader};
use std::fmt::Write;



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

fn validate_key_exists(yaml: &Yaml, key: &str, required: bool) -> ValidationResults {
    let mut results = ValidationResults::new(); 

    let value = &yaml[key];

    // there's a whole bunch of .unwrap() calls with the term crate
    // if we can't work with the terminal, just panic


    if value.is_badvalue() && required {
        results.fail_messages.push(format!("'{}' key is missing", key));
        results.valid = false;
    } else if required {

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

fn validate_list_values(
    yaml: &Yaml,
    validation_set: &HashSet<String>,
    key: &str,
    required:bool,
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
        if required { 
        results.valid = false; // not a yaml list (a vector)
        results.fail_messages.push(format!("{} is not a list", key));
        }
    }
    results
}

fn validate_list( yaml: &Yaml,
    validation_set: &HashSet<String>,
    key: &str,
    required:bool) -> ValidationResults{
    let mut results = validate_key_exists(yaml, key, required);

    if results.valid  {
        results.include(validate_list_values(
            yaml,
            validation_set,
            key,
            required,
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
    let yaml = &docs[0];

    // validate the presence of the keys that all items have
    results.include(validate_key_exists(&yaml, "Name", true));
    results.include(validate_key_exists(&yaml, "Item Number", true));
    results.include(validate_key_exists(&yaml, "Level", true));
    results.include(validate_list(&yaml,&item_validation_sets.categories, "Category", true));
    results.include(validate_list(&yaml,&item_validation_sets.classifications, "Classifications", true ));
    results.include(validate_list(&yaml,&item_validation_sets.elements, "Element", true ));
    
    let should_have_synthesis = should_have_synthesis(&yaml);

    results.include(validate_list(&yaml,&item_validation_sets.categories, "Materials", should_have_synthesis));
    
    Ok(results)
}


/// if the item has a classification of "Materials", then the item is a gathered item
fn should_have_synthesis(yaml: &Yaml) -> bool {
    
    let value = &yaml["Classifications"];

    let no_synthesis_classification = "Materials";
    let mut has_systhesis_classification = true;

    if let Some(list) = value.as_vec() {
        for value in list {
            if let Some(value) = value.as_str() {
                if value == no_synthesis_classification {
                    has_systhesis_classification = false
                }
            }
        }
    }
    has_systhesis_classification
}