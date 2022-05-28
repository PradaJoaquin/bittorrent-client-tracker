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
        let path = "test_file_01.txt";
        assert!(!Path::new(path).exists());
        assert!(save_piece(
            path.to_string(),
            &[0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8],
            0
        )
        .is_ok());
        assert!(Path::new(path).exists());
        remove_file(path).unwrap();
    }

    #[test]
    fn write_in_nonexistent_file() {
        let path = "test_file_02.txt";
        assert!(!Path::new(path).exists());

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece(path.to_string(), &content_to_write, 0).is_ok());
        assert!(Path::new(path).exists());

        read_file_and_assert_its_content_equals_expected_content(content_to_write, path);

        remove_file(path).unwrap();
    }

    #[test]
    fn write_in_existing_file() {
        let path = "test_file_03.txt";
        File::create(path).unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece(path.to_string(), &content_to_write, 0).is_ok());

        read_file_and_assert_its_content_equals_expected_content(content_to_write, path);

        remove_file(path).unwrap();
    }

    #[test]
    fn write_at_the_end_of_existing_file_that_already_has_contents() {
        let path = "test_file_04.txt";
        let mut file = File::create(path).unwrap();
        let previous_content = vec![0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8];
        file.write_all(&previous_content).unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece(path.to_string(), &content_to_write, 5).is_ok());

        read_file_and_assert_its_content_equals_expected_content(
            vec![
                0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8, 0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8,
            ],
            path,
        );

        remove_file(path).unwrap();
    }

    #[test]
    fn write_between_pieces_of_existing_file_that_already_has_contents() {
        let path = "test_file_05.txt";
        let mut file = File::create(path).unwrap();
        let first_piece = vec![0x56u8, 0x69u8, 0x76u8, 0x61u8];
        let second_piece = vec![0x20, 0x50u8, 0x65u8];
        let third_piece = vec![0x72u8, 0xF3u8, 0x6Eu8];

        file.write_all(&first_piece).unwrap();
        file.write_all_at(&third_piece, 7).unwrap();

        assert!(save_piece(path.to_string(), &second_piece, 4).is_ok());

        read_file_and_assert_its_content_equals_expected_content(
            vec![
                0x56u8, 0x69u8, 0x76u8, 0x61u8, 0x20u8, 0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8,
            ],
            path,
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
}
