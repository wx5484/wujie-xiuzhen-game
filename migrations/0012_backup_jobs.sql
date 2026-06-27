create table backup_jobs (
  id bigserial primary key,
  status text not null default 'queued' check (status in ('queued', 'running', 'succeeded', 'failed')),
  file_name text,
  requested_by bigint references accounts(id) on delete set null,
  reason text not null default '',
  error text,
  started_at timestamptz,
  finished_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create trigger backup_jobs_set_updated_at before update on backup_jobs
  for each row execute function set_updated_at();
