pub use compression::utils::{ADDRESS_SIZE, STORAGE_KEY_OR_VALUE_SIZE};

pub const MAX_COMPRESSED_DATA_SIZE: usize = 132; // 33 * 3 + 33
pub const MAX_UNCOMPRESSED_DATA_SIZE: usize = 129; // 32 * 3 + 33
pub const MAX_WORDS: usize = 3;

pub fn get_word_position_in_sequence_of_data(mut index: usize) -> (usize, usize) {
    let k = index / 3;
    index %= 3;
    let mut offset = k * (ADDRESS_SIZE + STORAGE_KEY_OR_VALUE_SIZE + STORAGE_KEY_OR_VALUE_SIZE);
    let size;
    match index {
        0 => {
            size = 20;
        }
        1 => {
            offset += 20;
            size = 32;
        }
        2 => {
            offset += 20 + 32;
            size = 32
        }
        _ => unreachable!(),
    };
    (offset, size)
}
