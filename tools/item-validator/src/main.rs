
mod item_model;

pub use crate::item_model::*;

fn main() {

    let file_path = "../../../data/items";

    let x = Item{
        name: "test".to_string(),
        item_number: 1,
        level: 1,
        category_list: vec!["".to_string()],
        elements : vec![ElementValue{element:Element::Fire, element_value:1}]
    };

    println!("{:#?}",x);
}
