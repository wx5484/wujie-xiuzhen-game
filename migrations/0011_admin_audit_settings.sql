create table admin_audit_logs (
  id bigserial primary key,
  admin_account_id bigint references accounts(id) on delete set null,
  action text not null,
  target text not null,
  detail jsonb not null default '{}'::jsonb,
  ip inet,
  created_at timestamptz not null default now()
);

create table game_settings (
  key text primary key,
  value jsonb not null,
  version bigint not null default 1,
  updated_by bigint references accounts(id) on delete set null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table invite_relations (
  id bigserial primary key,
  inviter_account_id bigint not null references accounts(id) on delete cascade,
  invitee_account_id bigint not null references accounts(id) on delete cascade,
  reward_claimed boolean not null default false,
  created_at timestamptz not null default now(),
  unique (invitee_account_id)
);

create trigger game_settings_set_updated_at before update on game_settings
  for each row execute function set_updated_at();
