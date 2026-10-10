//! Offline synthetic selector-input reducer for public issue #49 spec 6.
//! Fixture-provided identities/fences are NOT provider proof or execution authority.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub id: String,
    pub incarnation: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub id: String,
    pub incarnation: String,
    pub generation: u64,
    pub valid_from: u64,
    pub valid_until: u64,
    pub high_water: u64,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rank {
    pub model: String,
    pub source: String,
    pub source_incarnation: String,
    pub source_generation: u64,
    pub cut: u64,
    pub score: i64,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub claim: String,
    pub schema: u32,
    pub cut: u64,
    pub policy_generation: u64,
    pub registry_generation: u64,
    pub complete: bool,
    pub jointly_fenced: bool, // synthetic assertion only
    pub models: Vec<Model>,
    pub sources: Vec<Source>,
    pub ranks: Vec<Rank>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Denial {
    Malformed,
    Incomplete,
    UntrustedLineage,
    AmbiguousEvidence,
    NoUniqueWinner,
    EffectTimeDrift,
    ConflictingReplay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub model: String,
    pub score: i64,
    pub semantic_basis: String,
    pub confidence_gain: bool, // always false in synthetic model
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Committed(Selection),
    Reconciled(Selection),
}

fn atom(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':'))
}

fn unsafe_origin(
    id: &str,
    claim: &str,
    all: &BTreeMap<&str, &Source>,
    visiting: &mut BTreeSet<String>,
    clean: &mut BTreeSet<String>,
) -> bool {
    if id == claim {
        return true;
    }
    if clean.contains(id) {
        return false;
    }
    if !visiting.insert(id.to_owned()) {
        return true; // lineage cycle
    }
    let Some(source) = all.get(id) else {
        return true; // unregistered upstream dependency
    };
    for next in &source.dependencies {
        if unsafe_origin(next, claim, all, visiting, clean) {
            return true;
        }
    }
    visiting.remove(id);
    clean.insert(id.to_owned());
    false
}

fn field(out: &mut String, value: &str) {
    write!(out, "{}:", value.len()).expect("infallible String formatter");
    out.push_str(value);
    out.push('|');
}

fn basis(s: &Snapshot) -> String {
    let mut out = String::from("selector-input-v1|");
    for value in [
        s.claim.clone(),
        s.schema.to_string(),
        s.cut.to_string(),
        s.policy_generation.to_string(),
        s.registry_generation.to_string(),
        s.complete.to_string(),
        s.jointly_fenced.to_string(),
    ] {
        field(&mut out, &value);
    }
    let mut models = s.models.clone();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    for m in models {
        field(&mut out, "model");
        field(&mut out, &m.id);
        field(&mut out, &m.incarnation);
    }
    let mut sources = s.sources.clone();
    sources.sort_by(|a, b| a.id.cmp(&b.id));
    for mut source in sources {
        field(&mut out, "source");
        field(&mut out, &source.id);
        field(&mut out, &source.incarnation);
        field(&mut out, &source.generation.to_string());
        field(&mut out, &source.valid_from.to_string());
        field(&mut out, &source.valid_until.to_string());
        field(&mut out, &source.high_water.to_string());
        source.dependencies.sort();
        for dep in source.dependencies {
            field(&mut out, "dep");
            field(&mut out, &dep);
        }
        field(&mut out, "end-deps");
    }
    let mut ranks = s.ranks.clone();
    ranks.sort_by(|a, b| (&a.model, &a.source).cmp(&(&b.model, &b.source)));
    for r in ranks {
        field(&mut out, "rank");
        field(&mut out, &r.model);
        field(&mut out, &r.source);
        field(&mut out, &r.source_incarnation);
        field(&mut out, &r.source_generation.to_string());
        field(&mut out, &r.cut.to_string());
        field(&mut out, &r.score.to_string());
        field(&mut out, &r.unit);
    }
    out
}

/// Strict policy: all current required sources must agree on each candidate's
/// bounded score. Missing/conflicting data and equal best scores fail closed.
pub fn evaluate(s: &Snapshot) -> Result<Selection, Denial> {
    if s.schema != 1
        || !atom(&s.claim)
        || s.cut == 0
        || s.policy_generation == 0
        || s.registry_generation == 0
        || !s.complete
        || !s.jointly_fenced
        || s.models.is_empty()
        || s.sources.is_empty()
        || s.models.len() > 32
        || s.sources.len() > 32
        || s.ranks.len() > 1024
    {
        return Err(Denial::Incomplete);
    }
    let mut models = BTreeSet::new();
    for m in &s.models {
        if !atom(&m.id) || !atom(&m.incarnation) || !models.insert(m.id.as_str()) {
            return Err(Denial::Malformed);
        }
    }
    let mut sources = BTreeMap::new();
    for source in &s.sources {
        if !atom(&source.id)
            || !atom(&source.incarnation)
            || source.generation == 0
            || source.valid_from > s.cut
            || source.valid_until < s.cut
            || source.high_water < s.cut
            // All valid edges must reference the bounded source registry.
            // Cap the raw vector before duplicate checks or graph traversal:
            // an attacker-controlled million-edge list must not consume
            // unbounded resources merely to be rejected later.
            || source.dependencies.len() > s.sources.len()
        {
            return Err(Denial::Incomplete);
        }
        if sources.insert(source.id.as_str(), source).is_some() {
            return Err(Denial::Malformed);
        }
        let mut deps = BTreeSet::new();
        if source
            .dependencies
            .iter()
            .any(|d| !atom(d) || !deps.insert(d))
        {
            return Err(Denial::Malformed);
        }
    }
    let mut clean = BTreeSet::new();
    for source in &s.sources {
        if unsafe_origin(
            &source.id,
            &s.claim,
            &sources,
            &mut BTreeSet::new(),
            &mut clean,
        ) {
            return Err(Denial::UntrustedLineage);
        }
    }
    let mut ranks = BTreeMap::new();
    for r in &s.ranks {
        if !models.contains(r.model.as_str())
            || r.score < 0
            || r.score > 10_000
            || r.unit != "basis-points"
        {
            return Err(Denial::Malformed);
        }
        let Some(src) = sources.get(r.source.as_str()) else {
            return Err(Denial::UntrustedLineage);
        };
        if r.source_incarnation != src.incarnation
            || r.source_generation != src.generation
            || r.cut != s.cut
        {
            return Err(Denial::Incomplete);
        }
        if ranks
            .insert((r.model.as_str(), r.source.as_str()), r.score)
            .is_some()
        {
            return Err(Denial::AmbiguousEvidence);
        }
    }
    if ranks.len() != models.len() * sources.len() {
        return Err(Denial::Incomplete);
    }
    let mut winner: Option<(&str, i64)> = None;
    let mut tied = false;
    for model in models {
        let mut agreed = None;
        for source in sources.keys() {
            let Some(score) = ranks.get(&(model, *source)) else {
                return Err(Denial::Incomplete);
            };
            if agreed.is_some_and(|v| v != *score) {
                return Err(Denial::AmbiguousEvidence);
            }
            agreed = Some(*score);
        }
        let score = agreed.ok_or(Denial::Incomplete)?;
        match winner {
            None => {
                winner = Some((model, score));
                tied = false;
            }
            Some((_, old)) if score < old => {
                winner = Some((model, score));
                tied = false;
            }
            Some((_, old)) if score == old => tied = true,
            _ => {}
        }
    }
    if tied {
        return Err(Denial::NoUniqueWinner);
    }
    let (model, score) = winner.ok_or(Denial::Incomplete)?;
    Ok(Selection {
        model: model.into(),
        score,
        semantic_basis: basis(s),
        confidence_gain: false,
    })
}

/// One-process journal, NOT an atomic persistent provider effects service.
#[derive(Default)]
pub struct Journal {
    records: BTreeMap<u64, (String, Selection)>,
}

impl Journal {
    pub fn submit(
        &mut self,
        operation: u64,
        original: &Snapshot,
        effect: &Snapshot,
    ) -> Result<Outcome, Denial> {
        if operation == 0 {
            return Err(Denial::Malformed);
        }
        let decision = evaluate(original)?;
        if let Some((identity, saved)) = self.records.get(&operation) {
            return if *identity == decision.semantic_basis {
                Ok(Outcome::Reconciled(saved.clone()))
            } else {
                Err(Denial::ConflictingReplay)
            };
        }
        if decision.semantic_basis != evaluate(effect)?.semantic_basis {
            return Err(Denial::EffectTimeDrift);
        }
        self.records.insert(
            operation,
            (decision.semantic_basis.clone(), decision.clone()),
        );
        Ok(Outcome::Committed(decision))
    }
}

#[cfg(test)]
mod tests;
