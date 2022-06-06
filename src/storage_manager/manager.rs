use crate::config::cfg::Cfg;
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom::Start, Write};
use std::path::Path;

trait WriteWithOffset {
    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<(), std::io::Error>;
}
impl WriteWithOffset for File {
    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<(), std::io::Error> {
        self.seek(Start(offset))?;
        self.write_all(buf)
    }
}

pub fn save_piece(
    name: String,
    piece: &[u8],
    piece_offset: u64,
    config: Cfg,
) -> Result<(), std::io::Error> {
    let save_directory = config.download_directory;
    if !Path::new(&save_directory).exists() {
        fs::create_dir_all(save_directory.clone())?;
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(save_directory + "/" + &name)?;

    file.write_all_at(piece, piece_offset)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{read, remove_file, File};
    use std::io::Write;
    use std::path::Path;

    use super::*;

    const CONFIG_PATH: &str = "config.cfg";

    #[test]
    fn save_file_creates_file_if_it_does_not_exist() {
        let file_name = "test_file_01.txt".to_string();
        let path = format!("./downloads/{}", &file_name);

        assert!(!Path::new(&path).exists());
        assert!(save_piece(
            file_name,
            &[0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8],
            0,
            Cfg::new(CONFIG_PATH).unwrap()
        )
        .is_ok());
        assert!(Path::new(&path).exists());
        remove_file(path).unwrap();
    }

    #[test]
    fn write_in_nonexistent_file() {
        let file_name = "test_file_02.txt".to_string();
        let path = format!("./downloads/{}", &file_name);

        create_downloads_dir_if_necessary();

        assert!(!Path::new(&path).exists());

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece(
            file_name.to_string(),
            &content_to_write,
            0,
            Cfg::new(CONFIG_PATH).unwrap()
        )
        .is_ok());
        assert!(Path::new(&path).exists());

        read_file_and_assert_its_content_equals_expected_content(content_to_write, &path);

        remove_file(path).unwrap();
    }

    #[test]
    fn write_in_existing_file() {
        let file_name = "test_file_03.txt".to_string();
        let path = format!("./downloads/{}", &file_name);

        create_downloads_dir_if_necessary();

        File::create(&path).unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece(
            file_name.to_string(),
            &content_to_write,
            0,
            Cfg::new(CONFIG_PATH).unwrap()
        )
        .is_ok());

        read_file_and_assert_its_content_equals_expected_content(content_to_write, &path);

        remove_file(path).unwrap();
    }

    #[test]
    fn write_at_the_end_of_existing_file_that_already_has_contents() {
        let file_name = "test_file_04.txt".to_string();
        let path = format!("./downloads/{}", &file_name);

        create_downloads_dir_if_necessary();

        let mut file = File::create(&path).unwrap();
        let previous_content = vec![0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8];
        file.write_all(&previous_content).unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece(
            file_name.to_string(),
            &content_to_write,
            5,
            Cfg::new(CONFIG_PATH).unwrap()
        )
        .is_ok());

        read_file_and_assert_its_content_equals_expected_content(
            vec![
                0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8, 0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8,
            ],
            &path,
        );

        remove_file(path).unwrap();
    }

    #[test]
    fn write_between_pieces_of_existing_file_that_already_has_contents() {
        let file_name = "test_file_05.txt".to_string();
        let path = format!("./downloads/{}", &file_name);

        create_downloads_dir_if_necessary();

        let mut file = File::create(&path).unwrap();
        let first_piece = vec![0x56u8, 0x69u8, 0x76u8, 0x61u8];
        let second_piece = vec![0x20, 0x50u8, 0x65u8];
        let third_piece = vec![0x72u8, 0xF3u8, 0x6Eu8];

        file.write_all(&first_piece).unwrap();
        file.write_all_at(&third_piece, 7).unwrap();

        assert!(save_piece(
            file_name.to_string(),
            &second_piece,
            4,
            Cfg::new(CONFIG_PATH).unwrap()
        )
        .is_ok());

        read_file_and_assert_its_content_equals_expected_content(
            vec![
                0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8, 0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8,
            ],
            &path,
        );

        remove_file(path).unwrap();
    }

    fn read_file_and_assert_its_content_equals_expected_content(
        expected_content: Vec<u8>,
        file_name: &str,
    ) {
        let content = read(file_name).unwrap();
        assert_eq!(content, expected_content);
    }

    fn create_downloads_dir_if_necessary() {
        if !Path::new("./downloads").exists() {
            fs::create_dir_all("./downloads").unwrap();
        }
    }
}
