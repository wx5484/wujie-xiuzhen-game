create table guilds (
  id bigserial primary key,
  name text not null unique,
  notice text not null default '',
  level integer not null default 1,
  funds bigint not null default 0,
  sabak_owner boolean not null default false,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table guild_members (
  guild_id bigint not null references guilds(id) on delete cascade,
  character_id bigint not null references characters(id) on delete cascade,
  role text not null default 'member' check (role in ('leader', 'elder', 'member')),
  contribution bigint not null default 0,
  joined_at timestamptz not null default now(),
  primary key (guild_id, character_id)
);

create table guild_applications (
  id bigserial primary key,
  guild_id bigint not null references guilds(id) on delete cascade,
  character_id bigint not null references characters(id) on delete cascade,
  message text not null default '',
  status text not null default 'pending' check (status in ('pending', 'accepted', 'rejected')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  unique (guild_id, character_id, status)
);

create trigger guilds_set_updated_at before update on guilds
  for each row execute function set_updated_at();
create trigger guild_applications_set_updated_at before update on guild_applications
  for each row execute function set_updated_at();
