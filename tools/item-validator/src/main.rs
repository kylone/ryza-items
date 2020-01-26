use std::path::Path;
use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::str;

// extern crate yaml_rust;
// use yaml_rust::YamlLoader;

// code for str -> Yaml
//let item_doc = YamlLoader::load_from_str(item_text)?;


// mod item_model;
// use crate::item_model::*;

// get text from buffers

fn load_item_files_into_buffers(item_buffers: &mut Vec<Vec<u8>>, path: &Path) -> io::Result<()> {
    let files = fs::read_dir(path)?;
    for file_result in files{
        let file = file_result?;
        let file_name = file.file_name();
        println!("opening {:?}", file_name);
        let mut file = File::open(file.path())?;
        
        let mut file_buffer = Vec::new();
        // read the whole file
        file.read_to_end(&mut file_buffer)?;
        // store in buffer vector
        item_buffers.push(file_buffer);
    }
    Ok(())
}

fn convert_item_buffers_into_text(item_buffers: &Vec<Vec<u8>>) -> Vec<&str>  {
    let mut item_texts : Vec<&str> = Vec::new();
    for buffer in item_buffers{
        let item_text = str::from_utf8(buffer).expect("file is not valid utf8");
        item_texts.push(item_text);
    }

    item_texts
}

fn main() {
    let mut item_buffers :Vec<Vec<u8>> = Vec::new();
    let path_for_item_files = Path::new("../../../data/items/");
    load_item_files_into_buffers(&mut item_buffers, &path_for_item_files).unwrap();

    let item_texts =convert_item_buffers_into_text(&item_buffers);

    for text in item_texts{
        println!("{}",text);
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
