create table accounts (
  id bigserial primary key,
  username text not null unique,
  password_hash text not null,
  email text unique,
  status text not null default 'active' check (status in ('active', 'muted', 'banned')),
  banned_reason text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table sessions (
  id bigserial primary key,
  account_id bigint not null references accounts(id) on delete cascade,
  token text not null unique,
  expires_at timestamptz not null,
  revoked_at timestamptz,
  device text,
  ip inet,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table login_failures (
  id bigserial primary key,
  username text not null,
  ip inet,
  failed_at timestamptz not null default now()
);

create or replace function set_updated_at()
returns trigger language plpgsql as $$
begin
  new.updated_at = now();
  return new;
end $$;

create trigger accounts_set_updated_at before update on accounts
  for each row execute function set_updated_at();

create trigger sessions_set_updated_at before update on sessions
  for each row execute function set_updated_at();
