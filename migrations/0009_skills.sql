create table skills (
  id text primary key,
  name text not null,
  class text not null,
  min_level integer not null default 1,
  mp_cost bigint not null default 0,
  cooldown_ms integer not null default 1000,
  config jsonb not null default '{}'::jsonb,
  version bigint not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table character_skills (
  character_id bigint not null references characters(id) on delete cascade,
  skill_id text not null references skills(id),
  level integer not null default 1,
  proficiency bigint not null default 0,
  learned_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (character_id, skill_id)
);
