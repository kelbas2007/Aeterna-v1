# FRESH-G10 — STATISTICAL QUALIFICATION OF AUTONOMOUS COMPOSITE CONCEPT CONSTRUCTION

Status: **PRE-REGISTERED BEFORE FRESH-G10 EVALUATOR IMPLEMENTATION/RUN**

Date: 2026-10-07

## Claim

Can Aeterna-v1 repeatedly construct a new reusable composite concept from acquired lower-level relation atoms when no individual child is predictive enough, and use that concept on unseen absolute bindings and opaque motor mappings?

This qualifies the bounded G10 composite-construction mechanism. It does not qualify unrestricted concept invention or AGI.

## Fresh authority

Before execution:
- production G10 mechanism is committed;
- this protocol is committed;
- no authority-derived pack is observed.

At first-attempt GitHub Actions execution:
- authority seed = `github.run_id`;
- source SHA = `github.sha`;
- spec SHA = blob SHA of this file;
- run attempt must equal 1;
- exactly 10 deterministic sub-seeds are derived.

The complete relation-atom selection, pair assignment, motor permutation and held-out binding plan is printed and hashed before any tuition/scoring. The first observed pack is burned regardless of PASS/FAIL.

## Relation pool

Each sub-seed selects four distinct local two-pixel relation types from this fixed generic relative-offset pool:

`[(1,0),(0,1),(1,1),(2,0),(0,2),(2,1),(1,2),(2,2)]`.

These are raw geometric offsets only; no semantic relation labels enter cognition.

The four selected atoms are randomly permuted into temporary evaluator positions p0,p1,p2,p3.

The balanced XOR-like pair assignment is:

- class 0: {p0,p1} and {p2,p3};
- class 1: {p0,p2} and {p1,p3}.

Optionally flipping the two evaluator classes is equivalent to the independent opaque motor permutation and does not change the no-single-child-predictiveness property.

Every selected atom therefore appears in one class-0 and one class-1 pair.

## Tuition

For each sub-seed instantiate matched GENUINE and NO_CONSTRUCTION cold EvoPhase organisms.

- raw input is 12x12;
- two spatially separated motifs appear per scene;
- both opaque actions are factually tried for every tuition scene;
- four pair combinations x four absolute tuition bindings;
- composite readout OFF during tuition;
- GENUINE composite formation ON;
- NO_CONSTRUCTION composite formation OFF;
- all other observations/actions/Need are identical.

No evaluator atom/pair/class name enters cognition.

## Fresh held-out pack

Per sub-seed:
- two held-out absolute binding layouts are used for each of the four pair combinations;
- none of these layouts appears in tuition;
- 8 scored worlds per sub-seed.

Total N = **80**.

Learning is frozen before scoring.

## Arms

1. **FULL_COMPOSITE**
   - learned atom inventory;
   - learned composite construction;
   - composite readout ON.

2. **NO_CONSTRUCTION / ATOMS_ONLY**
   - matched learned atom inventory/statistics;
   - no promoted composites;
   - action chosen only from aggregate child-atom evidence.

3. **NO_READOUT / ATOMS_ONLY**
   - clone of FULL after tuition;
   - composite readout OFF;
   - action chosen only from aggregate child-atom evidence.

## Reporting

Before scoring report:
- source SHA;
- spec SHA;
- authority seed;
- pack digest;
- four selected offsets and their evaluator permutation per sub-seed;
- opaque motor swap per sub-seed;
- all held-out binding layouts.

After scoring report:
- FULL /80 and Wilson 95%;
- ATOMS_ONLY controls /80;
- per-sub-seed FULL counts;
- number of acquired atoms/composites per sub-seed;
- maximum absolute individual-child action evidence;
- minimum winning composite action evidence;
- atom-inventory mismatch count between matched arms.

## Frozen PASS thresholds

FRESH-G10 PASS requires all:

1. Exactly N=80 over exactly 10 sub-seeds.
2. FULL_COMPOSITE >= **76/80**.
3. Wilson 95% lower bound >= **0.87**.
4. Every sub-seed FULL >= **6/8**.
5. NO_CONSTRUCTION <= **48/80**.
6. NO_READOUT <= **48/80**.
7. FULL exceeds each control by >= **0.30** absolute.
8. Every sub-seed acquires exactly 4 selected lower-level atoms in both matched arms.
9. Atom inventories between GENUINE and NO_CONSTRUCTION have zero ID/prototype/support mismatches.
10. Every promoted/used composite references only acquired child atom IDs.
11. Maximum absolute individual-child action evidence <= **0.20 + tolerance**.
12. Minimum winning composite action evidence >= **0.60 - tolerance**.
13. Every sub-seed promotes all four useful pair composites in GENUINE and zero in NO_CONSTRUCTION.
14. Opaque motor swaps include both mappings across the 10 sub-seeds.
15. Ownership audit and all prior regressions PASS.
16. Release build PASS.

A workflow/compile failure before pack generation is technical. A completed authority-scored run missing any threshold is a burned scientific FAIL.

## Interpretation boundary

PASS would establish statistical bounded self-construction of composite predicates from acquired child carrier structures across new relation choices and bindings.

It would still not establish arbitrary arity, recursive concept hierarchies, natural-language semantics, autonomous invention of primitive feature vocabularies, AGI or consciousness.
