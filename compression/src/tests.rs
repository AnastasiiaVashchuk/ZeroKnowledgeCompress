use super::*;

#[test]
fn test_algorithm_correctness() {
    let transitions = vec![StorageTransition {
        address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
        key: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2,
        ],
        value: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2,
        ],
    }];
    let compressed = StorageTransition::compress(transitions.clone());
    assert_eq!(transitions, StorageTransition::decompress(compressed));
}

#[test]
fn test_effectiency_simple_contract() {
    let transitions = vec![StorageTransition {
        address: [
            222, 34, 125, 45, 64, 67, 218, 92, 55, 0, 12, 54, 223, 56, 77, 0, 142, 12, 3, 3,
        ],
        key: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 10,
        ],
        value: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 56, 111,
        ],
    }];
    let compressed = StorageTransition::compress(transitions.clone());
    let start_len = StorageTransition::into_bytes(transitions.clone()).len() as f64;
    println!("Plain data len is bytes: {start_len}");

    let optimise = compressed.len() as f64;
    println!("Compressed data len is bytes: {optimise}");
    
    println!(
        "Optimized {:.2} % for simple contract",
        (start_len - optimise) / start_len * 100.0
    );
    assert_eq!(transitions, StorageTransition::decompress(compressed));
}

#[test]
fn effeciency_erc20_contract() {
    let transitions = vec![
        StorageTransition {
            address: [
                221, 31, 123, 47, 33, 67, 233, 90, 55, 3, 11, 84, 222, 56, 77, 0, 132, 12, 1, 5,
            ],
            key: [
                34, 1, 123, 21, 44, 65, 78, 66, 34, 0, 0, 0, 234, 0, 0, 243, 0, 0, 22, 0, 0, 0,
                234, 0, 65, 0, 0, 0, 65, 0, 4, 42,
            ],
            value: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                21, 34, 46, 153,
            ],
        },
        StorageTransition {
            address: [
                221, 31, 123, 46, 34, 67, 213, 90, 55, 0, 12, 54, 222, 56, 77, 0, 132, 12, 1, 5,
            ],
            key: [
                31, 8, 32, 23, 2, 65, 222, 1, 34, 0, 6, 0, 234, 0, 0, 243, 0, 0, 22, 0, 0, 0,134,
                0, 122, 65, 33, 0, 4, 0, 4, 11,
            ],
            value: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                7, 212, 161, 5,
            ],
        },
    ];
    let compressed = StorageTransition::compress(transitions.clone());
    let start_len = StorageTransition::into_bytes(transitions.clone()).len() as f64;
    println!("Plain data len is bytes: {start_len}");

    let optimise = compressed.len() as f64;
    println!("Compressed data len is bytes: {optimise}");
    
    println!(
        "Optimized {:.2} % for ERC20",
        (start_len - optimise) / start_len * 100.0
    );
    assert_eq!(transitions, StorageTransition::decompress(compressed));
}
