create table game_config_versions (
  id bigserial primary key,
  area text not null,
  version bigint not null,
  payload jsonb not null default '{}'::jsonb,
  active boolean not null default false,
  published_by bigint references accounts(id) on delete set null,
  published_at timestamptz,
  created_at timestamptz not null default now(),
  unique (area, version)
);

insert into game_settings (key, value, version) values
  ('room_variant_count', '5', 1),
  ('registration_mode', '"open"', 1),
  ('drop_rate_multiplier', '1', 1),
  ('max_online_per_account', '1', 1)
on conflict (key) do nothing;
