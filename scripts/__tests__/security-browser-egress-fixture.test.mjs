import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const fixture = fileURLToPath(
  new URL("../ci/security-browser-egress-fixture.sh", import.meta.url),
);

test("native browser egress fixture runs its ignored TinySecurity test", () => {
  const dir = mkdtempSync(join(tmpdir(), "security-browser-egress-fixture-"));
  const log = join(dir, "invocations.txt");
  const bash = join(dir, "bash");
  writeFileSync(
    bash,
    `#!/bin/sh\nprintf 'ARGS %s\\n' "$*" >> "${log}"\n`,
  );
  chmodSync(bash, 0o755);

  const result = spawnSync("/bin/bash", [fixture], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${dir}:${process.env.PATH}`,
      OPENHUMAN_TEST_SECURITY_MODULE: fixture,
    },
  });

  assert.equal(result.status, 0, result.stderr);
  const calls = readFileSync(log, "utf8");
  assert.match(calls, /cargo build -p tinysecurity-module/);
  assert.match(calls, /cargo test -p openhuman --lib/);
  assert.match(calls, /--features[^\n]*native-security-tests/);
  assert.match(
    calls,
    /modules::browser_egress_host::tests::browser_egress_callbacks_use_tinysecurity_one_use_grants_and_leases -- --ignored --exact/,
  );
});
