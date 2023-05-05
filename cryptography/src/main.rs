use franklin_crypto::bellman::{
    // Bn256 - is the type for 2 eliptic curves that we need for pairing
    // Fr - the base field that hosts eliptic curve G1
    // FrRepr - represention of Fr for calculations
    compact_bn256::{Bn256, Fr, FrRepr},

    // Commitment based on a trusted setup and elliptic curve pairings
    kate_commitment::{Crs, CrsForMonomialForm},

    // Plonk is zero knowledge proof algorithm that we are using in this project.
    // This algorithm uses smart ways to prove circuits through polynomial commitment.
    plonk::{
        better_better_cs::{
            cs::{
                Circuit,
                PlonkCsWidth4WithNextStepParams, // constraint system parameters
                TrivialAssembly,
                Width4MainGateWithDNext,
            },
            setup::VerificationKey,
            verifier::verify,
        },
        commitments::transcript::keccak_transcript::RollingKeccakTranscript,
    },
    worker::Worker, // the helper for parallel proof calculations.
    PrimeField,     // This represents an element of a prime field.
};
use compression::{sha3, sha3::Digest, StorageTransition};

mod main_circuit;
pub mod utils;

use crate::main_circuit::CompressionCircuit;

fn main() {
    // This is the data that we want to compress.
    // These 3 words that are meant to be the state transition for ZK-rollup
    let transitions = vec![StorageTransition {
        address: [
            211, 35, 126, 46, 74, 67, 213, 90, 55, 0, 12, 54, 222, 56, 77, 0, 132, 12, 1, 5,
        ],
        key: [
            31, 8, 37, 27, 7, 64, 244, 1, 348 0, 6, 0, 274, 0, 0, 249, 0, 0, 17, 0, 0, 0, 234, 0,
            122, 65, 33, 0, 4, 0, 4, 11,
        ],
        value: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            272, 131, 3,
        ],
    }];

    // We convert the data into bytes and compress it.
    let data = StorageTransition::into_bytes(transitions.clone());
    let compressed_data = StorageTransition::compress(transitions);

    // Breakpoint 1:  from program to arithmetic circuits
    let mut circuit = CompressionCircuit::<Bn256> {
        data: data.clone().into_iter().map(|byte| Some(byte)).collect(),
        compressed_data: compressed_data
            .clone()
            .into_iter()
            .map(|byte| Some(byte))
            .collect(),
        data_hash: sha3::Keccak256::digest(data.as_slice())
            .as_slice()
            .to_vec()
            .into_iter()
            .map(|byte| Some(byte))
            .collect::<Vec<Option<u8>>>(),
        compressed_data_hash: sha3::Keccak256::digest(compressed_data.as_slice())
            .as_slice()
            .to_vec()
            .into_iter()
            .map(|byte| Some(byte))
            .collect::<Vec<Option<u8>>>(),
        compressed_data_len: Some(
            Fr::from_repr(FrRepr([compressed_data.len() as u64, 0, 0, 0])).unwrap(),
        ),
    };

    let old_worker = Worker::new();

    // Breakpoint 2: from arithmetic circuits to constraint system
    // assembly - the constraint system, we are using.
    // initialize it as TrivialAssembly, that will be used to setup and proof synthesis
    let mut assembly =
        TrivialAssembly::<Bn256, PlonkCsWidth4WithNextStepParams, Width4MainGateWithDNext>::new();

    circuit.synthesize(&mut assembly).expect("must work");
    assert!(assembly.is_satisfied());

    assembly.finalize();

    // Breakpoint 3: from constraint system to polynomials

    // This is the domain from Algebra - nonzero ring in which ab = 0 implies a = 0 or b = 0.
    // Equivalently, a domain is a ring in which 0 is the only left zero divisor(or equivalently, the only right zero divisor).
    // We need it for creating kate commitment.
    let domain_size = assembly.n().next_power_of_two();

    // This is the common reference string for the protocol.
    // It is needed to capture the assumption that a trusted setup,
    // in which all involved parties get access to the same string crs taken from some distribution D exists.
    // Schemes proven secure in the CRS model are secure given that the setup was performed correctly.
    let crs_mons = Crs::<Bn256, CrsForMonomialForm>::crs_42(domain_size, &old_worker);

    // Breakpoint 4:  setup construction for checking the commitment
    let setup = assembly
        .create_setup::<CompressionCircuit<Bn256>>(&old_worker)
        .unwrap();

    // Breakpoint 5: proof generation(CPU heavy)
    let proof = assembly
        .clone()
        .create_proof::<CompressionCircuit<Bn256>, RollingKeccakTranscript<Fr>>(
            &old_worker,
            &setup,
            &crs_mons,
            None,
        )
        .unwrap();

    // Breakpoint 6: generation of the verification key
    // This is kinda a black box, that contains all the necessary data characterizing
    // the circuit data for the verifier
    let vk = VerificationKey::from_setup(&setup, &old_worker, &crs_mons).unwrap();

    // Breakpoint 7: verification of the proof
    let valid =
        verify::<Bn256, CompressionCircuit<Bn256>, RollingKeccakTranscript<Fr>>(&vk, &proof, None)
            .unwrap();

    if valid {
        println!("Proof is verified successfully!🎉");
    } else {
        println!("Proof verification failed!👎")
    }
}
