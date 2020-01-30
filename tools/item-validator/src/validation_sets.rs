use std::collections::HashSet;

extern crate yaml_rust;
use yaml_rust::{ScanError, Yaml, YamlLoader};

pub struct ItemValidationSets {
    pub elements: HashSet<String>,
    pub categories: HashSet<String>,
    pub classifications: HashSet<String>,
    pub gathering_tools: HashSet<String>,
    pub materials: HashSet<String>,
}

impl ItemValidationSets {
    pub fn new() -> ItemValidationSets {
        ItemValidationSets {
            elements: HashSet::new(),
            categories: HashSet::new(),
            classifications: HashSet::new(),
            gathering_tools: HashSet::new(),
            materials: HashSet::new(),
        }
    }
}

pub fn build_item_validation_sets(contents: &str) -> Result<ItemValidationSets, ScanError> {
    let docs = YamlLoader::load_from_str(contents)?;
    // YAML files can actually contain multiple files inside, we want the first one
    let yaml = &docs[0];
    let mut validation_sets = ItemValidationSets::new();
    add_to_set(&yaml, "Item Categories", &mut validation_sets.categories);
    add_to_set(&yaml, "Item Categories", &mut validation_sets.materials);
    add_to_set(
        &yaml,
        "Item Classifications",
        &mut validation_sets.classifications,
    );
    add_to_set(&yaml, "Elements", &mut validation_sets.elements);
    add_to_set(
        &yaml,
        "Gathering Tools",
        &mut validation_sets.gathering_tools,
    );
    Ok(validation_sets)
}

fn add_to_set(yaml: &Yaml, key: &str, set: &mut HashSet<String>) {
    let value = &yaml[key];

    if let Yaml::Array(list) = value {
        for value in list {
            if let Yaml::String(value) = value {
                set.insert(value.to_string());
            }
        }
    }
    set.shrink_to_fit();
}