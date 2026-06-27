create table world_zones (
  id text primary key,
  name text not null,
  config jsonb not null default '{}'::jsonb,
  version bigint not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table world_rooms (
  zone_id text not null references world_zones(id) on delete cascade,
  id text not null,
  name text not null,
  description text not null default '',
  exits jsonb not null default '{}'::jsonb,
  spawns jsonb not null default '[]'::jsonb,
  safe boolean not null default false,
  version bigint not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (zone_id, id)
);

create table mob_templates (
  id text primary key,
  name text not null,
  level integer not null default 1,
  max_hp bigint not null default 1,
  atk bigint not null default 0,
  def bigint not null default 0,
  exp bigint not null default 0,
  gold bigint not null default 0,
  boss boolean not null default false,
  respawn_seconds integer not null default 60,
  drops jsonb not null default '[]'::jsonb,
  version bigint not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table mob_spawns (
  id bigserial primary key,
  template_id text not null references mob_templates(id),
  zone_id text not null,
  room_id text not null,
  hp bigint not null default 1,
  alive boolean not null default true,
  respawn_at timestamptz,
  hate jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create trigger world_zones_set_updated_at before update on world_zones
  for each row execute function set_updated_at();
create trigger world_rooms_set_updated_at before update on world_rooms
  for each row execute function set_updated_at();
create trigger mob_templates_set_updated_at before update on mob_templates
  for each row execute function set_updated_at();
create trigger mob_spawns_set_updated_at before update on mob_spawns
  for each row execute function set_updated_at();
