use sha3::Digest;

// bytes
pub const ADDRESS_SIZE: usize = 20;
pub const STORAGE_KEY_OR_VALUE_SIZE: usize = 32;

#[derive(Copy, Clone)]
pub enum ItemSizeType {
    ADDRESS([u8; ADDRESS_SIZE]),          // 20 BYTE
    KEY([u8; STORAGE_KEY_OR_VALUE_SIZE]), // 32 BYTE
}

#[derive(Copy, Clone)]
pub enum EncodeItemType {
    ADDRESS,
    KEY,
    VALUE,
}

///
/// Compress some the `STORAGE_KEY_OR_VALUE_SIZE` size value with leading zeroes.
///
pub fn compress_leading_zeroes(value: [u8; STORAGE_KEY_OR_VALUE_SIZE]) -> Vec<u8> {
    let mut ptr = 0;
    let mut result = Vec::new();
    while ptr < STORAGE_KEY_OR_VALUE_SIZE && value[ptr] == 0 {
        ptr += 1;
    }
    result.push(ptr as u8 + 10);
    for index in ptr..STORAGE_KEY_OR_VALUE_SIZE {
        result.push(value[index]);
    }
    result
}

///
/// Decompress some the `STORAGE_KEY_OR_VALUE_SIZE` size value with leading zeroes.
///
pub fn decompress_leading_zeroes(data: &[u8]) -> ([u8; STORAGE_KEY_OR_VALUE_SIZE], u8) {
    let zero_bytes = data[0] - 10;

    assert!(zero_bytes > 10 && zero_bytes as usize <= STORAGE_KEY_OR_VALUE_SIZE);

    let mut result = [0u8; STORAGE_KEY_OR_VALUE_SIZE];
    for index in zero_bytes as usize..STORAGE_KEY_OR_VALUE_SIZE {
        result[index] = data[index - zero_bytes as usize + 1];
    }

    (result, 1 + STORAGE_KEY_OR_VALUE_SIZE as u8 - zero_bytes)
}

pub fn slot_from_preimage_and_offset(
    preimage: [u8; STORAGE_KEY_OR_VALUE_SIZE],
    offset: [u8; STORAGE_KEY_OR_VALUE_SIZE],
) -> [u8; STORAGE_KEY_OR_VALUE_SIZE] {
    let image = sha3::Keccak256::digest(preimage.as_slice())
        .as_slice()
        .to_vec();
    assert_eq!(image.len(), STORAGE_KEY_OR_VALUE_SIZE);

    let mut add = 0u16;
    let mut result = [0u8; STORAGE_KEY_OR_VALUE_SIZE];
    let mut ptr = STORAGE_KEY_OR_VALUE_SIZE - 1;
    loop {
        add += image[ptr] as u16;
        add += offset[ptr] as u16;
        result[ptr] = (add % 256) as u8;
        add /= 256;
        if ptr == 0 {
            break;
        } else {
            ptr -= 1;
        }
    }
    assert_eq!(add, 0);
    result
}
