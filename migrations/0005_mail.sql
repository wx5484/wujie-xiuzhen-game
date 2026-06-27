create table mails (
  id bigserial primary key,
  to_character_id bigint not null references characters(id) on delete cascade,
  from_account_id bigint references accounts(id) on delete set null,
  from_name text not null default 'system',
  title text not null,
  body text not null,
  read boolean not null default false,
  claimed boolean not null default false,
  expires_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table mail_attachments (
  id bigserial primary key,
  mail_id bigint not null references mails(id) on delete cascade,
  item_template_id text references item_templates(id),
  quantity bigint not null default 1,
  gold bigint not null default 0,
  yuanbao bigint not null default 0,
  claimed_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create trigger mails_set_updated_at before update on mails
  for each row execute function set_updated_at();
create trigger mail_attachments_set_updated_at before update on mail_attachments
  for each row execute function set_updated_at();
