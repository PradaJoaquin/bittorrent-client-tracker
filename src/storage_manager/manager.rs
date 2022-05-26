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

        let content = read("test_file.txt").unwrap();
        assert_eq!(content, content_to_write);
        remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn write_in_existing_file() {
        File::create("test_file.txt").unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece("test_file.txt".to_string(), &content_to_write, 0).is_ok());

        let content = read("test_file.txt").unwrap();
        assert_eq!(content, content_to_write);
        remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn write_in_existing_file_that_already_has_contents() {
        let mut file = File::create("test_file.txt").unwrap();
        let previous_content = vec![0x56, 0x69, 0x76, 0x61, 0x20];
        file.write_all(&previous_content).unwrap();

        let content_to_write = vec![0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8];
        assert!(save_piece("test_file.txt".to_string(), &content_to_write, 5).is_ok());

        let content = read("test_file.txt").unwrap();
        let expected_content = vec![
            0x56, 0x69, 0x76, 0x61, 0x20, 0x50u8, 0x65u8, 0x72u8, 0xF3u8, 0x6Eu8,
        ];
        assert_eq!(content, expected_content);
        remove_file("test_file.txt").unwrap();
    }
}
