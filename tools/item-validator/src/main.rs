mod file_contents;
mod validate_item;

fn main() {
    let item_lists = file_contents::load_lists_file("../../../data/lists.yml")
                         .expect("can't load lists.yml"); 

    println!("{}", item_lists);
    let mut item_contents: Vec<file_contents::FileContents> = Vec::new();
    file_contents::load_item_files(&mut item_contents, "../../../data/items/").unwrap();

    for file in item_contents {
        println!("Validating {}", file.name);

        let result = validate_item::validate_item_contents(&file.contents);
        if result.is_err() {
            println!("unable to validate {}", file.name)
        }
    }
}
