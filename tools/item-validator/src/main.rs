use std::path::Path;
use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::str;

mod item_model;

pub use crate::item_model::*;


fn get_item_files(path: &Path,file_buffers: &mut Vec<Vec<u8>>) -> io::Result<()> {
    let files = fs::read_dir(path)?;
    for file_result in files{
        let file = file_result?;
        let file_name = file.file_name();
        println!("opening {:?}", file_name);
        let mut file = File::open(file.path())?;
        
        let mut buffer = Vec::new();
        // read the whole file
        file.read_to_end(&mut buffer)?;
        let item_text = str::from_utf8(&buffer).expect("file is not valid utf8");

        println!("{}",item_text);
        println!("   -----     ");
        // done using specific buffer, store in file buffers for reference
        file_buffers.push(buffer)
    }
    Ok(())
}

fn main() {
    let mut file_buffers :Vec<Vec<u8>> = Vec::new();
    let path_for_item_files = Path::new("../../../data/items/");

    get_item_files(&path_for_item_files, &mut file_buffers).unwrap();
    

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
