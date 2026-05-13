-- Semester calendar: instructional phases, exam periods, holidays.
-- Loaded from the institution's data repo via loadCalendar endpoint.
-- Used by provisionSessions to compute valid class dates.

CREATE TABLE IF NOT EXISTS semester_phases (
    id BIGSERIAL PRIMARY KEY,
    semester TEXT NOT NULL,
    name TEXT NOT NULL,
    label TEXT NOT NULL,
    phase_type TEXT NOT NULL,        -- 'instructional', 'exam', 'festival', 'administrative'
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    notes TEXT,
    UNIQUE(semester, name)
);
CREATE INDEX IF NOT EXISTS idx_semester_phases_semester ON semester_phases(semester);
CREATE INDEX IF NOT EXISTS idx_semester_phases_type ON semester_phases(phase_type);

CREATE TABLE IF NOT EXISTS semester_holidays (
    id BIGSERIAL PRIMARY KEY,
    semester TEXT NOT NULL,
    holiday_date DATE NOT NULL,
    name TEXT NOT NULL,
    holiday_type TEXT,               -- 'national', 'institutional', 'moon_dependent', etc.
    UNIQUE(semester, holiday_date)
);
CREATE INDEX IF NOT EXISTS idx_semester_holidays_semester ON semester_holidays(semester);

-- Timetable slot definitions: named time blocks mapped to weekly schedules.
-- Institution-specific (NITC uses A1-H, labs PA1-TB2, etc.)
-- Loaded from the data repo, used by provisionSessions.

CREATE TABLE IF NOT EXISTS timetable_slots (
    id BIGSERIAL PRIMARY KEY,
    semester TEXT NOT NULL,
    slot_name TEXT NOT NULL,
    slot_type TEXT NOT NULL DEFAULT 'theory',  -- 'theory' or 'lab'
    slot_system TEXT,                           -- '1', '2', 'common' — institution-specific
    UNIQUE(semester, slot_name)
);
CREATE INDEX IF NOT EXISTS idx_timetable_slots_semester ON timetable_slots(semester);

CREATE TABLE IF NOT EXISTS slot_occurrences (
    id BIGSERIAL PRIMARY KEY,
    slot_id BIGINT NOT NULL REFERENCES timetable_slots(id) ON DELETE CASCADE,
    day_of_week TEXT NOT NULL,        -- 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'
    start_time TIME NOT NULL,
    end_time TIME NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_slot_occurrences_slot ON slot_occurrences(slot_id);
