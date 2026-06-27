create table ranking_snapshots (
  id bigserial primary key,
  kind text not null,
  season text not null,
  entries jsonb not null default '[]'::jsonb,
  settled_at timestamptz not null default now(),
  created_at timestamptz not null default now(),
  unique (kind, season)
);

create table ranking_rewards (
  id bigserial primary key,
  snapshot_id bigint not null references ranking_snapshots(id) on delete cascade,
  character_id bigint not null references characters(id) on delete cascade,
  rank integer not null,
  reward jsonb not null default '{}'::jsonb,
  mailed_at timestamptz,
  created_at timestamptz not null default now()
);
