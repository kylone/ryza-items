mod file_contents;
mod validate_item;

extern crate term;

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
    let verbose = true;

    let item_validation_sets = get_item_validation_sets();

    let mut item_contents: Vec<file_contents::FileContents> = Vec::new();
    file_contents::load_item_files(&mut item_contents, "../../../data/items/").unwrap();

    for file in item_contents {
        println!("Validating {}", file.name);
        let result = validate_item::validate_item_contents(&file.contents, &item_validation_sets);

        // there's a some .unwrap() calls with the term crate
        // if we can't work with the terminal, just panic
        let mut terminal = term::stdout().unwrap();
        if let Ok(results) = result {
            // display results
            if verbose {
                terminal.fg(term::color::BRIGHT_GREEN).unwrap();
                for msg in results.pass_messages {
                    println!("- {}", msg);
                }
                terminal.reset().unwrap();
            }
            terminal.fg(term::color::BRIGHT_RED).unwrap();
            for msg in results.fail_messages {
                println!("- {}", msg);
            }
            terminal.reset().unwrap();
        } else {
            terminal.fg(term::color::BRIGHT_RED).unwrap();
            println!("unable to validate {}", file.name);
            terminal.reset().unwrap();
        }
    }
}
