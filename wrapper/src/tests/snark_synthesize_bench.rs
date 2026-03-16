use super::*;
use bellman::plonk::better_better_cs::cs::{
    PlonkCsWidth4WithNextStepAndCustomGatesParams, ProvingAssembly,
};
use bellman::plonk::better_better_cs::gates::selector_optimized_with_d_next::SelectorOptimizedWidth4MainGateWithDNext;

type PlonkProvingAssembly = ProvingAssembly<
    crate::Bn256,
    PlonkCsWidth4WithNextStepAndCustomGatesParams,
    SelectorOptimizedWidth4MainGateWithDNext,
>;

/// Benchmark: measures only the circuit synthesis (witness generation) part
/// of the SNARK wrapper, without any GPU/proving operations.
/// This is the CPU-bound bottleneck in gpu_snark_prove.
#[test]
fn snark_synthesize_benchmark() {
    use bellman::plonk::better_better_cs::cs::Circuit as SnarkCircuit;

    let compression_proof: crate::CompressionProof = deserialize_from_file(COMPRESSION_PROOF_PATH);
    let compression_vk: crate::CompressionVK = deserialize_from_file(COMPRESSION_VK_PATH);

    let fixed_parameters = compression_vk.fixed_parameters.clone();
    let wrapper_function = crate::SnarkWrapperFunction;
    let circuit = crate::SnarkWrapperCircuit {
        witness: Some(compression_proof),
        vk: compression_vk,
        fixed_parameters,
        transcript_params: (),
        wrapper_function,
    };

    let mut assembly = PlonkProvingAssembly::new();

    let synth_start = std::time::Instant::now();
    circuit.synthesize(&mut assembly).expect("synthesis failed");
    let synth_elapsed = synth_start.elapsed();

    println!("=== SNARK Synthesize Benchmark ===");
    println!("Synthesis time: {:.3} s", synth_elapsed.as_secs_f64());
    println!(
        "Variables (aux): {}",
        assembly.num_aux
    );
    println!(
        "Input gates: {}, Aux gates: {}",
        assembly.num_input_gates, assembly.num_aux_gates
    );

    let finalize_start = std::time::Instant::now();
    assembly.finalize_to_size_log_2(crate::L1_VERIFIER_DOMAIN_SIZE_LOG);
    let finalize_elapsed = finalize_start.elapsed();

    println!("Finalize time:   {:.3} s", finalize_elapsed.as_secs_f64());
    println!(
        "Total (synth+finalize): {:.3} s",
        (synth_elapsed + finalize_elapsed).as_secs_f64()
    );

    assert!(assembly.is_satisfied(), "assembly is not satisfied");
    println!("Assembly is satisfied: OK");
}
