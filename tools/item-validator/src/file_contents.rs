use std::path::Path;
use std::{fs, io};

pub struct FileContents {
    pub name: String,
    pub contents: String,
}

pub fn load_lists_file(file_path: &str) -> io::Result<String> {
    let file_path = Path::new(file_path);
    let contents = fs::read_to_string(file_path)?;
    Ok(contents)
}

pub fn load_item_files(item_contents: &mut Vec<FileContents>, path: &str) -> io::Result<()> {
    let path = Path::new(path);
    let files = fs::read_dir(path)?;
    for file_result in files {
        let file = file_result?;

        // get the file name, as a normal (utf8) string
        let file_name = file.file_name().to_string_lossy().to_string();

        // read open and read file as a string
        let contents = fs::read_to_string(file.path())?;

        item_contents.push(FileContents {
            name: file_name,
            contents,
        });
    }
    Ok(())
}
