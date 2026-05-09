# Changala — Institution Data Bootstrapping

> How courses, timetables, academic calendars, and institutional structure
> flow into a Ring from a single, institution-owned data repository.
> *Institution-agnostic by design. NITC is the first proving ground.*

---

## 0. The Problem

A fresh Ring is empty. Before any social learning can happen, it needs:

1. **Courses** — the full catalog for the current semester
2. **Timetable** — which courses run on which days, in which slots
3. **Sessions** — individual class periods, pre-created from the timetable
4. **Enrollments** — which students are in which courses
5. **Class reps** — who manages each course's session lifecycle

Today this is a manual process. An admin logs in, creates courses one by
one, assigns reps, hopes someone opens sessions on time. This doesn't
scale to 50+ courses per department, 10+ departments, 6 semesters of
active batches.

---

## 1. Design Principles

### 1.1 Institution-Agnostic

Every institution has different:
- Semester naming (`Fall 2026`, `2025-monsoon`, `S7`, `VII Semester`)
- Department structure (some have sub-departments, some don't)
- Timetable slot systems (NITC has named slots A1–F2; IITs use hourly blocks)
- Credit systems (NITC uses L-T-P; others vary)
- Elective structures (open electives, department electives, minors)

**We do not hardcode any of this.** The Ring accepts structured data and
the structure itself is defined externally.

### 1.2 One Repo Per Institution

Every institution running a Ring maintains **one data repository** — their
single source of truth for all academic structure. This repo contains
curriculum data, academic calendars, timetable definitions, and semester
offerings. It is the institutional equivalent of infrastructure-as-code.

- Curriculum PDFs → parsed to JSON → committed to the repo
- Academic calendar PDFs → parsed to JSON → committed to the repo
- Timetable slot definitions → JSON → committed
- Anyone can propose changes via PR
- History is preserved across semesters and academic years
- The repo lives on GitHub (or any Git provider) — institution-owned

The repo URL is the **identity** of the institution's academic data.
When setting up a Ring, the admin points MCP at this repo and everything
flows in.

### 1.3 MCP-First

The MCP server is the bridge between external data and the Ring. Instead
of building custom import endpoints, we use MCP tool calls to load data:

```
Human: "Load all ECE courses for S7 from the curriculum repo"
AI:    [reads JSON from repo] → [calls create_course × N]
```

This is not a one-time import. The MCP server is the ongoing interface
for academic administrators.

### 1.4 Separation of Concerns

```
┌─────────────────────────────────────────────────────────────────┐
│                    EXTERNAL (Git repos)                         │
│                                                                 │
│  ┌───────────────────┐  ┌──────────────────┐  ┌─────────────┐  │
│  │ Curriculum Data   │  │ Timetable Logic  │  │ Slot Defs   │  │
│  │                   │  │                  │  │             │  │
│  │ courses.json      │  │ rules.json       │  │ slots.json  │  │
│  │ electives.json    │  │ (institution-    │  │ (named time │  │
│  │ faculty.json      │  │  specific)       │  │  blocks)    │  │
│  └────────┬──────────┘  └────────┬─────────┘  └──────┬──────┘  │
└───────────┼──────────────────────┼───────────────────┼──────────┘
            │                      │                   │
            ▼                      ▼                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                    MCP SERVER (bridge)                          │
│                                                                 │
│  Reads external data → validates → calls Ring XRPC endpoints    │
│  Human-in-the-loop: admin confirms before bulk operations       │
└─────────────────────────────────────┬───────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RING (source of truth)                       │
│                                                                 │
│  Courses, sessions, enrollments, timetable records              │
│  (stored in Postgres, served via XRPC, indexed by Aggregator)   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. External Data: Curriculum Repository

### 2.1 Repository Structure

Each institution maintains a curriculum data repo. The canonical format:

```
nitc-data/                            # One repo per institution
├── meta.json                         # Institution metadata
├── calendar/
│   ├── 2026-2027.json                # Academic calendar for the year
│   ├── 2025-2026.json                # Previous year (history preserved)
│   └── holidays.json                 # Standing holidays (Republic Day, etc.)
├── departments/
│   ├── cse/
│   │   ├── department.json           # Department info
│   │   ├── courses.json              # All courses offered by CSE
│   │   ├── electives.json            # Elective offerings
│   │   └── faculty.json              # Faculty directory
│   ├── ece/
│   │   ├── department.json
│   │   ├── courses.json
│   │   ├── electives.json
│   │   └── faculty.json
│   └── ...
├── semesters/
│   ├── 2026-monsoon/
│   │   ├── semester.json             # Semester metadata (dates, batches)
│   │   ├── offerings.json            # Which courses are offered this sem
│   │   └── timetable/
│   │       ├── slots.json            # Slot definitions for this semester
│   │       ├── cse-s7.json           # CSE 7th sem timetable
│   │       ├── ece-s5.json           # ECE 5th sem timetable
│   │       └── ...
│   └── 2026-winter/
│       └── ...
└── README.md
```

### 2.2 Schema: `meta.json`

Institution-level metadata. Loaded once per Ring setup. Also declares
where the academic calendar and other data lives within the repo.

```json
{
  "institution": {
    "name": "National Institute of Technology Calicut",
    "short_name": "NITC",
    "domain": "nitc.ac.in",
    "ring_did": "did:web:nitc.changala.app",
    "timezone": "Asia/Kolkata"
  },
  "academic_system": {
    "type": "semester",
    "semesters_per_year": 2,
    "naming_convention": "{year}-{season}",
    "seasons": ["monsoon", "winter"]
  },
  "repo": {
    "url": "https://github.com/nitc-data/nitc-data",
    "calendar_path": "calendar/",
    "departments_path": "departments/",
    "semesters_path": "semesters/"
  }
}
```

### 2.3 Schema: `department.json`

```json
{
  "code": "ECE",
  "name": "Electronics and Communication Engineering",
  "hod": "Dr. Sameer S. M.",
  "email_prefix": "ece"
}
```

### 2.4 Schema: `courses.json`

The curriculum catalog — all courses a department has ever defined.
This is the **template**. Actual offerings per semester come from
`offerings.json`.

```json
{
  "department": "ECE",
  "courses": [
    {
      "code": "EC2001",
      "title": "Signals and Systems",
      "credits": { "lecture": 3, "tutorial": 1, "practical": 0, "total": 4 },
      "category": "core",
      "typical_semester": 3,
      "prerequisites": [],
      "description": "Continuous and discrete-time signals, LTI systems, Fourier analysis, Laplace and Z-transforms.",
      "syllabus_modules": [
        {
          "module": 1,
          "title": "Introduction to Signals",
          "topics": [
            "Classification of signals",
            "Basic operations on signals",
            "Elementary signals"
          ],
          "hours": 8
        },
        {
          "module": 2,
          "title": "LTI Systems",
          "topics": [
            "System properties",
            "Convolution",
            "Impulse response"
          ],
          "hours": 10
        }
      ],
      "textbooks": [
        "Oppenheim, Willsky — Signals and Systems",
        "Haykin, Van Veen — Signals and Systems"
      ],
      "references": [
        "Lathi — Linear Systems and Signals"
      ]
    },
    {
      "code": "EC2002",
      "title": "Analog Circuits",
      "credits": { "lecture": 3, "tutorial": 0, "practical": 1, "total": 4 },
      "category": "core",
      "typical_semester": 3,
      "prerequisites": ["EC1001"],
      "description": "BJT and MOSFET amplifiers, feedback, oscillators, operational amplifiers."
    }
  ]
}
```

### 2.5 Schema: `offerings.json`

What's actually being taught this semester. Maps curriculum courses to
faculty and batches.

```json
{
  "semester": "2026-monsoon",
  "offerings": [
    {
      "course_code": "EC2001",
      "sections": [
        {
          "section": "A",
          "faculty": ["Dr. Deepak Mishra"],
          "batch": "2024",
          "strength": 65
        }
      ]
    },
    {
      "course_code": "EC4099",
      "sections": [
        {
          "section": "A",
          "faculty": ["Dr. Lillykutty Jacob"],
          "batch": "2023",
          "strength": 40,
          "elective_type": "department"
        }
      ]
    }
  ]
}
```

### 2.6 Schema: `faculty.json`

```json
{
  "department": "ECE",
  "faculty": [
    {
      "name": "Dr. Deepak Mishra",
      "email": "deepak@nitc.ac.in",
      "designation": "Associate Professor",
      "specializations": ["Signal Processing", "Machine Learning"]
    }
  ]
}
```

---

## 3. External Data: Academic Calendar

Every institution publishes an academic calendar — usually as a PDF at
the start of each academic year. It defines the boundaries within which
everything else operates: when semesters start and end, when exams happen,
when holidays fall, when registration opens.

This calendar is parsed to JSON and committed to the data repo. The MCP
server uses it to:

- Set semester date boundaries when creating sessions
- Skip holidays and exam periods when generating sessions from timetables
- Know when keyword windows, archival, and rollover should happen
- Surface important dates in the frontend (post-MVP)

### 3.1 Schema: `calendar/2026-2027.json`

One file per academic year. Covers all semesters in that year.

```json
{
  "academic_year": "2026-2027",
  "institution": "NITC",
  "semesters": [
    {
      "name": "2026-monsoon",
      "display_name": "Monsoon Semester 2026",
      "start_date": "2026-07-21",
      "end_date": "2026-11-22",
      "phases": [
        {
          "type": "instruction",
          "label": "First half",
          "start": "2026-07-21",
          "end": "2026-09-12"
        },
        {
          "type": "exam",
          "label": "First periodical exam",
          "start": "2026-09-15",
          "end": "2026-09-20"
        },
        {
          "type": "instruction",
          "label": "Second half",
          "start": "2026-09-22",
          "end": "2026-11-07"
        },
        {
          "type": "exam",
          "label": "Second periodical exam",
          "start": "2026-11-09",
          "end": "2026-11-14"
        },
        {
          "type": "exam",
          "label": "End semester examination",
          "start": "2026-11-16",
          "end": "2026-11-22"
        }
      ],
      "important_dates": [
        { "date": "2026-07-14", "event": "Registration opens" },
        { "date": "2026-07-18", "event": "Late registration deadline" },
        { "date": "2026-08-15", "event": "Course add/drop deadline" },
        { "date": "2026-10-01", "event": "Course withdrawal deadline" }
      ]
    },
    {
      "name": "2026-winter",
      "display_name": "Winter Semester 2026-27",
      "start_date": "2027-01-06",
      "end_date": "2027-05-10",
      "phases": []
    }
  ]
}
```

### 3.2 Schema: `calendar/holidays.json`

Standing holidays that recur every year. Semester-specific holidays
(declared mid-year by the institution) go in the calendar year file's
`important_dates` with `"type": "holiday"`.

```json
{
  "institution": "NITC",
  "standing_holidays": [
    { "date_pattern": "01-26", "name": "Republic Day" },
    { "date_pattern": "08-15", "name": "Independence Day" },
    { "date_pattern": "10-02", "name": "Gandhi Jayanti" },
    { "date_pattern": "11-01", "name": "Kerala Piravi" }
  ],
  "notes": "Floating holidays (Onam, Eid, Diwali, etc.) vary by year and are declared in the annual calendar file."
}
```

### 3.3 How the Calendar Is Used

The calendar is **not loaded into the Ring as a record**. It's consumed
by the MCP server at session-creation time:

1. Admin says: "Create sessions for EC3001 in slot A1 for 2026-monsoon"
2. MCP reads `calendar/2026-2027.json` → finds monsoon semester runs
   July 21 – November 22
3. MCP reads `calendar/holidays.json` → finds standing holidays
4. MCP reads the semester's `phases` → filters out exam periods
   (only `"type": "instruction"` phases get sessions)
5. MCP expands slot A1 across all instruction-phase weeks, skipping
   holidays → generates session dates
6. MCP calls `create_session` for each date

The calendar is the **boundary definition**. The Ring stores the
resulting sessions. If the calendar is wrong, the admin corrects the
JSON in the repo, and re-runs the MCP load (deleting and recreating
sessions as needed).

---

## 4. External Data: Timetable

### 4.1 Why Timetable Logic Varies

| Institution | Slot system | Example |
|---|---|---|
| NITC | Named slots (A1, B1, C1...) mapped to fixed weekly times | A1 = Mon 8:00–8:50, Tue 10:00–10:50 |
| IIT Bombay | Hourly blocks, room-allocated | Slot 1 = Mon 8:30–9:25 |
| BITS Pilani | Hour-based, no named slots | Direct time mapping |
| Anna University | Period-based (I, II, III...) | Period I = 8:45–9:35 |

**We don't implement timetable generation.** That's a solved problem
(institutions already have timetables). We just consume the output.

### 4.2 Schema: `slots.json`

Slot definitions are institution-specific. This file defines the mapping
from slot names to actual weekly times.

```json
{
  "institution": "NITC",
  "semester": "2026-monsoon",
  "slot_duration_mins": 50,
  "break_duration_mins": 10,
  "slots": {
    "A1": {
      "occurrences": [
        { "day": "monday",    "start": "08:00", "end": "08:50" },
        { "day": "wednesday", "start": "10:00", "end": "10:50" },
        { "day": "friday",    "start": "08:00", "end": "08:50" }
      ]
    },
    "B1": {
      "occurrences": [
        { "day": "monday",    "start": "09:00", "end": "09:50" },
        { "day": "wednesday", "start": "11:00", "end": "11:50" },
        { "day": "thursday",  "start": "08:00", "end": "08:50" }
      ]
    },
    "LAB_P1": {
      "occurrences": [
        { "day": "tuesday", "start": "14:00", "end": "16:50" }
      ]
    }
  }
}
```

### 4.3 Schema: Timetable (`cse-s7.json`)

Maps courses to slots for a specific batch in a specific semester.

```json
{
  "semester": "2026-monsoon",
  "department": "CSE",
  "batch": "2023",
  "year": 4,
  "semester_number": 7,
  "timetable": [
    {
      "course_code": "CS4001",
      "slot": "A1",
      "room": "CC1"
    },
    {
      "course_code": "CS4002",
      "slot": "B1",
      "room": "CC2"
    },
    {
      "course_code": "CS4001L",
      "slot": "LAB_P1",
      "room": "CS Lab 1"
    }
  ]
}
```

---

## 5. Loading Flow: From Git to Ring

### 5.1 The MCP-Driven Workflow

There is no automated sync daemon. A human admin uses MCP (via AI
assistant) to drive data loading. This is intentional:

1. **Review before commit** — admin sees what will be created
2. **Incremental** — load one department, validate, then the next
3. **Correctable** — mistakes are caught in conversation, not after batch import
4. **Auditable** — every action goes through the Ring's audit log

### 5.2 Session 1: Semester Setup

```
Admin: "Set up 2026-monsoon semester for ECE department.
        Here's the data repo: github.com/nitc-data/nitc-data"

AI:    [fetches meta.json, departments/ece/courses.json, 
        semesters/2026-monsoon/offerings.json]
       
       "I found 24 ECE courses in the curriculum. 
        12 are offered this semester per offerings.json.
        Here's what I'll create:"
        
       | Code    | Title                  | Faculty           | Batch |
       |---------|------------------------|-------------------|-------|
       | EC3001  | Digital Signal Processing | Dr. Deepak Mishra | 2024  |
       | EC3002  | VLSI Design            | Dr. James Kurian  | 2024  |
       | ...     | ...                    | ...               | ...   |
       
       "Shall I proceed?"

Admin: "Yes, create them all."

AI:    [calls create_course × 12]
       "Created 12 courses. All assigned to ECE department, 
        semester 2026-monsoon."
```

### 5.3 Session 2: Timetable → Sessions

```
Admin: "Now load the timetable for ECE S5 batch."

AI:    [fetches semesters/2026-monsoon/timetable/ece-s5.json,
        semesters/2026-monsoon/timetable/slots.json,
        calendar/2026-2027.json,
        calendar/holidays.json]
       
       "The timetable maps 6 courses to slots. From the academic
        calendar, instruction runs July 21 – Sep 12 and Sep 22 –
        Nov 7 (exam periods excluded). I found 4 holidays in range.
        
        This will create:
        - EC3001 in slot A1 → 42 sessions (3×/week, instruction only)
        - EC3002 in slot B1 → 42 sessions
        - EC3001L in slot LAB_P1 → 14 sessions (1×/week)
        - ...
        
        Total: 236 sessions (exam weeks and holidays excluded).
        Shall I proceed?"

Admin: "Yes."

AI:    [calls create_session × 236]
       "Created 236 sessions. All in 'scheduled' status."
```

### 5.4 Session 3: Enrollments & Reps

```
Admin: "Enroll all 2024-batch ECE students. Their DIDs are in
        this spreadsheet: [paste/upload]"

AI:    [parses DID list]
       [calls enroll_student × N for each course]
       "Enrolled 62 students across 6 courses."

Admin: "Assign @alice.bsky.social as class rep for EC3001."

AI:    [resolves handle → DID]
       [calls assign_class_rep]
       "Done. Alice is now class rep for EC3001 Digital Signal Processing."
```

---

## 6. What the Ring Stores vs. What Stays External

| Data | Where it lives | Why |
|---|---|---|
| Course catalog (templates) | Data repo | Versioned, community-maintained, institution-owned |
| Syllabus modules/topics | Data repo | Reference material, doesn't change mid-semester |
| Faculty directory | Data repo | Informational, not a Ring concern |
| Academic calendar | Data repo | Boundary definitions — consumed at session-creation time |
| Holiday lists | Data repo | Standing + annual — consumed at session-creation time |
| Slot definitions | Data repo | Institution-specific, reused across semesters |
| Timetable mappings | Data repo | Semester-specific, reviewed before loading |
| **Active courses** | Ring | Runtime state, enrollments, session lifecycle |
| **Sessions** | Ring | Created from timetable + calendar, managed by class reps |
| **Enrollments** | Ring | Who's in what course |
| **Keywords, notes, votes** | Ring + PDS | The actual social learning data |
| **Archives** | Ring + archive.org | Sealed semester records |

The Ring is the **runtime** layer. The data repo is the **reference** layer.
The MCP server is the **bridge**.

---

## 7. Timetable on the Platform

### 7.1 What "Timetable" Means in Changala

The Ring doesn't have a first-class `timetable` record type. Instead,
the timetable is **materialised from sessions**:

- Each session has `scheduledAt` (datetime) and `courseUri`
- The Aggregator (or frontend) can reconstruct a weekly grid from all
  sessions for a batch
- The timetable is a **view**, not a stored entity

This is important because:
- Timetable changes mid-semester (rescheduled sessions, cancellations)
- The session-level granularity is needed anyway for keywords/notes
- A static timetable record would drift from reality

### 7.2 Timetable API (Aggregator)

The Aggregator should provide a convenience endpoint:

```
GET /xrpc/app.changala.globalview.getWeeklySchedule
  ?batch=2024
  &department=ECE
  &week=2026-W30
```

Returns a grid of sessions grouped by day and time slot. This is a
**read-only materialised view** — the source of truth is always the
individual session records on the Ring.

### 7.3 Slot Metadata

When sessions are created from a timetable, the slot name (e.g. "A1")
can be stored in the session's `topic` field or as a tag. This preserves
the institutional slot context without adding a slot-specific schema:

```json
{
  "courseUri": "at://did:web:nitc.changala.app/app.changala.course/...",
  "scheduledAt": "2026-07-21T08:00:00+05:30",
  "durationMins": 50,
  "topic": "[A1] Digital Signal Processing — Module 1"
}
```

Alternatively, a future schema extension could add an optional `slot`
field to the session record. But for MVP, encoding it in `topic` works.

---

## 8. Institution Adoption Pattern

### 8.1 How a New Institution Onboards

```
1. Deploy a Ring (Docker / k8s / bare metal)
2. Fork the data repo template → github.com/our-institution/our-data
3. Fill in:
   - meta.json (institution info, repo URL)
   - calendar/ (academic calendar PDFs → JSON)
   - departments/ (courses, faculty from curriculum PDFs → JSON)
   - semesters/ (current offerings, timetable)
4. Connect MCP (via Zed, Claude, or any MCP client)
5. "Set up our Ring from github.com/our-institution/our-data"
6. MCP reads meta.json, calendar, departments, offerings → loads everything
7. Students verify email → enroll → class reps open sessions → learning begins
```

### 8.2 Data Repo Template

We provide a **template repository** that institutions fork:

```
changala-social/institution-data-template/
├── meta.json.example
├── calendar/
│   ├── YYYY-YYYY.json.example      # Academic calendar template
│   └── holidays.json.example       # Standing holidays template
├── departments/
│   └── _template/
│       ├── department.json.example
│       ├── courses.json.example
│       ├── electives.json.example
│       └── faculty.json.example
├── semesters/
│   └── _template/
│       ├── semester.json.example
│       ├── offerings.json.example
│       └── timetable/
│           ├── slots.json.example
│           └── _batch-template.json.example
├── SCHEMA.md              # Full JSON schema documentation
├── CONTRIBUTING.md         # How to add/update data
└── validate.py            # Schema validation script
```

### 8.3 The Data Repo Maintainer Role

Within each institution, curriculum data is maintained by:
- **Departmental volunteers** (students who care about accuracy)
- **Class representatives** (already have elevated access)
- **Faculty** (post-MVP, when faculty verification exists)

Changes go through Git PRs. The data is reviewed before being loaded
into the Ring. This mirrors how Wikipedia works — open contribution,
reviewed changes, versioned history.

---

## 9. MCP Extensions for Data Loading

### 9.1 New MCP Tools Needed

The current MCP tools (`create_course`, `create_session`, etc.) are
sufficient for one-at-a-time operations. For bulk loading, we add
higher-level tools:

| Tool | Description |
|---|---|
| `load_courses_from_json` | Parse a courses.json and create all courses |
| `load_timetable` | Parse slots + timetable + calendar JSON, create sessions (auto-skipping holidays/exams) |
| `bulk_enroll` | Enroll a list of DIDs into a course |
| `preview_load` | Dry-run: show what would be created without creating it |
| `semester_status` | Summary of what's loaded for a semester (counts, gaps) |
| `load_calendar` | Parse academic calendar JSON and display semester boundaries, holidays, exam periods |

These tools don't need new Ring endpoints — they compose existing ones.
The MCP server reads JSON (from a URL, file, or inline), validates it
against the schema, and calls `create_course` / `create_session` in a
loop.

### 9.2 Fetch-Based Loading

The MCP server already has `reqwest` as a dependency. It can fetch JSON
directly from a Git repo's raw URL:

```
"Set up from https://github.com/nitc-data/nitc-data"
```

The MCP server reads `meta.json` first (to discover repo structure),
then fetches calendar, department, and semester files as needed. No
need to clone repos or manage local state.

---

## 10. What This Enables

### 10.1 Day-One Experience

A student at NITC installs Changala. Day one:
- All their courses are already there
- Sessions are pre-scheduled for the entire semester
- They can see today's classes, what's coming up
- When the class rep opens a session, they can submit keywords immediately

No bootstrapping friction. The knowledge trail starts from day one.

### 10.2 Cross-Institution Discovery

When multiple institutions adopt Changala:
- The Aggregator indexes courses from all Rings
- A student at IIT Bombay can browse NITC's archived CS301 notes
- Data repos become a public directory of what's taught where
- Academic calendars reveal when semesters align across institutions
- The brain layer connects knowledge across institutional boundaries

### 10.3 Semester Rollover

Each semester:
1. Update `calendar/` with the new academic year (if needed)
2. Update `offerings.json` in the data repo
3. Add new timetable files
4. MCP: "Set up 2027-winter semester from the updated repo"
5. Previous semester's data → archived (existing archive pipeline)

The data repo accumulates history. The Ring resets per semester
(new courses, new sessions). The archive preserves everything.

---

## 11. Non-Goals

| Item | Why not |
|---|---|
| Auto-sync from Git | Human review is essential. MCP gives the human control. |
| Timetable generation | Solved problem. Institutions already have timetables. We consume, not generate. |
| Room allocation | Facility management is out of scope. |
| Attendance tracking | Changala is about knowledge, not surveillance. |
| Exam scheduling | Separate concern. |
| Grade management | LMS territory. Changala is explicitly not an LMS. |
| Faculty-facing features (MVP) | Faculty verification doesn't exist yet. Post-MVP. |

---

## 12. Implementation Sequence

1. **Define JSON schemas** — write `SCHEMA.md` with full JSON Schema definitions (including calendar)
2. **Create template repo** — `changala-social/institution-data-template`
3. **Populate NITC** — create `nitc-data/nitc-data` repo with real data:
   - Parse academic calendar PDF → `calendar/2026-2027.json` + `holidays.json`
   - Parse ECE curriculum PDF → `departments/ece/courses.json`
   - Add ECE timetable and slot definitions
4. **Add MCP bulk tools** — `load_courses_from_json`, `load_timetable`, `load_calendar`, `bulk_enroll`, `preview_load`
5. **Test end-to-end** — load ECE 2026-monsoon via MCP into a staging Ring (calendar-aware session generation)
6. **Add Aggregator timetable endpoint** — `getWeeklySchedule` materialised view
7. **Repeat for CSE, ME, EE** — validate the pattern works across departments
8. **Document the adoption guide** — how any institution forks the template and fills it in

---

## 13. Open Questions

- **Elective handling**: How do we represent courses where students from
  multiple departments/batches mix? Current model is department-bound.
  Likely solution: offerings.json supports `cross_listed: true` with
  multiple batch entries.

- **Mid-semester changes**: What happens when a faculty changes, a course
  is added/dropped, or slots are swapped? The MCP approach handles this
  naturally (admin asks the AI to make the change), but we should define
  the expected workflow.

- **Slot field on sessions**: Is encoding the slot in `topic` sufficient,
  or should we add an optional `slot` field to the session lexicon?
  Topic encoding is simpler; a dedicated field is cleaner for queries.

- **Calendar granularity**: Should `phases` support sub-types beyond
  `instruction` and `exam`? E.g. `lab-week`, `project-review`,
  `remedial`. For MVP, `instruction` and `exam` are sufficient.

- **Calendar-driven automation**: Should the MCP server proactively
  suggest actions based on calendar dates? E.g. "Registration opens
  tomorrow — have you set up courses for 2027-winter?" This is a
  post-MVP enhancement (notifications + MCP prompts).

- **Validation**: Should the MCP server validate JSON against schemas
  before loading, or trust the data? Validation is safer, but adds
  complexity. Minimum: validate required fields before calling Ring
  endpoints.

---

*First draft — May 2026. Living document.*
*Designed to be institution-agnostic from day one.*
*One repo per institution. NITC is the first to prove the pattern.*
