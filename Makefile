# pulseengine/example-kvs — eclipse-score persistency::kvs treatment
# through the full pulseengine stack (rivet + spar + WIT + witness +
# sigil + artifact-driven verification gate).

RIVET   ?= rivet
SPAR    ?= spar
SIGIL   ?= sigil
PYTHON  ?= python3
SCHEMAS := vendor/rivet-schemas

.PHONY: all validate aadl wit verify attest clean

# Default: run all checks that work without external dependencies
# (rivet and the verification gate). aadl/wit/attest are optional —
# they require spar / sigil to be installed.
all: validate verify

# ── rivet typed-artifact validation (always runs) ───────────────────
validate:
	@$(RIVET) --schemas $(SCHEMAS) validate

# ── spar validates the AADL package (optional) ──────────────────────
aadl:
	@command -v $(SPAR) >/dev/null 2>&1 || { \
	    echo "spar not in PATH — install from https://github.com/pulseengine/spar"; \
	    exit 0; }
	@$(SPAR) validate arch/kvs.aadl

# ── spar AADL → WIT round-trip check (optional) ─────────────────────
wit:
	@command -v $(SPAR) >/dev/null 2>&1 || { \
	    echo "spar not in PATH — skipping AADL→WIT round-trip check"; \
	    exit 0; }
	@$(SPAR) emit --format wit arch/kvs.aadl > /tmp/kvs.spar.wit
	@diff -u arch/kvs.wit /tmp/kvs.spar.wit \
	    && echo "AADL → WIT round-trip matches arch/kvs.wit"

# ── Artifact-driven verification gate (always runs) ─────────────────
# Same shape as the LS-N gate in pulseengine/meld. Walks every
# comp-req with status:approved, finds tests by name convention,
# reports PASSED / FAILED / MISSING per artifact. Exits 1 if any
# FAILED or MISSING.
verify:
	@$(PYTHON) tools/verify.py --artifacts artifacts --tests verification

# ── sigil-signed release manifest (optional) ────────────────────────
attest:
	@command -v $(SIGIL) >/dev/null 2>&1 || { \
	    echo "sigil not in PATH — install from https://github.com/pulseengine/sigil"; \
	    echo "(manifest skeleton is at attestation/release-manifest.yaml)"; \
	    exit 0; }
	@$(SIGIL) attest --manifest attestation/release-manifest.yaml

clean:
	@rm -f .verify-output.json /tmp/kvs.spar.wit
