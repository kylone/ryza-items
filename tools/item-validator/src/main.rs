use std::path::Path;
use std::{fs, io};

// extern crate yaml_rust;
// use yaml_rust::YamlLoader;

// code for str -> Yaml
//let item_doc = YamlLoader::load_from_str(item_text)?;


// mod item_model;
// use crate::item_model::*;

// get text from buffers

struct FileContents{
    name: String,
    contents: String,
}


fn load_item_files_into_buffers(item_buffers: &mut Vec<FileContents>, path: &Path) -> io::Result<()> {
    let files = fs::read_dir(path)?;
    for file_result in files{
        let file = file_result?;
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

fn main() {
    let mut item_buffers :Vec<FileContents> = Vec::new();
    let path_for_item_files = Path::new("../../../data/items/");
    load_item_files_into_buffers(&mut item_buffers, &path_for_item_files).unwrap();

    for file in item_buffers{
        println!(" ---> {}", file.name);
        println!("{}",file.contents);
        println!("   -----     ");
    }

    // let mut buffer = Vec::new();
    // // read the whole file
    // file.read_to_end(&mut buffer).expect("unable to read from file");

    // let item_text = str::from_utf8(&buffer).expect("file is not valid utf8");
    // println!("{}",item_text);

    // let x = Item{
    //     name: "test".to_string(),
    //     item_number: 1,
    //     level: 1,
    //     category_list: vec!["".to_string()],
    //     elements : vec![ElementValue{element:Element::Fire, element_value:1}]
    // };

}
