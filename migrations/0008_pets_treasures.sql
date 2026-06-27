create table pet_templates (
  id text primary key,
  name text not null,
  rarity text not null default 'common',
  base_hp bigint not null default 1,
  base_atk bigint not null default 1,
  skills jsonb not null default '[]'::jsonb,
  version bigint not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table pets (
  id bigserial primary key,
  character_id bigint not null references characters(id) on delete cascade,
  template_id text not null references pet_templates(id),
  name text not null,
  level integer not null default 1,
  exp bigint not null default 0,
  fighting boolean not null default false,
  skills jsonb not null default '[]'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table treasure_templates (
  id text primary key,
  name text not null,
  family text not null,
  passive text not null,
  config jsonb not null default '{}'::jsonb,
  version bigint not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table treasures (
  id bigserial primary key,
  character_id bigint not null references characters(id) on delete cascade,
  template_id text not null references treasure_templates(id),
  level integer not null default 1,
  stage integer not null default 0,
  equipped boolean not null default false,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
