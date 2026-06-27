create table sabak_campaigns (
  id bigserial primary key,
  signup_starts_at timestamptz not null,
  battle_starts_at timestamptz not null,
  battle_ends_at timestamptz not null,
  defending_guild_id bigint references guilds(id) on delete set null,
  winner_guild_id bigint references guilds(id) on delete set null,
  tax_rate_pct integer not null default 5,
  status text not null default 'scheduled' check (status in ('scheduled', 'signup', 'battle', 'settled', 'cancelled')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table sabak_signups (
  campaign_id bigint not null references sabak_campaigns(id) on delete cascade,
  guild_id bigint not null references guilds(id) on delete cascade,
  paid_gold bigint not null default 0,
  created_at timestamptz not null default now(),
  primary key (campaign_id, guild_id)
);

create trigger sabak_campaigns_set_updated_at before update on sabak_campaigns
  for each row execute function set_updated_at();
