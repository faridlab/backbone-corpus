#!/usr/bin/env bash
# §5 round-trip: the KB write path (publish gate + deflection read) + oracle survive a codegen --force regen.
set -euo pipefail
cd "$(dirname "$0")/.."
export DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5433/backbone_corpus}"
FILES=(src/application/service/corpus_write_service.rs tests/corpus_golden_cases.rs tests/integrity_probes.rs tests/corpus_support_seam.rs)
before=$(shasum "${FILES[@]}")
echo "== regenerating (--force) =="
metaphor schema schema generate --force >/dev/null
after=$(shasum "${FILES[@]}")
if [[ "$before" != "$after" ]]; then echo "FAIL: user-owned files changed across regen"; diff <(echo "$before") <(echo "$after"); exit 1; fi
echo "OK: KB write path + oracle byte-identical across regen"
echo "== re-running the oracle + seam =="
cargo test --test corpus_golden_cases --test integrity_probes --test corpus_support_seam 2>&1 | grep -E "test result"
echo "OK: §5 round-trip holds"
