use crate::validation_sets::ItemValidationSets;

use std::collections::HashSet;

extern crate yaml_rust;
use yaml_rust::{ScanError, Yaml, YamlLoader};

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
    fn include(&mut self, other: ValidationResults) {
        // need to make other mutable to move vec contents
        let mut other = other;

        self.valid = self.valid && other.valid;
        self.pass_messages.append(&mut other.pass_messages);
        self.fail_messages.append(&mut other.fail_messages);
    }

    fn new() -> ValidationResults {
        ValidationResults {
            valid: true,
            pass_messages: Vec::new(),
            fail_messages: Vec::new(),
        }
    }
}



/// top level validation function for an item, returns ValidationResults, which contains a flag
/// for whether the item is valid or not, and lists of pass and fail messages
pub fn validate_item_contents(
    contents: &str,
    item_validation_sets: &ItemValidationSets,
) -> Result<ValidationResults, ScanError> {
    let mut results = ValidationResults::new();

    let docs = YamlLoader::load_from_str(contents)?;
    // YAML files can actually contain multiple files inside, we want the first one
    let yaml = &docs[0];

    // validate the presence of the keys that all items have
    results.include(validate_key(&yaml, "Name", true));
    results.include(validate_key(&yaml, "Item Number", true));
    results.include(validate_key(&yaml, "Level", true));
    results.include(validate_list(
        &yaml,
        "Category",
        &item_validation_sets.categories,
        true,
    ));
    results.include(validate_list(
        &yaml,
        "Classifications",
        &item_validation_sets.classifications,
        true,
    ));
    results.include(validate_list(
        &yaml,
        "Element",
        &item_validation_sets.elements,
        true,
    ));

    let synthesis_required = synthesis::should_have_synthesis(&yaml);

    results.include(validate_list(
        &yaml,
        "Materials",
        &item_validation_sets.materials,
        synthesis_required,
    ));
    if synthesis_required {
        results.include(synthesis::validate_synthesis(
            &yaml["Synthesis"],
            item_validation_sets,
        ));
    }
    Ok(results)
}

/// Check to see if a particular key is a child of the given yaml position
/// (if the key isn't required, it's absence goes unremarked)
fn validate_key(yaml: &Yaml, key: &str, required: bool) -> ValidationResults {
    let mut results = ValidationResults::new();

    let value = &yaml[key];

    if value.is_badvalue() && required {
        results
            .fail_messages
            .push(format!("'{}' key is missing", key));
        results.valid = false;
    } else if required {
        let value_message = match value {
            Yaml::String(value) => format!(": {}", value),
            Yaml::Integer(value) => format!(": {}", value),
            _ => "".to_string(),
        };
        let pass_message = format!("{} is present{}", key, value_message);
        results.pass_messages.push(pass_message);
        results.valid = true;
    }
    results
}

/// Check to see if a particular key is a child of the given yaml position
/// (if the key isn't required, it's absence goes unremarked)
fn validate_key_and_value(yaml: &Yaml, key: &str, validation_set: &HashSet<String>,  required: bool) -> ValidationResults {
    let mut results = ValidationResults::new();

    let value = &yaml[key];

    if value.is_badvalue() && required {
        results
            .fail_messages
            .push(format!("'{}' key is missing", key));
        results.valid = false;
    } else if required {
        match value {
            Yaml::String(value) => {
                if validation_set.contains(value){
                    results.pass_messages.push(format!("key {}: {} is a known value", key, value));
                } else {
                    results.fail_messages.push(format!("key {}: {} is an unknown value", key, value));
                }
            },
            _ => {},
        };
        let pass_message = format!("{} is present", key);
        results.pass_messages.push(pass_message);
        results.valid = true;
    }
    results
}

/// top level method for validating a list of the given YAML position
fn validate_list(
    yaml: &Yaml,
    key: &str,
    validation_set: &HashSet<String>,
    required: bool,
) -> ValidationResults {
    let mut results = validate_key(yaml, key, required);

    if results.valid {
        results.include(validate_list_values(yaml, key, validation_set, required));
    }
    results
}

/// validate a list of the given key with a given set of allowed values
fn validate_list_values(
    yaml: &Yaml,
    key: &str,
    validation_set: &HashSet<String>,
    required: bool,
) -> ValidationResults {
    let mut results = ValidationResults::new();

    let value = &yaml[key];

    if let Yaml::Array(list) = value {
        results.valid = true;
        for value in list {
            if let Yaml::String(value) = value {
                if !validation_set.contains(value) {
                    results.valid = false;
                    results
                        .fail_messages
                        .push(format!("{}: {} is an unknown value", key, value));
                }
            }
            if let Yaml::Hash(value) = value {
                for (hash_key, _) in value {
                    if let Yaml::String(hash_key) = hash_key {
                        if !validation_set.contains(hash_key) {
                            results.valid = false;
                            results
                                .fail_messages
                                .push(format!("{}: {} is an unknown value", key, hash_key));
                        }
                    }
                }
            }
        }

        if results.valid {
            results
                .pass_messages
                .push(format!("{} values are known", key));
        }
    } else if required {
        results.valid = false; // not a yaml list (a vector)
        results.fail_messages.push(format!("{} is not a list", key));
    }

    results
}

/// the synthesis part of validation is complex enough to warrant its own module
mod synthesis {
    use crate::validate_item::{validate_key, validate_key_and_value, validate_list};
    use crate::validate_item::{ItemValidationSets, ValidationResults};
    use std::collections::HashSet;
    use yaml_rust::Yaml;

    /// if the item has a classification of "Materials", then the item is a gathered item. Otherwise,
    /// it's a synthesized item.
    pub fn should_have_synthesis(yaml: &Yaml) -> bool {
        let value = &yaml["Classifications"];

        let no_synthesis_classification = "Materials";
        let mut has_systhesis_classification = true;

        if let Yaml::Array(list) = value {
            for value in list {
                if let Yaml::String(value) = value {
                    if value == no_synthesis_classification {
                        has_systhesis_classification = false
                    }
                }
            }
        }
        has_systhesis_classification
    }

    pub fn validate_synthesis(
        yaml: &Yaml,
        item_validation_sets: &ItemValidationSets,
    ) -> ValidationResults {
        let mut results = ValidationResults::new();

        if yaml.is_badvalue() {
            results
                .fail_messages
                .push("Synthesis key is missing.".to_string());
        } else {
            results.include(validate_key(yaml, "Required Materials", true));
            results.include(validate_material_loops(
                &yaml["Material Loops"],
                item_validation_sets,
            ));

            // prepend validation messages with the Synthesis key
            results.pass_messages = results
                .pass_messages
                .drain(0..)
                .map(|msg| format!("{}: {}", "Synthesis", msg))
                .collect();
            results.fail_messages = results
                .fail_messages
                .drain(0..)
                .map(|msg| format!("{}: {}", "Synthesis", msg))
                .collect();
        }
        results
    }

    fn validate_material_loops(
        yaml: &Yaml,
        item_validation_sets: &ItemValidationSets,
    ) -> ValidationResults {
        let mut results = ValidationResults::new();
        if yaml.is_badvalue() {
            results
                .fail_messages
                .push("Material Loops key is missing.".to_string());
        } else if let Yaml::Array(material_loops) = yaml {
            // check to see if position values are unique
            results.include(validate_unique_positions(material_loops));
            for material_loop in material_loops {
                results.include(validate_material_loop_contents(
                    material_loop,
                    item_validation_sets,
                ));
            }
        }
        results
    }

    /// validate if position values are unique for each material loop
    fn validate_unique_positions(material_loops: &[Yaml]) -> ValidationResults {
        let mut results = ValidationResults::new();
        let mut position_set = HashSet::new();
        for material_loop in material_loops {
            if let Yaml::Hash(material_loop_hash) = material_loop {
                for (name, details) in material_loop_hash {
                    if let Yaml::String(name) = name {
                        match &details["Position"] {
                            Yaml::BadValue => results
                                .fail_messages
                                .push(format!("loop '{}' is missing position", name)),
                            Yaml::Integer(position) => {
                                if position_set.contains(&position) {
                                    results.fail_messages.push(format!(
                                        "loop '{}' has duplicate position value: {}",
                                        name, position
                                    ));
                                } else {
                                    results.pass_messages.push(format!(
                                        "loop '{}' has new position value: {}",
                                        name, position
                                    ));
                                    position_set.insert(position);
                                }
                            }
                            _ => {}
                        };
                    }
                }
            }
        }
        results
    }

    fn validate_material_loop_contents(
        yaml: &Yaml,
        item_validation_sets: &ItemValidationSets,
    ) -> ValidationResults {
        let mut results = ValidationResults::new();

        if let Yaml::Hash(material_loop_hash) = yaml {
            for (name, details) in material_loop_hash {
                results.include(validate_key(details, "Distance", true));
                results.include(validate_key(details, "Position", true));
                // consider changing from list of 1 to validate key and value
                results.include(validate_key_and_value(
                    details,
                    "Material",
                    &item_validation_sets.materials,
                    true,
                ));
                results.include(validate_key(details, "Linked From Position", false));
                results.include(validate_list(
                    details,
                    "Unlock",
                    &item_validation_sets.elements,
                    false,
                ));
                results.include(validate_loop_levels(
                    &details["Levels"],
                    &item_validation_sets,
                ));

                // prepend validation messages with the Synthesis key
                if let Yaml::String(name) = name {
                    results.pass_messages = results
                        .pass_messages
                        .drain(0..)
                        .map(|msg| format!("{}: {}", name, msg))
                        .collect();
                    results.fail_messages = results
                        .fail_messages
                        .drain(0..)
                        .map(|msg| format!("{}: {}", name, msg))
                        .collect();
                }
            }
        } else {
            results
                .fail_messages
                .push("Malformed material loop".to_string());
        }
        results
    }

    fn validate_loop_levels(
        yaml: &Yaml,
        item_validation_sets: &ItemValidationSets,
    ) -> ValidationResults {
        let mut results = ValidationResults::new();
        if let Yaml::Array(level_vec) = yaml {
            for level in level_vec {
                if let Yaml::Hash(level_hash) = level {
                    for (loop_effect, details) in level_hash {
                        if let Yaml::String(loop_effect) = loop_effect {
                            results.include(validate_list(
                                &details,
                                "Element",
                                &item_validation_sets.elements,
                                true,
                            ));
                            let is_recipe_morph = loop_effect == "Recipe Morph";
                            results.include(validate_key(
                                &details,
                                "Required Alchemy Level",
                                is_recipe_morph,
                            ));
                            results.include(validate_key_and_value(
                                &details,
                                "Recipe",
                                &item_validation_sets.materials,
                                is_recipe_morph,
                            ));

                            // prepend validation messages with the loop effect
                            results.pass_messages = results
                                .pass_messages
                                .drain(0..)
                                .map(|msg| format!("{}: {}", loop_effect, msg))
                                .collect();
                            results.fail_messages = results
                                .fail_messages
                                .drain(0..)
                                .map(|msg| format!("{}: {}", loop_effect, msg))
                                .collect();
                        }
                    }
                }
            }
        }
        results
    }
}
