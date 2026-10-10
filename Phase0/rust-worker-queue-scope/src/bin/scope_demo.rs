//! Offline #280 demonstration. Not a real worker assignment exchange.
use free_energy_worker_queue_scope::{
    AdmissionError, ScopeConflictBasis, ScopeSelector, SimulationBook, SCHEMA_VERSION,
};

fn main() -> Result<(), AdmissionError> {
    let basis = ScopeConflictBasis {
        version: SCHEMA_VERSION,
        generation: 1,
        repository_id: 1360059617,
        repository_incarnation: "offline-demo-only".to_owned(),
    };
    let mut book = SimulationBook::new(basis.clone())?;
    let first = vec![ScopeSelector::Conversation(100)];
    let second = vec![ScopeSelector::Conversation(101)];
    let overlap = vec![ScopeSelector::Conversation(100)];
    let a = book.reserve(10, &first, &basis)?.assignment_generation;
    let b = book.reserve(11, &second, &basis)?.assignment_generation;
    assert_ne!(a, b);
    assert_eq!(
        book.reserve(12, &overlap, &basis),
        Err(AdmissionError::Collision { held_by_task: 10 })
    );
    book.require_recovery(10)?;
    assert_eq!(
        book.reserve(12, &overlap, &basis),
        Err(AdmissionError::Collision { held_by_task: 10 })
    );
    println!("SIMULATION_ONLY two disjoint reservations; overlap and recovery hold rejected");
    Ok(())
}
