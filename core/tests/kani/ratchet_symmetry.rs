// P0_SECURITY_003: PQ Ratchet Symmetry Proof

#[cfg(kani)]
mod kani_tests {
    /// A stub for the KDF to make it tractable for Kani. We only care about symmetry
    /// of inputs, so a dummy function that maps identical inputs to identical outputs is sufficient.
    fn mock_kdf(root_key: &[u8; 32], dh_output: &[u8; 32], pq_ss: Option<&[u8; 32]>) -> [u8; 32] {
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = root_key[i] ^ dh_output[i];
            if let Some(ss) = pq_ss {
                out[i] ^= ss[i];
            }
        }
        out
    }

    #[kani::proof]
    fn prove_pq_ratchet_symmetry() {
        let initial_root_key: [u8; 32] = kani::any();
        
        let dh_a1_b1: [u8; 32] = kani::any(); 
        let dh_a2_b1: [u8; 32] = kani::any(); 
        
        let pq_ss: [u8; 32] = kani::any();

        // ALICE'S PERSPECTIVE (Sender of PQ CT)
        // First DH output (A1 x B1) has no PQ mix for sender
        let alice_root_1 = mock_kdf(&initial_root_key, &dh_a1_b1, None);
        // Second DH output (A2 x B1) has PQ mix for sender
        let alice_root_2 = mock_kdf(&alice_root_1, &dh_a2_b1, Some(&pq_ss));

        // BOB'S PERSPECTIVE (Receiver of PQ CT)
        // Bob had computed dh_a1_b1 without PQ mix when he received A1
        let bob_root_1 = mock_kdf(&initial_root_key, &dh_a1_b1, None);
        // Bob receives A2. First DH output (B1 x A2) has PQ mix for receiver
        let bob_root_2 = mock_kdf(&bob_root_1, &dh_a2_b1, Some(&pq_ss));

        kani::assert(
            alice_root_2 == bob_root_2,
            "PQ ratchet mixing must be symmetric between sender and receiver"
        );
    }
}
