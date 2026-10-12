import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
  existsSync,
  rmSync,
  statSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  readRegistrySource,
  resolveTestModuleAssets,
} from "../ci/self-hosted/test-module-assets.mjs";

test("native document regressions receive a checksum-pinned module fixture", () => {
  const assets = resolveTestModuleAssets(
    readRegistrySource(),
    "ubuntu-22.04-x86_64",
  );
  const documents = assets.find((asset) => asset.id === "tinydocs");
  assert.ok(
    documents,
    "the full Rust suite must include its native document fixture",
  );
  assert.match(documents.url, /\/releases\/download\/v\d+\.\d+\.\d+\//);
  assert.match(documents.sha256, /^[a-f0-9]{64}$/);
});

const installer = fileURLToPath(
  new URL("../ci/self-hosted/install-test-modules.sh", import.meta.url),
);
const localRustRunner = fileURLToPath(
  new URL("../test-rust-with-mock.sh", import.meta.url),
);

test("locally built native Rust test modules use the checkout target tree", () => {
  const source = readFileSync(localRustRunner, "utf8");
  assert.match(source, /target\/native-test-modules\/tinyjuice/);
  assert.match(source, /target\/native-test-modules\/tinysearch/);
  assert.match(source, /target\/native-test-modules\/tinyconnectors/);
  assert.doesNotMatch(
    source,
    /vendor\/tiny(?:juice|search|connectors)\/target/,
  );
});

test("local MCP registry E2E tests load the pinned TinyMCP module", () => {
  const source = readFileSync(localRustRunner, "utf8");
  assert.match(source, /test-module-assets\.mjs .*tinymcp/);
  assert.match(source, /target\/test-modules\/tinymcp/);
  assert.match(source, /OPENHUMAN_MODULE_PATH="\$tinymcp_dir/);
});

test("MCP registry Rust integration tests run serially", () => {
  const source = readFileSync(localRustRunner, "utf8");
  assert.match(
    source,
    /mcp_registry_e2e[\s\S]*?--test-threads=1/,
    "the TinyMCP process state is shared across test cases",
  );
});

for (const [description, resolverExit] of [
  ["failed", 7],
  ["empty", 0],
]) {
  test(`${description} module asset resolution refuses to publish fixture paths`, () => {
    const root = mkdtempSync(join(tmpdir(), "module-install-"));
    try {
      const bin = join(root, "bin");
      mkdirSync(bin);
      writeFileSync(join(bin, "node"), `#!/bin/sh\nexit ${resolverExit}\n`, {
        mode: 0o755,
      });
      // No builds or downloads are needed to verify failure admission.
      for (const command of ["git", "cargo", "rustc", "curl"]) {
        writeFileSync(
          join(bin, command),
          `#!/bin/sh\ntouch '${root}/attempted'\nexit 0\n`,
          { mode: 0o755 },
        );
      }
      const envFile = join(root, "modules.env");
      const result = spawnSync(
        "bash",
        [installer, join(root, "modules"), envFile],
        {
          encoding: "utf8",
          env: {
            ...process.env,
            PATH: `${bin}:${process.env.PATH}`,
            CI_CACHE_DIR: "",
          },
        },
      );
      assert.notEqual(result.status, 0, result.stdout + result.stderr);
      assert.equal(
        existsSync(join(root, "attempted")),
        false,
        "resolve pins before building or downloading",
      );
      assert.equal(
        existsSync(envFile),
        false,
        "unresolved modules must not be exported",
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
}

for (const id of [
  "tinybox",
  "tinywallet",
  "tinychannels",
  "tinymcp",
  "tinyhosts",
]) {
  test(`native ${id} regressions receive their pinned release fixture`, () => {
    const assets = resolveTestModuleAssets(
      readRegistrySource(),
      "ubuntu-22.04-x86_64",
    );
    const artifact = assets.find((asset) => asset.id === id);
    assert.ok(
      artifact,
      `${id} must not fall back to an untrusted runner cache`,
    );
    assert.match(artifact.sha256, /^[a-f0-9]{64}$/);
    assert.match(artifact.url, /\/releases\/download\/v/);
  });
}

for (const [runner, archiveVariable, directoryVariable] of [
  ["test-rust-e2e.sh", "tinybox_archive", "tinybox_dir"],
  ["test-rust-with-mock.sh", "hosts_archive", "hosts_dir"],
  ["test-rust-with-mock.sh", "tinybox_archive", "tinybox_dir"],
]) {
  test(`${runner} extracts ${archiveVariable} with the current user's ownership`, () => {
    const source = readFileSync(
      fileURLToPath(new URL(`../${runner}`, import.meta.url)),
      "utf8",
    );
    const extraction = source
      .split("\n")
      .find(
        (line) =>
          line.trimStart().startsWith("tar ") &&
          line.includes(`"$${archiveVariable}"`),
      );
    assert.ok(extraction, "runner must extract the pinned archive");
    const root = mkdtempSync(join(tmpdir(), "module-ownership-"));
    try {
      const payload = join(root, "payload");
      const destination = join(root, "extracted");
      mkdirSync(payload);
      mkdirSync(destination);
      writeFileSync(join(payload, "module.so"), "native module fixture");
      const archive = join(root, "module.tar.gz");
      const packed = spawnSync(
        "python3",
        [
          "-c",
          [
            "import sys, tarfile",
            "with tarfile.open(sys.argv[1], 'w:gz') as archive:",
            "    info = archive.gettarinfo(sys.argv[2], arcname='module.so')",
            "    info.uid = info.gid = 12345",
            "    info.uname = info.gname = ''",
            "    with open(sys.argv[2], 'rb') as payload:",
            "        archive.addfile(info, payload)",
          ].join("\n"),
          archive,
          join(payload, "module.so"),
        ],
        { encoding: "utf8" },
      );
      assert.equal(packed.status, 0, packed.stderr);
      // Force the superuser default even on unprivileged developer machines.
      // The runner's --no-same-owner must override it on the real tar command.
      const result = spawnSync(
        "bash",
        [
          "-c",
          ['tar() { command tar --same-owner "$@"; }', extraction].join("\n"),
        ],
        {
          encoding: "utf8",
          env: {
            ...process.env,
            [archiveVariable]: archive,
            [directoryVariable]: destination,
          },
        },
      );
      assert.equal(result.status, 0, result.stdout + result.stderr);
      assert.equal(
        statSync(join(destination, "module.so")).uid,
        statSync(destination).uid,
      );
      assert.equal(
        statSync(join(destination, "module.so")).gid,
        statSync(destination).gid,
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
}

for (const runner of ["test-rust-with-mock.sh", "test-rust-e2e.sh"]) {
  test(`${runner} supplies the TinyMCP artifact and executes ignored registry cases`, () => {
    const source = readFileSync(
      new URL(`../${runner}`, import.meta.url),
      "utf8",
    );
    const setupStart = source.indexOf('tinymcp_asset="');
    assert.ok(setupStart >= 0, "runner prepares the pinned TinyMCP fixture");
    const setupEnd = source.indexOf("\n\n", setupStart);
    const setup = source.slice(setupStart, setupEnd);
    const root = mkdtempSync(join(tmpdir(), "mcp-runner-"));
    try {
      const moduleDir = join(root, "target/test-modules/tinymcp/test-digest");
      mkdirSync(moduleDir, { recursive: true });
      writeFileSync(join(moduleDir, "libtinymcp_module.so"), "fixture");
      // Evaluate the real runner branches with only external commands stubbed.
      // A cached fixture avoids downloads while exercising the path export.
      const cargoStub = `cargo_test() {
        if [[ " $* " == *" --test mcp_registry_e2e "* ]]; then
          printf 'ARTIFACT=%s\\n' "\${OPENHUMAN_TINYMCP_TEST_MODULE:-missing}"
          printf 'ARG=%s\\n' "$@"
        fi
      }`;
      let invocation;
      if (runner === "test-rust-with-mock.sh") {
        const start = source.indexOf("run_full_suite() {");
        const end = source.indexOf("\n}\n", start);
        const directStart = source.indexOf(
          'elif [ "$#" -ge 2 ] && [ "$1" = "--test" ] && [ "$2" = "mcp_registry_e2e" ]',
        );
        const directEnd = source.indexOf("\nelse\n", directStart);
        invocation = [
          "integration_test_targets() { echo mcp_registry_e2e; }",
          "run_build_only_reaper_test() { :; }",
          source.slice(start, end + 3),
          "run_full_suite",
          "set -- --test mcp_registry_e2e -- --nocapture",
          "if false; then :",
          source.slice(directStart, directEnd),
          "fi",
        ].join("\n");
      } else {
        const start = source.indexOf('  if [ "$suite" = "mcp_registry_e2e" ]');
        const end = source.indexOf('  if [ "$HAS_EXTRA_ARGS"', start);
        invocation = [
          "suite=mcp_registry_e2e",
          "CARGO_BIN=cargo_test",
          "EXTRA_ARGS=(--nocapture)",
          'bash() { shift; "$@"; }',
          "for once in 1; do",
          source.slice(start, end),
          "done",
        ].join("\n");
      }
      const result = spawnSync(
        "bash",
        [
          "-c",
          [
            "set -euo pipefail",
            'REPO_ROOT="$TEST_ROOT"',
            'SCRIPT_DIR="$TEST_ROOT"',
            "wallet_platform=ubuntu-22.04-x86_64",
            "module_platform=ubuntu-22.04-x86_64",
            "module_ext=so",
            "PRODUCT_FEATURES=mcp",
            "connectors_module=unused",
            "node() { printf 'tinymcp\\tunused\\tmodule.tar.gz\\ttest-digest\\n'; }",
            setup,
            cargoStub,
            invocation,
          ].join("\n"),
        ],
        {
          encoding: "utf8",
          env: {
            ...process.env,
            TEST_ROOT: root,
            OPENHUMAN_TINYMCP_TEST_MODULE: "",
          },
        },
      );
      assert.equal(result.status, 0, result.stdout + result.stderr);
      const runs = result.stdout.split("ARTIFACT=").slice(1);
      assert.equal(runs.length, runner === "test-rust-with-mock.sh" ? 2 : 1);
      for (const run of runs) {
        assert.equal(
          run.split("\n")[0],
          join(moduleDir, "libtinymcp_module.so"),
        );
        assert.match(
          run,
          /^ARG=--ignored$/m,
          "ignored cases must actually execute",
        );
        assert.match(run, /^ARG=--test-threads=1$/m);
      }
      assert.match(
        runs.at(-1),
        /^ARG=--nocapture$/m,
        "caller arguments survive",
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
}
