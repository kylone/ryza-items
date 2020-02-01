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
    fn include(&mut self, other: ValidationResults) -> &mut ValidationResults {
        // need to make other mutable to move vec contents
        let mut other = other;

        self.valid = self.valid && other.valid;
        self.pass_messages.append(&mut other.pass_messages);
        self.fail_messages.append(&mut other.fail_messages);
        self
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
    match value {
        Yaml::BadValue if required => {
            results
                .fail_messages
                .push(format!("'{}' key is missing", key));
            results.valid = false;
        }
        Yaml::BadValue => {} // if this key isn't required, BadValue means do nothing
        Yaml::String(value) => {
            results
                .pass_messages
                .push(format!("{} is present: {}", key, value));
            results.valid = true;
        }
        Yaml::Integer(value) => {
            results
                .pass_messages
                .push(format!("{} is present: {}", key, value));
            results.valid = true;
        }
        _ => {
            results.pass_messages.push(format!("{} is present", key));
            results.valid = true;
        }
    }
    results
}

/// Check to see if a particular key is a child of the given yaml position
/// (if the key isn't required, it's absence goes unremarked)
fn validate_key_and_value(
    yaml: &Yaml,
    key: &str,
    validation_set: &HashSet<String>,
    required: bool,
) -> ValidationResults {
    let mut results = ValidationResults::new();

    let value = &yaml[key];
    match value {
        Yaml::BadValue if required => {
            results
                .fail_messages
                .push(format!("'{}' key is missing", key));
            results.valid = false;
        }
        Yaml::BadValue => {} // if this key isn't required, BadValue means do nothing
        Yaml::String(value) => {
            if validation_set.contains(value) {
                results
                    .pass_messages
                    .push(format!("key {}: known value '{}'", key, value));
            } else {
                results
                    .fail_messages
                    .push(format!("key {}: unknown value '{}' (typo, or item file needed)", key, value));
            }
        }
        _ => {
            results
                .fail_messages
                .push(format!("key {}: value is not a string", key));
        }
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
            if let Yaml::Hash(hash_map) = value {
                for hash_key in hash_map
                    .iter()
                    .filter_map(|(hash_key, _)| hash_key.as_str())
                {
                    if !validation_set.contains(hash_key) {
                        results.valid = false;
                        results
                            .fail_messages
                            .push(format!("{}: {} is an unknown value", key, hash_key));
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
    use std::collections::HashMap;
    use yaml_rust::Yaml;

    /// if the item has a classification of "Materials", then the item is a gathered item. Otherwise,
    /// it's a synthesized item.
    pub fn should_have_synthesis(yaml: &Yaml) -> bool {
        let value = &yaml["Classifications"];

        let no_synthesis_classification = "Materials";
        let mut has_synthesis_classification = true;

        if let Yaml::Array(list) = value {
            for value in list {
                if let Yaml::String(value) = value {
                    if value == no_synthesis_classification {
                        has_synthesis_classification = false
                    }
                }
            }
        }
        has_synthesis_classification
    }

    pub fn validate_synthesis(
        yaml: &Yaml,
        item_validation_sets: &ItemValidationSets,
    ) -> ValidationResults {
        let mut results = ValidationResults::new();

        if let Yaml::BadValue = yaml {
            results
                .fail_messages
                .push(String::from("Synthesis key is missing."));
        } else {
            results.include(validate_key(yaml, "Required Materials", true));
            results.include(validate_key(yaml, "Required Alchemy Level", true));
            results.include(validate_material_loops(
                &yaml["Material Loops"],
                item_validation_sets,
            ));

            // prefix validation messages with the Synthesis key
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
        match yaml {
            Yaml::BadValue => {
                results
                    .fail_messages
                    .push("Material Loops key is missing.".to_string());
            }
            Yaml::Array(material_loops) => {
                // check to see if position values are unique
                results.include(validate_unique_positions(material_loops));

                for material_loop in material_loops {
                    results.include(validate_material_loop_contents(
                        material_loop,
                        item_validation_sets,
                    ));
                }
            }
            _ => {
                results
                    .fail_messages
                    .push(String::from("Material Loops key is not a list."));
            }
        }
        results
    }

    /// validate if position values are unique for each material loop, and distance values make sense.
    #[allow(clippy::map_entry)] // not sure how to make this method better using std::collections::hash_map::Entry...
    fn validate_unique_positions(material_loops: &[Yaml]) -> ValidationResults {
        let mut results = ValidationResults::new();
        let mut hash_map = HashMap::new();
        for material_loop in material_loops.iter().filter_map(|yaml| yaml.as_hash()) {
            // populate hash_map with position -> distance maps,
            // ensuring unique positions for each loop
            for (name, details) in material_loop {
                if let (Yaml::String(name), Yaml::Integer(position), Yaml::Integer(distance)) =
                    (name, &details["Position"], &details["Distance"])
                {
                    if hash_map.contains_key(&position) {
                        results.fail_messages.push(format!(
                            "loop '{}' has duplicate position value: {}",
                            name, position
                        ));
                    } else {
                        results.pass_messages.push(format!(
                            "loop '{}' has new position value: {}",
                            name, position
                        ));
                        hash_map.insert(position, distance);
                    }
                }
            }
            // now, confirm Linked From Position keys
            for (name, details) in material_loop {
                if let (
                    Yaml::String(name),
                    Yaml::Integer(position),
                    Yaml::Integer(linked_from_position),
                    Yaml::Integer(distance),
                ) = (
                    name,
                    &details["Position"],
                    &details["Linked From Position"],
                    &details["Distance"],
                ) {
                    let linked_from_distance = hash_map[linked_from_position];
                    if linked_from_distance >= distance {
                        results.fail_messages.push(format!(
                            "loop '{}' position {}: linked to loop with same or higher distance value",
                            name, position
                        ));
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

                // prefix validation messages with the material loop name/type
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
            for level in level_vec.iter().filter_map(|level| level.as_hash()) {
                for (loop_effect, details) in level {
                    if let Yaml::String(loop_effect) = loop_effect {
                        results.include(validate_list(
                            &details,
                            "Element",
                            &item_validation_sets.elements,
                            true,
                        ));
                        let is_recipe_morph = loop_effect == "Recipe Morph";
                        results.include(validate_key_and_value(
                            &details,
                            "Recipe",
                            &item_validation_sets.materials,
                            is_recipe_morph,
                        ));

                        // prefix validation messages with the loop effect name
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
        results
    }
}
