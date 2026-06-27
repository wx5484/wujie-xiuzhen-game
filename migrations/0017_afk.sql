create table afk_sessions (
  character_id bigint primary key references characters(id) on delete cascade,
  active boolean not null default false,
  started_at timestamptz,
  last_settled_at timestamptz,
  exp_per_minute bigint not null default 0,
  gold_per_minute bigint not null default 0,
  state jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table afk_reward_logs (
  id bigserial primary key,
  character_id bigint not null references characters(id) on delete cascade,
  exp bigint not null default 0,
  gold bigint not null default 0,
  detail jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now()
);

create trigger afk_sessions_set_updated_at before update on afk_sessions
  for each row execute function set_updated_at();
