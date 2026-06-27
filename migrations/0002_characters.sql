create table characters (
  id bigserial primary key,
  account_id bigint not null references accounts(id) on delete cascade,
  name text not null unique,
  class text not null check (class in ('warrior', 'mage', 'taoist', 'assassin')),
  level integer not null default 1,
  exp bigint not null default 0,
  gold bigint not null default 0,
  yuanbao bigint not null default 0,
  power bigint not null default 0,
  deleted_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table character_stats (
  character_id bigint primary key references characters(id) on delete cascade,
  "str" bigint not null default 0,
  dex bigint not null default 0,
  "int" bigint not null default 0,
  con bigint not null default 0,
  spirit bigint not null default 0,
  max_hp bigint not null default 1,
  max_mp bigint not null default 0,
  atk bigint not null default 0,
  def bigint not null default 0,
  mag bigint not null default 0,
  mdef bigint not null default 0,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table character_state (
  character_id bigint primary key references characters(id) on delete cascade,
  zone text not null,
  room text not null,
  hp bigint not null default 1,
  mp bigint not null default 0,
  online boolean not null default false,
  temp_state jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create trigger characters_set_updated_at before update on characters
  for each row execute function set_updated_at();
create trigger character_stats_set_updated_at before update on character_stats
  for each row execute function set_updated_at();
create trigger character_state_set_updated_at before update on character_state
  for each row execute function set_updated_at();
