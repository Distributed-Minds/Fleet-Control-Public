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

class SetArrayError extends Error {
  constructor(reason, detail = "") {
    super(detail ? `${reason} (${detail})` : reason);
    this.reason = reason;
    this.detail = detail;
  }
}

const SET_ARRAY_NOT_CANONICAL = "SET_ARRAY_NOT_CANONICAL";

// SEM-ORDER-1: ECMAScript relational operators compare strings by UTF-16 code
// units, matching RFC 8785 object-property ordering.
function utf16Compare(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

function compareRef(a, b) {
  const byId = utf16Compare(a.id, b.id);
  return byId !== 0 ? byId : utf16Compare(a.definition_hash, b.definition_hash);
}

function refsContain(refs, wanted) {
  return refs.some((ref) => compareRef(ref, wanted) === 0);
}

function validateRefArray(refs, field) {
  for (let i = 1; i < refs.length; i++) {
    if (compareRef(refs[i - 1], refs[i]) >= 0) {
      throw new SetArrayError(SET_ARRAY_NOT_CANONICAL, field);
    }
  }
  const ids = new Set();
  for (const ref of refs) {
    if (ids.has(ref.id)) throw new SetArrayError(SET_ARRAY_NOT_CANONICAL, `${field}:duplicate_id`);
    ids.add(ref.id);
  }
}

function validateSetArrays(normative) {
  for (const field of ["requires", "optional_requires", "conflicts", "exports"]) {
    if (field in normative) validateRefArray(normative[field], field);
  }
  if ("extends" in normative) {
    const edges = normative.extends;
    for (let i = 1; i < edges.length; i++) {
      const byChild = utf16Compare(edges[i - 1].child, edges[i].child);
      const cmp = byChild !== 0 ? byChild : utf16Compare(edges[i - 1].parent, edges[i].parent);
      if (cmp >= 0) throw new SetArrayError(SET_ARRAY_NOT_CANONICAL, "extends");
    }
  }
  for (const field of ["ordering_constraints", "visibility_constraints"]) {
    if (field in normative) validateRefArray(normative[field].accepted ?? [], `${field}.accepted`);
  }
  if ("authority_scopes" in normative) {
    const scopes = normative.authority_scopes;
    for (let i = 1; i < scopes.length; i++) {
      if (utf16Compare(scopes[i - 1], scopes[i]) >= 0) {
        throw new SetArrayError(SET_ARRAY_NOT_CANONICAL, "authority_scopes");
      }
    }
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
    for (const dependency of [...(definition.requires ?? [])].sort(compareRef)) visit(dependency);
    state.set(semanticId, 2);
  }

  for (const selected of [...inp.selected_profiles].sort(compareRef)) visit(selected);

  const sortedClosure = () => [...closure.values()].sort((a, b) => compareRef(a.semantic, b.semantic));

  for (const definition of sortedClosure()) {
    for (const optional of [...(definition.optional_requires ?? [])].sort(compareRef)) {
      if (closure.has(optional.id)) {
        const activeHash = closure.get(optional.id).semantic.definition_hash;
        if (activeHash !== optional.definition_hash) {
          throw new CompositionError("SEMANTIC_DEFINITION_CONFLICT");
        }
      }
    }
  }

  const activeIds = new Set(closure.keys());
  for (const definition of sortedClosure()) {
    for (const conflict of [...(definition.conflicts ?? [])].sort(compareRef)) {
      if (activeIds.has(conflict.id)) {
        const otherHash = closure.get(conflict.id).semantic.definition_hash;
        if (otherHash === conflict.definition_hash) throw new CompositionError("PROFILE_CONFLICT");
      }
    }
  }

  const concepts = new Map();
  for (const definition of sortedClosure()) {
    for (const exported of definition.exports ?? []) {
      if (concepts.has(exported.id) && concepts.get(exported.id).definition_hash !== exported.definition_hash) {
        throw new CompositionError("CONCEPT_DEFINITION_CONFLICT");
      }
      concepts.set(exported.id, exported);
    }
  }

  for (const definition of sortedClosure()) {
    for (const edge of definition.extends ?? []) {
      if (edge.child === edge.parent || !concepts.has(edge.parent) || !concepts.has(edge.child)) {
        throw new CompositionError("INVALID_EXTENSION_TARGET");
      }
    }
  }

  let ordering = inp.ordering_model;
  if (ordering === undefined && Object.hasOwn(inp, "ordering_preferences")) {
    for (const candidate of inp.ordering_preferences) {
      const compatible = [...closure.values()].every((definition) => {
        const accepted = definition.ordering_constraints?.accepted ?? [];
        return !accepted.length || refsContain(accepted, candidate);
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
    for (const definition of closure.values()) {
      const accepted = definition.ordering_constraints?.accepted ?? [];
      if (accepted.length && !refsContain(accepted, ordering)) {
        throw new CompositionError("ORDERING_MODEL_CONFLICT");
      }
    }
  }

  let visibility = inp.visibility_policy;
  if (visibility === undefined && Object.hasOwn(inp, "visibility_preferences")) {
    for (const candidate of inp.visibility_preferences) {
      const compatible = [...closure.values()].every((definition) => {
        const accepted = definition.visibility_constraints?.accepted ?? [];
        return !accepted.length || refsContain(accepted, candidate);
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
    for (const definition of closure.values()) {
      const accepted = definition.visibility_constraints?.accepted ?? [];
      if (accepted.length && !refsContain(accepted, visibility)) {
        throw new CompositionError("VISIBILITY_POLICY_CONFLICT");
      }
    }
  }

  const result = {
    profiles: [...closure.values()].map((definition) => definition.semantic).sort(compareRef),
    concepts: [...concepts.values()].sort(compareRef),
  };
  if (ordering) result.ordering_model = ordering;
  if (visibility) result.visibility_policy = visibility;
  return result;
}

function negotiateOptional(inp) {
  const selected = [...inp.selected_profiles];
  let composition = compose({ ...inp, selected_profiles: selected });
  const activatedOptional = [];
  const skippedOptional = [];

  for (const candidate of inp.optional_profile_preferences ?? []) {
    try {
      const trialComposition = compose({
        ...inp,
        selected_profiles: [...selected, candidate],
      });
      selected.push(candidate);
      activatedOptional.push(candidate);
      composition = trialComposition;
    } catch (err) {
      if (!(err instanceof CompositionError)) throw err;
      skippedOptional.push({ profile: candidate, reason: err.reason });
    }
  }

  return {
    composition,
    activated_optional: activatedOptional,
    skipped_optional: skippedOptional,
  };
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function runVectors(vectorDir) {
  const failures = [];
  const hv = readJson(path.join(vectorDir, "hash-vector-001.json"));
  try {
    validateSetArrays(hv.definition.normative);
  } catch (err) {
    if (!(err instanceof SetArrayError)) throw err;
    failures.push(`HASH-VECTOR-001 unexpected set-array rejection: ${err.reason}`);
  }
  const actualJcs = canonicalize(hv.definition.normative);
  const actualHash = sha256Jcs(hv.definition.normative);
  if (actualJcs !== hv.expected_jcs) failures.push("HASH-VECTOR-001 JCS mismatch");
  if (actualHash !== hv.expected_definition_hash) failures.push("HASH-VECTOR-001 digest mismatch");

  const setVectors = readJson(path.join(vectorDir, "hash-set-array-vectors.json"));
  for (const vector of setVectors) {
    const expectedReason = vector.expected_reason ?? null;
    let rejectedReason = null;
    try {
      validateSetArrays(vector.normative);
    } catch (err) {
      if (!(err instanceof SetArrayError)) throw err;
      rejectedReason = err.reason;
    }
    if (expectedReason === null) {
      if (rejectedReason !== null) failures.push(`${vector.vector_id} unexpectedly rejected: ${rejectedReason}`);
      else if (vector.expected_definition_hash && sha256Jcs(vector.normative) !== vector.expected_definition_hash) {
        failures.push(`${vector.vector_id} digest mismatch`);
      }
    } else if (rejectedReason === null) {
      failures.push(`${vector.vector_id} expected ${expectedReason}, accepted instead`);
    } else if (rejectedReason !== expectedReason) {
      failures.push(`${vector.vector_id} expected ${expectedReason}, got ${rejectedReason}`);
    }
  }

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

  const unicodeVectors = readJson(path.join(vectorDir, "composition-unicode-vectors.json"));
  for (const vector of unicodeVectors) {
    const actual = compose(vector.input);
    if (canonicalize(actual) !== canonicalize(vector.expected_composition)) {
      failures.push(`${vector.vector_id} composition mismatch`);
    }
    if (canonicalize(actual) !== vector.expected_jcs) {
      failures.push(`${vector.vector_id} JCS mismatch`);
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

  const optionalVector = readJson(path.join(vectorDir, "optional-negotiation-vector-001.json"));
  const optionalResult = negotiateOptional(optionalVector.input);
  if (canonicalize(optionalResult) !== canonicalize(optionalVector.expected_result)) {
    failures.push("OPTIONAL-NEGOTIATION-VECTOR-001 result mismatch");
  }
  if (sha256Jcs(optionalResult) !== optionalVector.expected_result_hash) {
    failures.push("OPTIONAL-NEGOTIATION-VECTOR-001 digest mismatch");
  }

  const contractVector = readJson(path.join(vectorDir, "contract-vector-001.json"));
  try {
    validateRefArray(contractVector.contract.profiles, "contract.profiles");
  } catch (err) {
    if (!(err instanceof SetArrayError)) throw err;
    failures.push(`CONTRACT-VECTOR-001 unexpected set-array rejection: ${err.reason}`);
  }
  const contractJcs = canonicalize(contractVector.contract);
  const contractHash = sha256Jcs(contractVector.contract);
  if (contractJcs !== contractVector.expected_jcs) failures.push("CONTRACT-VECTOR-001 JCS mismatch");
  if (contractHash !== contractVector.expected_contract_hash) failures.push("CONTRACT-VECTOR-001 digest mismatch");

  const setOrderNegatives = readJson(path.join(vectorDir, "composition-set-order-vectors.json"));
  for (const vector of setOrderNegatives) {
    try {
      compose(vector.input);
      failures.push(`${vector.vector_id} unexpectedly composed successfully`);
    } catch (err) {
      if (!(err instanceof CompositionError)) throw err;
      if (err.reason !== vector.expected_reason) failures.push(`${vector.vector_id} expected ${vector.expected_reason}, got ${err.reason}`);
    }
  }

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
  console.log("PASS: OPTIONAL-NEGOTIATION-VECTOR-001");
  console.log("PASS: CONTRACT-VECTOR-001");
  console.log(`PASS: ${additional.length} additional composition vectors`);
  console.log(`PASS: ${negatives.length} negative composition vectors`);
  console.log(`PASS: ${setVectors.length} HASH05 set-array vectors`);
  console.log(`PASS: ${unicodeVectors.length} non-BMP composition vectors`);
  console.log(`PASS: ${setOrderNegatives.length} deterministic set-order negative vectors`);
}

if (process.argv.length !== 3) {
  console.error("usage: reference.mjs <test-vector-directory>");
  process.exit(2);
}
runVectors(process.argv[2]);
