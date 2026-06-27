-- 0.2.9: pending adventures and one-hour adventure buffs.

create table if not exists character_adventures (
  id bigserial primary key,
  character_id bigint not null references characters(id) on delete cascade,
  script_id text not null,
  title text not null,
  body text not null,
  options jsonb not null default '[]'::jsonb,
  status text not null default 'pending' check (status in ('pending', 'resolved', 'expired')),
  triggered_by text not null default 'unknown',
  zone text not null default '',
  room text not null default '',
  outcome jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  resolved_at timestamptz
);

create index if not exists character_adventures_pending_idx
  on character_adventures (character_id, status, created_at desc);

create table if not exists character_adventure_buffs (
  id bigserial primary key,
  character_id bigint not null references characters(id) on delete cascade,
  source_adventure_id bigint references character_adventures(id) on delete set null,
  stat text not null check (stat in ('atk', 'hp', 'def')),
  pct integer not null check (pct between -90 and 500),
  expires_at timestamptz not null,
  created_at timestamptz not null default now()
);

create index if not exists character_adventure_buffs_active_idx
  on character_adventure_buffs (character_id, expires_at);
