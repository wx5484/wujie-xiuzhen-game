create table activities (
  id bigserial primary key,
  code text not null unique,
  name text not null,
  enabled boolean not null default false,
  config jsonb not null default '{}'::jsonb,
  starts_at timestamptz,
  ends_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table activity_points (
  character_id bigint not null references characters(id) on delete cascade,
  activity_code text not null references activities(code),
  points bigint not null default 0,
  updated_at timestamptz not null default now(),
  primary key (character_id, activity_code)
);

create table vip_records (
  id bigserial primary key,
  character_id bigint not null references characters(id) on delete cascade,
  tier text not null check (tier in ('vip', 'svip', 'permanent_svip')),
  starts_at timestamptz not null default now(),
  ends_at timestamptz,
  created_at timestamptz not null default now()
);

create table recharge_cards (
  id bigserial primary key,
  code text not null unique,
  yuanbao bigint not null default 0,
  used_by bigint references characters(id) on delete set null,
  used_at timestamptz,
  created_at timestamptz not null default now()
);
