

#[derive(Debug, PartialEq)]
pub enum Element{
    Fire,
    Ice,
    Lightning,
    Wind
}

#[derive(Debug, PartialEq)]
pub struct ElementValue{
    pub element :Element,
    pub element_value : u8
}
#[derive(Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub item_number: u8,
    pub level: u8,
    pub category_list: Vec<String>,
    pub elements: Vec<ElementValue>,
}


