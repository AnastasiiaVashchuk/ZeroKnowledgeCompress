pub use franklin_crypto::{
    bellman::{
        kate_commitment::{Crs, CrsForMonomialForm},
        plonk::{
            better_better_cs::{
                cs::{
                    ArithmeticTerm, Assembly, Circuit, ConstraintSystem, Gate, GateInternal,
                    LookupTableApplication, MainGateTerm,
                    PlonkCsWidth4WithNextStepAndCustomGatesParams, PolyIdentifier, Setup,
                    TrivialAssembly, Width4MainGateWithDNext,
                },
                gates::selector_optimized_with_d_next::SelectorOptimizedWidth4MainGateWithDNext,
                proof::Proof,
                setup::VerificationKey,
                verifier,
            },
            commitments::transcript::{keccak_transcript::RollingKeccakTranscript, Transcript},
        },
        worker::Worker,
        Engine, Field, PrimeField, ScalarEngine, SynthesisError,
    },
    plonk::circuit::{
        allocated_num::{AllocatedNum, Num},
        boolean::{AllocatedBit, Boolean},
        byte::Byte,
        custom_rescue_gate::Rescue5CustomGate,
    },
};

use crate::utils::*;

// The main circuit structure.
// PlonK circuit is composed by gates, which supports only multiplication and addition.
// The circuit needs proving validity of input/output of the gates, as well as wire’s relationship.
pub struct CompressionCircuit<E: Engine> {
    pub data: Vec<Option<u8>>,
    pub compressed_data: Vec<Option<u8>>,
    pub data_hash: Vec<Option<u8>>,
    pub compressed_data_hash: Vec<Option<u8>>,
    pub compressed_data_len: Option<E::Fr>,
}

impl<E: Engine> Circuit<E> for CompressionCircuit<E> {
    type MainGate = Width4MainGateWithDNext;

    fn synthesize<CS: ConstraintSystem<E>>(&self, cs: &mut CS) -> Result<(), SynthesisError> {
        let columns = vec![
            PolyIdentifier::VariablesPolynomial(0),
            PolyIdentifier::VariablesPolynomial(1),
            PolyIdentifier::VariablesPolynomial(2),
        ];

        let range_table = LookupTableApplication::new_range_table_of_width_3(8, columns.clone())?;
        let range_table_name = range_table.functional_name();
        cs.add_table(range_table)?;

        let _compressed_data_hash_bytes = allocate_and_prove_bytes(
            &self.compressed_data_hash,
            32,
            cs,
            range_table_name.as_str(),
            true,
        )?;

        let _data_hash =
            allocate_and_prove_bytes(&self.data_hash, 32, cs, range_table_name.as_str(), true)?;

        let compressed_data_bytes = allocate_and_prove_bytes(
            &self.compressed_data,
            MAX_COMPRESSED_DATA_SIZE,
            cs,
            range_table_name.as_str(),
            false,
        )?;
        let data_bytes = allocate_and_prove_bytes(
            &self.data,
            MAX_UNCOMPRESSED_DATA_SIZE,
            cs,
            range_table_name.as_str(),
            false,
        )?;

        // TODO: prove hashes correctness

        let _compressed_data_len = Num::alloc(cs, self.compressed_data_len)?;

        let mut ptr = Num::zero();
        let zero = Num::zero();
        let one = Num::one();

        for word in 0..MAX_WORDS {
            let compressed_word = get_word_from_bytes(cs, &compressed_data_bytes, &ptr)?;
            assert_eq!(compressed_word.len(), 33);
            let (uncompressed_pos, size) =
                crate::utils::get_word_position_in_sequence_of_data(word);
            let mut ok = Boolean::alloc(cs, Some(false))?;
            if size == 20 {
                let mut is1 = Num::equals(cs, &compressed_word[0].inner, &one)?;
                for i in 0..20 {
                    let eq = Num::equals(
                        cs,
                        &data_bytes[uncompressed_pos + i].inner,
                        &compressed_word[i + 1].inner,
                    )?;
                    is1 = Boolean::and(cs, &is1, &eq)?;
                }
                ok = is1;
                let _21 = Num::Constant(E::Fr::from_str("21").unwrap());
                ptr = ptr.add(cs, &_21)?;
            } else {
                let mut is0 = Num::equals(cs, &compressed_word[0].inner, &zero)?;
                for i in 0..32 {
                    let eq = Num::equals(
                        cs,
                        &data_bytes[uncompressed_pos + i].inner,
                        &compressed_word[i + 1].inner,
                    )?;
                    is0 = Boolean::and(cs, &is0, &eq)?;
                }
                ok = Boolean::or(cs, &ok, &is0)?;

                for i in 11..=42 {
                    let i_num = Num::alloc(cs, Some(E::Fr::from_str(&format!("{}", i)).unwrap()))?;
                    let mut is = Num::equals(cs, &compressed_word[0].inner, &i_num)?;
                    for index in 0..32 {
                        let eq;
                        if index < i - 10 {
                            eq = Num::equals(
                                cs,
                                &data_bytes[uncompressed_pos + index].inner,
                                &zero,
                            )?;
                        } else {
                            eq = Num::equals(
                                cs,
                                &data_bytes[uncompressed_pos + index].inner,
                                &compressed_word[index - (i - 10) + 1].inner,
                            )?;
                        }
                        is = Boolean::and(cs, &is, &eq)?;
                    }
                    ok = Boolean::or(cs, &ok, &is)?;
                }
                let _33 = Num::alloc(cs, Some(E::Fr::from_str("33").unwrap()))?;
                ptr = ptr.add(cs, &_33)?;
            }
            // TODO: Add 2 byte type
            let true_bool = Boolean::alloc(cs, Some(true))?;
            Boolean::enforce_equal(cs, &ok, &true_bool)?;
        }
        Ok(())
    }
}

// Allocate byte array and prove tha values of bytes.
// circuit arithmetic
fn allocate_and_prove_bytes<E: Engine, CS: ConstraintSystem<E>>(
    bytes: &Vec<Option<u8>>,
    len: usize,
    cs: &mut CS,
    range_table_name: &str,
    alloc_as_inputs: bool,
) -> Result<Vec<Byte<E>>, SynthesisError> {
    let mut result = Vec::with_capacity(bytes.len());

    for i in 0..len {
        let byte = bytes.get(i).unwrap_or(&Some(0));
        let inner = if alloc_as_inputs {
            Num::Variable(AllocatedNum::alloc_input(cs, || {
                Ok(E::Fr::from_str(&format!("{}", byte.unwrap())).unwrap())
            })?)
        } else {
            Num::alloc(
                cs,
                Some(E::Fr::from_str(&format!("{}", byte.unwrap())).unwrap()),
            )?
        };

        let table = cs.get_table(range_table_name)?;
        let num_keys_and_values = table.width();

        let var_zero = cs.get_explicit_zero()?;
        let dummy = CS::get_dummy_variable();

        let inner_var = inner.get_variable().get_variable();
        let vars = [inner_var, var_zero.clone(), var_zero.clone(), dummy];

        cs.begin_gates_batch_for_step()?;

        cs.allocate_variables_without_gate(&vars, &[])?;

        cs.apply_single_lookup_gate(&vars[..num_keys_and_values], table)?;
        cs.end_gates_batch_for_step()?;

        result.push(Byte { inner });
    }

    Ok(result)
}

// Read word by dynamic index from bytes array(O(n)).
// circuit arithmetic
fn get_word_from_bytes<E: Engine, CS: ConstraintSystem<E>>(
    cs: &mut CS,
    bytes: &Vec<Byte<E>>,
    pos: &Num<E>,
) -> Result<Vec<Byte<E>>, SynthesisError> {
    let mut result = Vec::with_capacity(33);
    for index in 0..33 {
        let zero = Num::zero();
        let mut index_num = Num::Constant(E::Fr::from_str(&format!("{}", index)).unwrap());
        index_num = index_num.add(cs, pos)?;
        let mut res_byte = Byte::from_num(cs, zero)?;
        for (i, byte) in bytes.iter().enumerate() {
            let i_num = Num::Constant(E::Fr::from_str(&format!("{}", i)).unwrap());
            let flag = Num::equals(cs, &i_num, &index_num)?;
            let mut flag = Num::from_boolean_is(flag);
            flag = flag.mul(cs, &byte.inner)?;
            res_byte.inner = res_byte.inner.add(cs, &flag)?;
        }
        result.push(res_byte);
    }

    Ok(result)
}
