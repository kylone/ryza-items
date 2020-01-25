use std::fs::File;
use std::io::Read;
use std::str;

mod item_model;

pub use crate::item_model::*;

fn main() {

    let mut file = File::open("../../../data/items/1-explosive-uni.yaml").expect("can't find file");
    

    let mut buffer = Vec::new();
    // read the whole file
    file.read_to_end(&mut buffer).expect("unable to read from file");

    let item_text = str::from_utf8(&buffer).expect("file is not valid utf8");
    // let x = Item{
    //     name: "test".to_string(),
    //     item_number: 1,
    //     level: 1,
    //     category_list: vec!["".to_string()],
    //     elements : vec![ElementValue{element:Element::Fire, element_value:1}]
    // };

    println!("{}",item_text);
}
