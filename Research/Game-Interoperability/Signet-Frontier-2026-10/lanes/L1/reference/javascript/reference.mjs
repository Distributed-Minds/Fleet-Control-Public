#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import process from "node:process";

class CompositionError extends Error {
  constructor(reason) {
    super(reason);
    this.reason = reason;
  }
}

function validateString(value) {
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(i + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) throw new Error("JCS input contains a lone high surrogate");
      i++;
    } else if (code >= 0xdc00 && code <= 0xdfff) {
      throw new Error("JCS input contains a lone low surrogate");
    }
  }
}

function quote(value) {
  validateString(value);
  return JSON.stringify(value);
}

function canonicalize(value) {
  if (value === null) return "null";
  if (value === true) return "true";
  if (value === false) return "false";
  if (typeof value === "string") return quote(value);
  if (typeof value === "number") {
    if (!Number.isSafeInteger(value)) throw new Error("numeric value is outside this reference subset");
    return String(value);
  }
  if (Array.isArray(value)) return "[" + value.map(canonicalize).join(",") + "]";
  if (typeof value === "object") {
    const keys = Object.keys(value).sort();
    return "{" + keys.map((key) => quote(key) + ":" + canonicalize(value[key])).join(",") + "}";
  }
  throw new Error(`unsupported JSON value: ${typeof value}`);
}

function sha256Jcs(value) {
  return "sha256:" + crypto.createHash("sha256").update(Buffer.from(canonicalize(value), "utf8")).digest("hex");
}

function refKey(ref) {
  return `${ref.id}\u0000${ref.definition_hash}`;
}

function compose(inp) {
  const catalog = new Map(inp.catalog.map((item) => [item.semantic.id, item]));
  const state = new Map();
  const closure = new Map();

  function visit(ref) {
    const semanticId = ref.id;
    const requestedHash = ref.definition_hash;
    if (!catalog.has(semanticId)) throw new CompositionError("REQUIRED_PROFILE_UNSUPPORTED");
    const definition = catalog.get(semanticId);
    if (definition.semantic.definition_hash !== requestedHash) throw new CompositionError("SEMANTIC_DEFINITION_CONFLICT");
    if (closure.has(semanticId) && closure.get(semanticId).semantic.definition_hash !== requestedHash) {
      throw new CompositionError("SEMANTIC_DEFINITION_CONFLICT");
    }
    if (state.get(semanticId) === 1) throw new CompositionError("REQUIRED_PROFILE_CYCLE");
    if (state.get(semanticId) === 2) return;
    state.set(semanticId, 1);
    closure.set(semanticId, definition);
    for (const dependency of definition.requires ?? []) visit(dependency);
    state.set(semanticId, 2);
  }

  for (const selected of inp.selected_profiles) visit(selected);

  for (const definition of closure.values()) {
    for (const optional of definition.optional_requires ?? []) {
      if (closure.has(optional.id)) {
        const activeHash = closure.get(optional.id).semantic.definition_hash;
        if (activeHash !== optional.definition_hash) {
          throw new CompositionError("SEMANTIC_DEFINITION_CONFLICT");
        }
      }
    }
  }

  const activeIds = new Set(closure.keys());
  for (const definition of closure.values()) {
    for (const conflict of definition.conflicts ?? []) {
      if (activeIds.has(conflict.id)) {
        const otherHash = closure.get(conflict.id).semantic.definition_hash;
        if (otherHash === conflict.definition_hash) throw new CompositionError("PROFILE_CONFLICT");
      }
    }
  }

  const concepts = new Map();
  for (const definition of closure.values()) {
    for (const exported of definition.exports ?? []) {
      if (concepts.has(exported.id) && concepts.get(exported.id).definition_hash !== exported.definition_hash) {
        throw new CompositionError("CONCEPT_DEFINITION_CONFLICT");
      }
      concepts.set(exported.id, exported);
    }
  }

  for (const definition of closure.values()) {
    for (const edge of definition.extends ?? []) {
      if (edge.child === edge.parent || !concepts.has(edge.parent) || !concepts.has(edge.child)) {
        throw new CompositionError("INVALID_EXTENSION_TARGET");
      }
    }
  }

  let ordering = inp.ordering_model;
  if (ordering === undefined && Object.hasOwn(inp, "ordering_preferences")) {
    for (const candidate of inp.ordering_preferences) {
      const wanted = refKey(candidate);
      const compatible = [...closure.values()].every((definition) => {
        const accepted = definition.ordering_constraints?.accepted ?? [];
        return !accepted.length || new Set(accepted.map(refKey)).has(wanted);
      });
      if (compatible) {
        ordering = candidate;
        break;
      }
    }
    if (inp.ordering_preferences.length && ordering === undefined) {
      throw new CompositionError("ORDERING_MODEL_CONFLICT");
    }
  }

  if (ordering) {
    const wanted = refKey(ordering);
    for (const definition of closure.values()) {
      const accepted = definition.ordering_constraints?.accepted ?? [];
      if (accepted.length && !new Set(accepted.map(refKey)).has(wanted)) {
        throw new CompositionError("ORDERING_MODEL_CONFLICT");
      }
    }
  }

  let visibility = inp.visibility_policy;
  if (visibility === undefined && Object.hasOwn(inp, "visibility_preferences")) {
    for (const candidate of inp.visibility_preferences) {
      const wanted = refKey(candidate);
      const compatible = [...closure.values()].every((definition) => {
        const accepted = definition.visibility_constraints?.accepted ?? [];
        return !accepted.length || new Set(accepted.map(refKey)).has(wanted);
      });
      if (compatible) {
        visibility = candidate;
        break;
      }
    }
    if (inp.visibility_preferences.length && visibility === undefined) {
      throw new CompositionError("VISIBILITY_POLICY_CONFLICT");
    }
  }

  if (visibility) {
    const wanted = refKey(visibility);
    for (const definition of closure.values()) {
      const accepted = definition.visibility_constraints?.accepted ?? [];
      if (accepted.length && !new Set(accepted.map(refKey)).has(wanted)) {
        throw new CompositionError("VISIBILITY_POLICY_CONFLICT");
      }
    }
  }

  const result = {
    profiles: [...closure.values()].map((definition) => definition.semantic).sort((a, b) => refKey(a) < refKey(b) ? -1 : refKey(a) > refKey(b) ? 1 : 0),
    concepts: [...concepts.values()].sort((a, b) => refKey(a) < refKey(b) ? -1 : refKey(a) > refKey(b) ? 1 : 0),
  };
  if (ordering) result.ordering_model = ordering;
  if (visibility) result.visibility_policy = visibility;
  return result;
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function runVectors(vectorDir) {
  const failures = [];
  const hv = readJson(path.join(vectorDir, "hash-vector-001.json"));
  const actualJcs = canonicalize(hv.definition.normative);
  const actualHash = sha256Jcs(hv.definition.normative);
  if (actualJcs !== hv.expected_jcs) failures.push("HASH-VECTOR-001 JCS mismatch");
  if (actualHash !== hv.expected_definition_hash) failures.push("HASH-VECTOR-001 digest mismatch");

  const cv = readJson(path.join(vectorDir, "composition-vector-001.json"));
  const actual = compose(cv.input);
  if (canonicalize(actual) !== canonicalize(cv.expected_composition)) failures.push("COMP-VECTOR-001 composition mismatch");
  if (canonicalize(actual) !== cv.expected_jcs) failures.push("COMP-VECTOR-001 JCS mismatch");
  if (sha256Jcs(actual) !== cv.expected_profile_set_hash) failures.push("COMP-VECTOR-001 digest mismatch");

  const additional = readJson(path.join(vectorDir, "composition-additional-vectors.json"));
  for (const vector of additional) {
    const actual = compose(vector.input);
    if (canonicalize(actual) !== canonicalize(vector.expected_composition)) {
      failures.push(`${vector.vector_id} composition mismatch`);
    }
    if (sha256Jcs(actual) !== vector.expected_profile_set_hash) {
      failures.push(`${vector.vector_id} digest mismatch`);
    }
  }

  const negotiationVector = readJson(path.join(vectorDir, "negotiation-vector-001.json"));
  const negotiationResult = compose(negotiationVector.input);
  if (canonicalize(negotiationResult) !== canonicalize(negotiationVector.expected_composition)) {
    failures.push("NEGOTIATION-VECTOR-001 composition mismatch");
  }
  if (sha256Jcs(negotiationResult) !== negotiationVector.expected_profile_set_hash) {
    failures.push("NEGOTIATION-VECTOR-001 digest mismatch");
  }

  const contractVector = readJson(path.join(vectorDir, "contract-vector-001.json"));
  const contractJcs = canonicalize(contractVector.contract);
  const contractHash = sha256Jcs(contractVector.contract);
  if (contractJcs !== contractVector.expected_jcs) failures.push("CONTRACT-VECTOR-001 JCS mismatch");
  if (contractHash !== contractVector.expected_contract_hash) failures.push("CONTRACT-VECTOR-001 digest mismatch");

  const negatives = readJson(path.join(vectorDir, "composition-negative-vectors.json"));
  for (const vector of negatives) {
    try {
      compose(vector.input);
      failures.push(`${vector.vector_id} unexpectedly composed successfully`);
    } catch (err) {
      if (!(err instanceof CompositionError)) throw err;
      if (err.reason !== vector.expected_reason) failures.push(`${vector.vector_id} expected ${vector.expected_reason}, got ${err.reason}`);
    }
  }

  if (failures.length) {
    for (const failure of failures) console.error("FAIL:", failure);
    process.exit(1);
  }
  console.log("PASS: HASH-VECTOR-001");
  console.log("PASS: COMP-VECTOR-001");
  console.log("PASS: NEGOTIATION-VECTOR-001");
  console.log("PASS: CONTRACT-VECTOR-001");
  console.log(`PASS: ${additional.length} additional composition vectors`);
  console.log(`PASS: ${negatives.length} negative composition vectors`);
}

if (process.argv.length !== 3) {
  console.error("usage: reference.mjs <test-vector-directory>");
  process.exit(2);
}
runVectors(process.argv[2]);
