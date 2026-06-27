-- 0.2.10: per-skill automatic cast toggle for learned active skills.

alter table character_skills
  add column if not exists auto_enabled boolean not null default true;

update character_skills
set auto_enabled = true
where auto_enabled is null;
