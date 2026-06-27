create table rate_limit_buckets (
  id bigserial primary key,
  bucket_key text not null,
  scope text not null,
  points integer not null default 0,
  resets_at timestamptz not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  unique (bucket_key, scope)
);

create table admin_ip_allowlist (
  id bigserial primary key,
  cidr cidr not null unique,
  note text not null default '',
  created_at timestamptz not null default now()
);

create trigger rate_limit_buckets_set_updated_at before update on rate_limit_buckets
  for each row execute function set_updated_at();
