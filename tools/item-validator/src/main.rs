mod file_contents;
mod validate_item;

#[macro_use]
extern crate lazy_static;

fn get_item_validation_sets() -> validate_item::ItemValidationSets {
    let item_list_contents =
        file_contents::load_lists_file("../../../data/lists.yml").expect("can't load lists.yml");

    let item_validation_sets = validate_item::build_item_validation_sets(&item_list_contents)
        .expect("can't parse item validation lists");

    println!("Parsed lists.yml:");
    println!(
        "  Found {} Item Categories",
        item_validation_sets.categories.len()
    );
    println!(
        "  Found {} Item Classifications",
        item_validation_sets.classifications.len()
    );
    println!("  Found {} Elements", item_validation_sets.elements.len());
    println!(
        "  Found {} Gathering Tools",
        item_validation_sets.gathering_tools.len()
    );
    println!();

    item_validation_sets
}

fn main() {
    let item_validation_sets = get_item_validation_sets();

    let mut item_contents: Vec<file_contents::FileContents> = Vec::new();
    file_contents::load_item_files(&mut item_contents, "../../../data/items/").unwrap();

    for file in item_contents {
        println!("Validating {}", file.name);
        let only_output_invalid = true;
        let result = validate_item::validate_item_contents(
            &file.contents,
            &item_validation_sets,
            only_output_invalid,
        );
        if result.is_err() {
            println!("unable to validate {}", file.name)
        }
    }
}
