# Data Loading — Gap Analysis & Action Plan

> Analysis of `tellmeY18/nitc-curriculum-data` against the SyllabiPlan.md spec,
> inspired by OHCNETWORK/care's dynamic form architecture.
> Goal: zero hardcoded data structures — any institution can plug in.

---

## 1. Current State of nitc-curriculum-data

### What Has Real Data

| File | Status | Detail |
|---|---|---|
| `meta.json` | ✅ Complete | Institution metadata, academic system config, repo paths |
| `departments/ece/courses.json` | ✅ Rich | 25 core courses with syllabi, credits, prerequisites |
| `departments/ece/electives.json` | ✅ Rich | 72 electives across 4 categories |
| `departments/ece/department.json` | ⚠️ Partial | Code + name populated, HoD empty |
| `institute-core/courses.json` | ✅ Rich | 8 institute-core courses (MA, PH, BT, etc.) |
| `calendar/holidays.json` | ✅ Complete | 4 standing holidays + notes |

### What's Empty (Stubs Only)

| File | What's Missing |
|---|---|
| `departments/ece/faculty.json` | `"faculty": []` — zero entries |
| `departments/cse/*` | All stubs — empty courses, empty faculty |
| `departments/ce/*`, `che/*`, `eee/*`, `me/*` | Assumed empty (same pattern) |
| `calendar/2025-2026.json` | Semesters listed but all dates blank, no phases, no important_dates |
| `semesters/2025-monsoon/semester.json` | All dates blank, no batches |
| `semesters/2025-monsoon/offerings.json` | `"offerings": []` — zero entries |
| `semesters/2025-monsoon/timetable/slots.json` | `"slots": {}` — zero slot definitions |
| Per-batch timetable files | None exist (e.g., `ece-s5.json`, `cse-s7.json`) |

### Schema Mismatches (Spec vs. Reality)

| Field | SyllabiPlan.md Spec | Actual in Repo | Impact |
|---|---|---|---|
| `syllabus_modules` | Array of `{module, title, topics[], hours}` objects | Array of **flat strings** (topic summaries) | 🔴 Can't extract per-module hours or structured topics |
| `textbooks` | Populated array of strings | `[]` for all 25 ECE + 8 IC courses | 🟡 Empty but schema-correct |
| `description` | Populated string | Empty `""` for most courses | 🟡 Empty but schema-correct |
| `electives.json` | Not defined in spec | Exists with 72 entries, own schema | 🟡 Needs spec addition |
| Course code format | `"EC2001"` (no suffix) | `"EC2001E"` (E suffix) | 🟢 Cosmetic, both work |
| `semester.json` | Spec has `start_date`, `end_date` | Extra fields: `registration_open`, `registration_close`, `batches` | 🟢 Additive, not breaking |
| `curriculum_version` | Not in spec | Present in courses.json root | 🟢 Useful, should be spec'd |

---

## 2. Design Philosophy — Inspired by OHCNETWORK/care

### What care Does

OHCNETWORK/care solved the same problem for healthcare: **different hospitals need different intake forms, assessment forms, and reporting structures — without hardcoding any of them.**

Their approach:
1. **Questionnaire** (schema stored as a JSON tree in the DB, not code)
2. **QuestionnaireResponse** (filled data, validated against the schema at submission time)
3. **Templates** (pre-filled answers scoped to organizations)
4. **System registry** (hardcoded escape hatch for universal fields that all facilities share)

### What Changala Should Steal

| Care Concept | Changala Analogue |
|---|---|
| Questionnaire (form schema) | **Institution Schema** — "what does a course record look like at this institution?" |
| Question types (string, int, choice, group) | **Field types** for course/session metadata |
| QuestionnaireResponse (filled data) | **Course/session instances** created from the schema |
| Organization scoping | **Ring scoping** — each Ring defines its own schema extensions |
| System registry (universal fields) | **Core Changala fields** — `title`, `code`, `credits`, `semester` are universal |
| enable_when conditionals | **Conditional fields** — "if course is a lab, include `lab_hours`" |
| version + status lifecycle | **Schema versioning** — institutions can evolve their schemas across semesters |

### What Changala Should NOT Copy

care's full FHIR Questionnaire tree is overkill for course data. Course structures are more predictable than medical forms. Instead:

- **Don't build a generic form engine.** Course data has a known shape.
- **DO make the shape extensible.** Each institution adds fields beyond the core.
- **Store extension fields as JSON.** The Ring's `courses` table has a core schema + a `metadata JSONB` column for institution-specific extras.

### The Pattern: Core + Extensions

```
┌──────────────────────────────────────────────────────────────┐
│  CORE FIELDS (universal, Ring schema)                        │
│                                                              │
│  code, title, credits, semester, department, category        │
│  → Stored in typed Postgres columns                          │
│  → Indexed, queryable, sortable                              │
│  → Defined by Changala, same across all institutions         │
└──────────────────────────────────────────────────────────────┘
                          +
┌──────────────────────────────────────────────────────────────┐
│  EXTENSION FIELDS (institution-specific, from data repo)     │
│                                                              │
│  syllabus_modules, textbooks, references, faculty,           │
│  prerequisites, elective_type, slot, room, batch...          │
│  → Stored in `metadata JSONB` column                         │
│  → Shape defined by data repo's courses.json                 │
│  → Validated by MCP at load time, not by Ring schema         │
│  → Searchable via JSONB operators (post-MVP)                 │
└──────────────────────────────────────────────────────────────┘
```

This means:
- **The Ring never rejects data because it doesn't know the shape** — unknown fields go into JSONB
- **Different institutions CAN have different course shapes** — NITC has `syllabus_modules`, IIT might have `learning_outcomes`
- **Core fields are always consistent** — feeds, search, archival rely on core fields only
- **MCP validates against the data repo's schema** — the repo IS the schema definition

---

## 3. What You Need to Populate (Priority Order)

### P0 — Blocks everything (load a single department end-to-end)

1. **`calendar/2025-2026.json`** — fill in real dates for at least the monsoon semester
   - `start_date`, `end_date` for `2025-monsoon`
   - At minimum one `instruction` phase and one `exam` phase
   - A few `important_dates`

2. **`semesters/2025-monsoon/timetable/slots.json`** — define NITC's actual slot system
   - Named slots (A1, A2, B1, B2, C1, C2, D1, D2, E1, E2, F1, F2, etc.)
   - Each with `occurrences` (day + start/end times)
   - Lab slots (LAB_P1, LAB_P2, etc.)

3. **`semesters/2025-monsoon/offerings.json`** — which ECE courses run this semester
   - At least 5-6 offerings with `course_code`, `sections` (faculty, batch, strength)
   - This is the bridge between the curriculum catalog and live courses

4. **One batch timetable** — e.g., `semesters/2025-monsoon/timetable/ece-s5.json`
   - Maps 5-6 courses to slots for the S5 ECE batch

### P1 — Needed for realistic MCP demo

5. **`departments/ece/faculty.json`** — populate with at least 5-10 ECE faculty
   - name, email, designation, specializations

6. **`semesters/2025-monsoon/semester.json`** — fill in dates and at least one batch
   - `start_date`, `end_date` matching the calendar
   - `batches: [{"year": 2023, "semester_number": 5}, {"year": 2024, "semester_number": 3}]`

7. **Fix `syllabus_modules` format** (decision needed — see below)

### P2 — Multi-department validation

8. **Populate CSE** — at least `courses.json` and `department.json` with real data
9. **Populate one more department** (EEE or ME) to validate the pattern
10. **Cross-department offerings** for open electives

### P3 — Polish

11. **HoD fields** for all departments
12. **`SCHEMA.md`** — formal JSON Schema definitions
13. **`validate.py`** — validation script
14. **Template repo** — `changala-social/institution-data-template`

---

## 4. Schema Decision: `syllabus_modules` Format

**Current (actual data):**
```json
"syllabus_modules": [
  "Introduction to signals, classification, basic operations",
  "LTI systems, convolution, impulse response",
  "Fourier series, Fourier transform, properties"
]
```

**Spec'd (SyllabiPlan.md):**
```json
"syllabus_modules": [
  {
    "module": 1,
    "title": "Introduction to Signals",
    "topics": ["Classification of signals", "Basic operations", "Elementary signals"],
    "hours": 8
  }
]
```

**Recommendation: Accept both. Let the data repo decide.**

The MCP loader should handle both formats:
- If `syllabus_modules[0]` is a string → flat format, store as-is in JSONB
- If `syllabus_modules[0]` is an object → structured format, store as-is in JSONB

The Ring doesn't need to understand the internal structure — it's stored in the `metadata` JSONB column. The frontend/client renders based on what it finds. The data repo maintainer can upgrade from flat to structured over time without breaking anything.

This is the care-inspired insight: **validate at load time, store flexibly, render adaptively.**

---

## 5. Schema Addition: `electives.json`

This file exists in the repo but isn't spec'd. Add to SyllabiPlan.md:

```json
{
  "department": "ECE",
  "elective_categories": [
    {
      "category": "signal_processing_machine_learning",
      "display_name": "Signal Processing and Machine Learning",
      "courses": [
        {
          "code": "EC4031E",
          "title": "Digital Image Processing",
          "credits": { "lecture": 3, "tutorial": 0, "practical": 0, "total": 3 }
        }
      ]
    }
  ]
}
```

Electives are just courses with a category grouping. The MCP loader treats them the same as core courses when creating Ring course records — the `category` field distinguishes them.

---

## 6. Loading Flow — How It Actually Works

### Step 1: MCP Reads the Data Repo

```
Admin: "Set up from https://github.com/tellmeY18/nitc-curriculum-data"

MCP: [fetches meta.json]
     → Discovers institution: NITC
     → Discovers repo layout: calendar/, departments/, semesters/
     → Lists available semesters: 2025-monsoon
     → Lists departments: ece, cse, ce, che, eee, me
     → "NITC Ring. I found 1 semester (2025-monsoon) and 6 departments.
        Only ECE has populated course data (25 core + 72 electives).
        What would you like to load?"
```

### Step 2: Load Courses (Curriculum → Ring)

```
Admin: "Load ECE courses for 2025-monsoon"

MCP: [fetches departments/ece/courses.json]
     [fetches semesters/2025-monsoon/offerings.json]
     → "offerings.json is empty. Without offerings, I can create the
        course catalog but can't assign faculty or sections.
        Shall I create 25 ECE courses + 8 institute-core courses
        as the catalog for this semester?"

Admin: "Yes, create them."

MCP: [calls create_course × 33]
     → Core fields: code, title, credits, department, semester
     → Extension fields: syllabus_modules, prerequisites, references → metadata JSONB
     → "Created 33 courses (25 ECE + 8 IC). No faculty assigned yet
        (offerings.json is empty). Fill offerings.json to assign faculty."
```

### Step 3: Load Timetable → Sessions

```
Admin: "Load timetable for ECE S5"

MCP: [fetches timetable/slots.json] → "Slot definitions are empty."
     → "I can't create sessions without slot definitions.
        Please populate semesters/2025-monsoon/timetable/slots.json
        with NITC's slot system, then try again."
```

This is the **human-in-the-loop** pattern. The MCP server doesn't silently fail — it tells the admin exactly what's missing and what to fix.

### Step 4: Timetable (After Data is Populated)

```
Admin: "I've updated slots.json and added ece-s5.json. Try again."

MCP: [fetches slots.json] → 12 slot definitions found
     [fetches ece-s5.json] → 6 course-to-slot mappings
     [fetches calendar/2025-2026.json] → monsoon: Jul 21 – Nov 22
     [fetches holidays.json] → 4 standing holidays
     → "I'll create sessions for 6 courses across 18 instruction weeks.
        Holidays and exam periods excluded.
        Total: ~210 sessions. Preview?"

Admin: "Show me the first week."

MCP: [shows Mon Jul 21 – Fri Jul 25 session grid]
     → "Looks right?"

Admin: "Yes, create them all."

MCP: [calls create_session × 210]
```

---

## 7. Implementation Sequence

### Phase A: Fix the Data (You, in the repo)

- [ ] Populate `calendar/2025-2026.json` with real monsoon semester dates + phases
- [ ] Populate `semesters/2025-monsoon/timetable/slots.json` with NITC slot system
- [ ] Populate `semesters/2025-monsoon/offerings.json` with ECE S5 or S7 offerings
- [ ] Create at least one batch timetable (e.g., `ece-s5.json`)
- [ ] Populate `departments/ece/faculty.json` with ≥5 faculty
- [ ] Fill `semesters/2025-monsoon/semester.json` dates + batches
- [ ] Decision: keep flat `syllabus_modules` or restructure to objects? (Rec: keep flat, accept both)

### Phase B: Add `metadata JSONB` to Ring Schema

- [ ] New Ring migration: `ALTER TABLE courses ADD COLUMN metadata JSONB DEFAULT '{}'`
- [ ] Update `create_course` handler to accept + store extension fields
- [ ] Update `get_course` handler to return `metadata` in response
- [ ] Aggregator: index `metadata` for search (ILIKE on `metadata::text` for MVP)

### Phase C: MCP Bulk Loading Tools

- [ ] `load_institution_meta` — fetch + display `meta.json` summary
- [ ] `load_courses_from_repo` — fetch `courses.json` + `electives.json` → `create_course × N`
- [ ] `load_timetable_from_repo` — fetch slots + timetable + calendar → `create_session × N`
- [ ] `preview_load` — dry-run showing what would be created
- [ ] `semester_status` — summary of what's loaded vs. what's missing
- [ ] `load_calendar_from_repo` — display calendar boundaries (not stored in Ring)

### Phase D: Validation & Template

- [ ] `SCHEMA.md` with formal JSON Schema definitions (in data repo)
- [ ] `validate.py` schema validation script (in data repo)
- [ ] `changala-social/institution-data-template` repo with examples
- [ ] `CONTRIBUTING.md` guide for data repo maintainers

### Phase E: Cross-Department Validation

- [ ] Populate CSE in the data repo
- [ ] Load both ECE + CSE via MCP into staging Ring
- [ ] Test open elective handling (cross-department offerings)
- [ ] Test `getWeeklySchedule` aggregator endpoint with real timetable data

---

## 8. Open Decisions

| Decision | Options | Recommendation |
|---|---|---|
| `syllabus_modules` format | A) Restructure to objects B) Accept flat strings C) Accept both | **C — Accept both.** Store in JSONB. Let institutions evolve at their pace. |
| Slot field on sessions | A) Encode in `topic` B) Add optional `slot` field to session lexicon | **B — Add `slot` field.** Clean for queries, cheap to add. |
| `electives.json` schema | A) Merge into `courses.json` B) Keep separate with `elective_categories` | **B — Keep separate.** Different access pattern (browse by category). |
| Extension field storage | A) Typed columns for common fields B) Single `metadata JSONB` C) Both | **C — Core typed + JSONB overflow.** Best of both worlds. |
| Validation responsibility | A) Ring validates B) MCP validates C) Data repo CI validates | **C then B.** Repo has `validate.py`, MCP validates before calling Ring. Ring trusts MCP. |

---

*Gap analysis generated May 2026.*
*Data repo: `tellmeY18/nitc-curriculum-data` (main branch)*
*Spec reference: `SyllabiPlan.md` in changala monorepo*
