use std::path::Path;
use std::{fs, io};

extern crate yaml_rust;
use yaml_rust::{YamlLoader, ScanError, Yaml};

extern crate term;

// code for str -> Yaml
//let item_doc = YamlLoader::load_from_str(item_text)?;


// mod item_model;
// use crate::item_model::*;


struct FileContents{
    name: String,
    contents: String,
}


fn load_item_files_into_buffers(item_buffers: &mut Vec<FileContents>, path: &Path) -> io::Result<()> {
    let files = fs::read_dir(path)?;
    for file_result in files{
        let file = file_result?;

        // get the file name, as a normal (utf8) string
        let file_name = file.file_name().to_string_lossy().to_string();

        // read open and read file as a string
        let contents = fs::read_to_string(file.path())?;
       
        item_buffers.push(FileContents{
            name:file_name,
            contents: contents
        });
    }
    Ok(())
}

fn validate_yaml_key(yaml: &Yaml, key: &str) {
    let item_name_element = &yaml[key];

    let mut terminal = term::stdout().unwrap();

    if item_name_element.is_badvalue(){
        terminal.fg(term::color::BRIGHT_RED).unwrap();
        println!("'{}' key is missing", key);
        terminal.reset().unwrap();
    }

}

fn validate_item_contents(contents:&str) -> Result<(),ScanError> {
    let docs = YamlLoader::load_from_str(contents)?;
    // YAML files can actually contain multiple files inside, we want the first one
    let doc = &docs[0];
    validate_yaml_key(&doc, "Name");
    validate_yaml_key(&doc, "Item Number");
    validate_yaml_key(&doc, "Level");

    Ok(())
}

fn main() {
    let mut item_buffers :Vec<FileContents> = Vec::new();
    let path_for_item_files = Path::new("../../../data/items/");
    load_item_files_into_buffers(&mut item_buffers, &path_for_item_files).unwrap();

    for file in item_buffers{
        println!("Validating {}", file.name);
    
        let result = validate_item_contents(&file.contents);
        if result.is_err(){
            println!("unable to validate {}", file.name)
        }


    }




}
