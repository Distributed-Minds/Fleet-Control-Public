use free_energy_provider_effect_m1::{
    parse_manifest, Capability, ManifestError, EMBEDDED_MANIFEST, MANIFEST_HEADER,
};

#[test]
fn c18_all_current_transports_fail_closed_without_effect_proof() {
    let rows = parse_manifest(EMBEDDED_MANIFEST).expect("versioned manifest must parse");
    assert!(rows.len() >= 8);
    for row in &rows {
        assert_ne!(row.lost_ack_attribution, Capability::SupportedConditionally);
        assert_ne!(row.automatic_replay, Capability::SupportedConditionally);
        assert_ne!(row.target_cas, Capability::SupportedConditionally);
        assert_eq!(row.proof_root, "none");
    }
    assert!(rows.iter().any(|r| r.operation == "merge_pr"));
    assert!(rows.iter().any(|r| r.operation == "create_comment"));
}

#[test]
fn duplicate_operation_transport_and_unknown_schema_fail_closed() {
    let header = format!("{MANIFEST_HEADER}\n");
    let row = "1\tcreate_comment\tgithub-rest\tUNKNOWN\tUNSUPPORTED\tUNSUPPORTED\tUNKNOWN\tnone\n";
    assert!(matches!(
        parse_manifest(&(header.clone() + row + row)),
        Err(ManifestError::DuplicateOperation(3))
    ));
    let invalid = row.replacen("1\t", "9\t", 1);
    assert!(matches!(
        parse_manifest(&(header + &invalid)),
        Err(ManifestError::InvalidRow(2))
    ));
}

#[test]
fn proofless_support_claim_is_rejected() {
    let candidate = format!(
        "{MANIFEST_HEADER}\n1\tcreate_comment\tgithub-rest\tUNKNOWN\tSUPPORTED_CONDITIONALLY\tUNSUPPORTED\tUNKNOWN\tnone\n"
    );
    assert!(matches!(
        parse_manifest(&candidate),
        Err(ManifestError::UnsupportedPositiveClaim(2))
    ));
}
