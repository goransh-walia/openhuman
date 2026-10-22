/**
 * Fixture-based self-tests for check-storage-owner.mjs
 *
 * Run with: node --test scripts/ci/__tests__/check-storage-owner.test.mjs
 *
 * Cargo metadata dep_kinds format:
 *   - Normal dep: { kind: "normal" } or absent (treated as normal by cargo)
 *   - Build dep:  { kind: "build" }
 *   - Dev dep:    { kind: "dev" }
 */

import { checkMetadata } from '../check-storage-owner.mjs';

const ROOT = '/fake/repo';
const TSD_MANIFEST = `${ROOT}/vendor/tinystoragedrivers/crates/tinystoragedrivers-sqlite/Cargo.toml`;
const REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index';
const tsd_git = (tag) => `git+https://github.com/tinyhumansai/tinystoragedrivers?tag=${tag}#`;

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

/** Path crate (first-party, source = null). */
const pathPkg = (name, version, manifest, deps = []) => ({
  id: `${name} ${version}`,
  name,
  version,
  manifest_path: manifest,
  source: null,
  dependencies: deps,
});

/** Registry crate. */
const regPkg = (name, version, manifest, deps = []) => ({
  id: `${name} ${version} ${REGISTRY}`,
  name,
  version,
  manifest_path: manifest,
  source: REGISTRY,
  dependencies: deps,
});

/** Normal dep_kinds entry. */
const NORMAL = [{ kind: 'normal' }];

/** Resolve node for a path crate. */
const pathNode = (name, version, deps) => ({
  id: `${name} ${version}`,
  deps: deps.map(([dep_name, dep_ver, dep_source]) => ({
    name: dep_name,
    pkg: dep_source ? `${dep_name} ${dep_ver} ${dep_source}` : `${dep_name} ${dep_ver}`,
    dep_kinds: NORMAL,
  })),
});

/** Resolve node for a registry crate. */
const regNode = (name, version, deps) => ({
  id: `${name} ${version} ${REGISTRY}`,
  deps: deps.map(([dep_name, dep_ver, dep_source]) => ({
    name: dep_name,
    pkg: dep_source ? `${dep_name} ${dep_ver} ${dep_source}` : `${dep_name} ${dep_ver}`,
    dep_kinds: NORMAL,
  })),
});

// ---------------------------------------------------------------------------
// Fixture: transitive engine through tsd crate → must FAIL
// tinyagents-session (path) -> tinystoragedrivers-sqlite (path) -> rusqlite (registry)
// ---------------------------------------------------------------------------

const transitive_engine_fixture = {
  packages: [
    pathPkg('tinystoragedrivers-sqlite', '1.0.0', TSD_MANIFEST, [
      { name: 'rusqlite', pkg: `rusqlite 0.32.1 ${REGISTRY}` },
    ]),
    regPkg('rusqlite', '0.32.1', `${ROOT}/.cargo/registry/src/foo/rusqlite-0.32.1/Cargo.toml`),
    pathPkg('tinyagents-session', '0.1.0', `${ROOT}/crates/tinyagents-session/Cargo.toml`, [
      { name: 'tinystoragedrivers-sqlite', pkg: 'tinystoragedrivers-sqlite 1.0.0' },
    ]),
  ],
  resolve: {
    nodes: [
      pathNode('tinyagents-session', '0.1.0', [['tinystoragedrivers-sqlite', '1.0.0', null]]),
      pathNode('tinystoragedrivers-sqlite', '1.0.0', [['rusqlite', '0.32.1', REGISTRY]]),
      regNode('rusqlite', '0.32.1', []),
    ],
  },
};

// ---------------------------------------------------------------------------
// Fixture: tsd tag mismatch → must FAIL
// some-odd-crate (path) depends on tinystoragedrivers-core with git tag "99.0.0"
// but vendored version is "1.0.0".
// Note: dep.source must be set for the mismatch check to read the tag.
// ---------------------------------------------------------------------------

const tsd_tag_mismatch_fixture = {
  packages: [
    pathPkg('tinystoragedrivers-core', '1.0.0',
      `${ROOT}/vendor/tinystoragedrivers/crates/tinystoragedrivers-core/Cargo.toml`),
    // some-odd-crate depends on tinystoragedrivers-core with an explicit git source
    // whose tag differs from the vendored version.
    {
      id: `some-odd-crate 0.1.0`,
      name: 'some-odd-crate',
      version: '0.1.0',
      manifest_path: `${ROOT}/crates/some-odd-crate/Cargo.toml`,
      source: null,
      dependencies: [
        {
          name: 'tinystoragedrivers-core',
          pkg: `tinystoragedrivers-core 1.0.0 ${tsd_git('99.0.0')}`,
          source: tsd_git('99.0.0'),  // <-- must be set for tag mismatch check
        },
      ],
    },
  ],
  resolve: {
    nodes: [
      pathNode('some-odd-crate', '0.1.0', [['tinystoragedrivers-core', '1.0.0', tsd_git('99.0.0')]]),
      pathNode('tinystoragedrivers-core', '1.0.0', []),
    ],
  },
};

// ---------------------------------------------------------------------------
// Fixture: allowlisted root reaches engine → must PASS
// openhuman (allowlisted) -> rusqlite
// ---------------------------------------------------------------------------

const allowlisted_root_fixture = {
  packages: [
    pathPkg('openhuman', '0.1.0', `${ROOT}/crates/openhuman/Cargo.toml`, [
      { name: 'rusqlite', pkg: `rusqlite 0.32.1 ${REGISTRY}` },
    ]),
    regPkg('rusqlite', '0.32.1', '/fake/.cargo/registry/src/foo/rusqlite-0.32.1/Cargo.toml'),
  ],
  resolve: {
    nodes: [
      pathNode('openhuman', '0.1.0', [['rusqlite', '0.32.1', REGISTRY]]),
      regNode('rusqlite', '0.32.1', []),
    ],
  },
};

// ---------------------------------------------------------------------------
// Fixture: crate naming violation → must FAIL
// Name ends with "-sqlite" → forbidden pattern.
// ---------------------------------------------------------------------------

const naming_violation_fixture = {
  packages: [
    pathPkg('tinyagents-sqlite', '0.1.0', `${ROOT}/crates/tinyagents-sqlite/Cargo.toml`),
  ],
  resolve: {
    nodes: [
      pathNode('tinyagents-sqlite', '0.1.0', []),
    ],
  },
};

// ---------------------------------------------------------------------------
// Fixture: all tsd crates vendored → must PASS
// ---------------------------------------------------------------------------

const vendored_ok_fixture = {
  packages: [
    pathPkg('tinystoragedrivers-core', '1.0.0',
      `${ROOT}/vendor/tinystoragedrivers/crates/tinystoragedrivers-core/Cargo.toml`),
    pathPkg('tinystoragedrivers-sqlite', '1.0.0', TSD_MANIFEST),
    pathPkg('openhuman-embed', '0.1.0', `${ROOT}/crates/openhuman-embed/Cargo.toml`, [
      { name: 'tinystoragedrivers-core', pkg: 'tinystoragedrivers-core 1.0.0' },
    ]),
  ],
  resolve: {
    nodes: [
      pathNode('tinystoragedrivers-core', '1.0.0', []),
      pathNode('tinystoragedrivers-sqlite', '1.0.0', []),
      pathNode('openhuman-embed', '0.1.0', [['tinystoragedrivers-core', '1.0.0', null]]),
    ],
  },
};

// ---------------------------------------------------------------------------
// Fixture: aws-sdk-s3 reached directly from non-allowlisted crate → must FAIL
// openhuman-core (NOT allowlisted) -> aws-sdk-s3 (registry)
// ---------------------------------------------------------------------------

const aws_s3_fixture = {
  packages: [
    pathPkg('openhuman-core', '0.1.0', `${ROOT}/crates/openhuman-core/Cargo.toml`, [
      { name: 'aws-sdk-s3', pkg: `aws-sdk-s3 1.0.0 ${REGISTRY}` },
    ]),
    regPkg('aws-sdk-s3', '1.0.0', '/fake/.cargo/registry/src/aws/aws-sdk-s3-1.0.0/Cargo.toml'),
  ],
  resolve: {
    nodes: [
      pathNode('openhuman-core', '0.1.0', [['aws-sdk-s3', '1.0.0', REGISTRY]]),
      regNode('aws-sdk-s3', '1.0.0', []),
    ],
  },
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

const { test, describe } = await import('node:test');

describe('transitive engine through tsd crate → must FAIL', () => {
  test('produces at least one engine-reach violation', () => {
    const violations = checkMetadata(transitive_engine_fixture, ROOT);
    const engineViolations = violations.filter(v => v.kind === 'engine-reach');
    if (engineViolations.length === 0) {
      throw new Error(`Expected engine-reach violation, got: ${JSON.stringify(violations)}`);
    }
  });
});

describe('tsd tag mismatch → must FAIL', () => {
  test('produces a tsd-tag-mismatch violation', () => {
    const violations = checkMetadata(tsd_tag_mismatch_fixture, ROOT);
    const mismatch = violations.filter(v => v.kind === 'tsd-tag-mismatch');
    if (mismatch.length === 0) {
      throw new Error(`Expected tsd-tag-mismatch violation, got: ${JSON.stringify(violations)}`);
    }
  });
});

describe('allowlisted root reaches engine → must PASS', () => {
  test('produces no violations', () => {
    const violations = checkMetadata(allowlisted_root_fixture, ROOT);
    if (violations.length > 0) {
      throw new Error(`Expected no violations for allowlisted root, got: ${JSON.stringify(violations)}`);
    }
  });
});

describe('crate naming violation → must FAIL', () => {
  test('produces a crate-name violation', () => {
    const violations = checkMetadata(naming_violation_fixture, ROOT);
    const naming = violations.filter(v => v.kind === 'crate-name');
    if (naming.length === 0) {
      throw new Error(`Expected crate-name violation, got: ${JSON.stringify(violations)}`);
    }
  });
});

describe('all tsd crates vendored → must PASS', () => {
  test('produces no violations', () => {
    const violations = checkMetadata(vendored_ok_fixture, ROOT);
    if (violations.length > 0) {
      throw new Error(`Expected no violations for vendored-clean tree, got: ${JSON.stringify(violations)}`);
    }
  });
});

describe('aws-sdk-s3 reached directly → must FAIL', () => {
  test('produces an engine-reach violation for aws-sdk-s3', () => {
    const violations = checkMetadata(aws_s3_fixture, ROOT);
    const engineViolations = violations.filter(v =>
      v.kind === 'engine-reach' && v.engine === 'aws-sdk-s3'
    );
    if (engineViolations.length === 0) {
      throw new Error(`Expected engine-reach violation for aws-sdk-s3, got: ${JSON.stringify(violations)}`);
    }
  });
});
