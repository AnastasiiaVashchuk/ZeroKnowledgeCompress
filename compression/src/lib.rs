pub use sha3;

pub mod utils;
use crate::utils::{
    compress_leading_zeroes, decompress_leading_zeroes,
    EncodeItemType, ItemSizeType, ADDRESS_SIZE, STORAGE_KEY_OR_VALUE_SIZE,
};

#[cfg(test)]
mod tests;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct StorageTransition {
    /// account address
    pub address: [u8; ADDRESS_SIZE],
    /// storage key
    pub key: [u8; STORAGE_KEY_OR_VALUE_SIZE],
    /// new value
    pub value: [u8; STORAGE_KEY_OR_VALUE_SIZE],
}

impl StorageTransition {
    pub fn into_bytes(transitions: Vec<Self>) -> Vec<u8> {
        let mut result =
            Vec::with_capacity(transitions.len() * (ADDRESS_SIZE + 2 * STORAGE_KEY_OR_VALUE_SIZE));
        for transition in transitions {
            result.extend(transition.address);
            result.extend(transition.key);
            result.extend(transition.value);
        }
        result
    }

    pub fn compress(transitions: Vec<Self>) -> Vec<u8> {
        let mut result = Vec::new();
        for transition in transitions {
            // address
            result.push(1);
            result.extend(transition.address);

            // key
            let mut key = compress_leading_zeroes(transition.key);

            if key.len() >= STORAGE_KEY_OR_VALUE_SIZE + 1 {
                key = vec![0];
                key.extend(transition.key);
            }
            result.extend(key);

            // value
            let mut value = utils::compress_leading_zeroes(transition.value);
            if value.len() >= STORAGE_KEY_OR_VALUE_SIZE + 1 {
                value = vec![0];
                value.extend(transition.value);
            }

            result.extend(value);
        }

        result
    }

    pub fn decompress(data: Vec<u8>) -> Vec<StorageTransition> {
        let mut result: Vec<StorageTransition> = Vec::new();

        let mut ptr = 0;
        let mut expected_field = EncodeItemType::ADDRESS;
        result.push(Self::default());

        while ptr < data.len() {
            let value = if data[ptr] == 0 {
                let mut value = [0; STORAGE_KEY_OR_VALUE_SIZE];
                for index in ptr + 1..=ptr + STORAGE_KEY_OR_VALUE_SIZE {
                    value[index - ptr - 1] = data[index];
                }
                ptr += STORAGE_KEY_OR_VALUE_SIZE + 1;
                ItemSizeType::KEY(value)
            } else if data[ptr] == 1 {
                let mut value = [0; ADDRESS_SIZE];
                for index in ptr + 1..=ptr + ADDRESS_SIZE {
                    value[index - ptr - 1] = data[index];
                }
                ptr += ADDRESS_SIZE + 1;
                ItemSizeType::ADDRESS(value)
            } else if data[ptr] == 2 {
                let (preimage, offset) = utils::decompress_leading_zeroes(&data[ptr + 1..]);
                ptr += 1 + offset as usize;
                let (image_offset, offset) = utils::decompress_leading_zeroes(&data[ptr..]);
                ptr += offset as usize;
                ItemSizeType::KEY(utils::slot_from_preimage_and_offset(preimage, image_offset))
            } else {
                let (value, offset) = decompress_leading_zeroes(&data[ptr..]);
                ptr += offset as usize;
                ItemSizeType::KEY(value)
            };

            match (expected_field, value) {
                (EncodeItemType::ADDRESS, ItemSizeType::ADDRESS(address)) => {
                    result.last_mut().unwrap().address = address;
                    expected_field = EncodeItemType::KEY;
                }
                (EncodeItemType::KEY, ItemSizeType::KEY(value)) => {
                    result.last_mut().unwrap().key = value;
                    expected_field = EncodeItemType::VALUE;
                }
                (EncodeItemType::VALUE, ItemSizeType::KEY(value)) => {
                    result.last_mut().unwrap().value = value;
                    expected_field = EncodeItemType::ADDRESS;
                    result.push(Self::default());
                }
                _ => panic!("Invalid data"),
            }
        }

        result.pop().unwrap();
        result
    }
}
