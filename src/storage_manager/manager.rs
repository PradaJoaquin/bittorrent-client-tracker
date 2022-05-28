use std::fs::OpenOptions;
use std::os::unix::fs::FileExt;

pub fn save_piece(name: String, piece: &[u8], piece_offset: u64) -> Result<(), std::io::Error> {
    // Por ahora guardamos en el mismo directorio, usando el mismo nombre que tenga el archivo en el torrent.
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(name)?;

    file.write_all_at(piece, piece_offset)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{read, remove_file, File};
    use std::io::Write;
    use std::path::Path;

    use super::*;

    #[test]
    fn save_file_creates_file_if_it_does_not_exist() {
        assert!(!Path::new("test_file.txt").exists());
        assert!(save_piece(
            "test_file.txt".to_string(),
            &[0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8],
            0
        )
        .is_ok());
        assert!(Path::new("test_file.txt").exists());
        remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn write_in_nonexistent_file() {
        assert!(!Path::new("test_file.txt").exists());

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece("test_file.txt".to_string(), &content_to_write, 0).is_ok());
        assert!(Path::new("test_file.txt").exists());

        read_file_and_assert_its_content_equals_expected_content(content_to_write, "test_file.txt");

        remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn write_in_existing_file() {
        File::create("test_file.txt").unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece("test_file.txt".to_string(), &content_to_write, 0).is_ok());

        read_file_and_assert_its_content_equals_expected_content(content_to_write, "test_file.txt");

        remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn write_at_the_end_of_existing_file_that_already_has_contents() {
        let mut file = File::create("test_file.txt").unwrap();
        let previous_content = vec![0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8];
        file.write_all(&previous_content).unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece("test_file.txt".to_string(), &content_to_write, 5).is_ok());

        read_file_and_assert_its_content_equals_expected_content(
            vec![
                0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8, 0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8,
            ],
            "test_file.txt",
        );

        remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn write_between_pieces_of_existing_file_that_already_has_contents() {
        let mut file = File::create("test_file.txt").unwrap();
        let first_piece = vec![0x56u8, 0x69u8, 0x76u8, 0x61u8];
        let second_piece = vec![0x20, 0x50u8, 0x65u8];
        let third_piece = vec![0x72u8, 0xF3u8, 0x6Eu8];

        file.write_all(&first_piece).unwrap();
        file.write_all_at(&third_piece, 7).unwrap();

        assert!(save_piece("test_file.txt".to_string(), &second_piece, 4).is_ok());

        read_file_and_assert_its_content_equals_expected_content(
            vec![
                0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8, 0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8,
            ],
            "test_file.txt",
        );

        remove_file("test_file.txt").unwrap();
    }

    fn read_file_and_assert_its_content_equals_expected_content(
        expected_content: Vec<u8>,
        file_name: &str,
    ) {
        let content = read(file_name).unwrap();
        assert_eq!(content, expected_content);
    }
}
