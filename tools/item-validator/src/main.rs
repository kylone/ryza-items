use std::env;

mod file_contents;
mod validate_item;
mod validation_sets;
mod settings;

extern crate term;

fn get_item_validation_sets(path:&str) -> validation_sets::ItemValidationSets {
    let list_path = format!("{}/lists.yml", path);
    let item_list_contents =
        file_contents::load_file(&list_path).expect("can't load lists.yml");

    let item_validation_sets = validation_sets::build_item_validation_sets(&item_list_contents)
        .expect("can't parse item validation lists");
    println!("Parsed lists.yml:");
    item_validation_sets
}

fn main() {
    // setup the verbose parameter
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let verbose = args.contains(&"verbose".to_string());

    let settings = settings::get_settings();

    let item_validation_sets = get_item_validation_sets(&settings.data_folder);

    let mut item_contents: Vec<file_contents::FileContents> = Vec::new();
    let item_dir_path = format!("{}/items/", settings.data_folder);
    file_contents::load_directory(&mut item_contents, &item_dir_path).unwrap();

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
