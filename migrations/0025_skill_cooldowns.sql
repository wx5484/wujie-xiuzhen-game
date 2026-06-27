alter table character_skills
  add column if not exists last_used_at timestamptz;
