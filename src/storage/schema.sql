CREATE TABLE questions (
  id TEXT PRIMARY KEY,
  discipline TEXT NOT NULL,
  block_type TEXT NOT NULL,
  question_type TEXT NOT NULL,
  question TEXT NOT NULL,
  options_json TEXT NOT NULL,
  correct_answers_json TEXT NOT NULL,
  correct_text TEXT,
  source_hash TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE attempts (
  id TEXT PRIMARY KEY,
  question_id TEXT NOT NULL,
  selected_answers_json TEXT NOT NULL,
  user_text_answer TEXT,
  is_correct INTEGER NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE mistakes (
  question_id TEXT PRIMARY KEY,
  mistake_count INTEGER NOT NULL DEFAULT 0,
  correct_streak INTEGER NOT NULL DEFAULT 0,
  last_mistake_at TEXT,
  status TEXT NOT NULL DEFAULT 'learning'
);

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
