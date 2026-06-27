create table item_templates (
  id text primary key,
  name text not null,
  kind text not null,
  slot text,
  rarity text not null default 'common',
  price bigint not null default 0,
  stackable boolean not null default false,
  stats jsonb not null default '{}'::jsonb,
  flags jsonb not null default '{}'::jsonb,
  version bigint not null default 1,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table inventory_items (
  id bigserial primary key,
  character_id bigint not null references characters(id) on delete cascade,
  template_id text not null references item_templates(id),
  quantity bigint not null default 1 check (quantity > 0),
  location text not null check (location in ('bag', 'warehouse', 'equipped', 'mail_attachment', 'consignment')),
  slot text,
  bind boolean not null default false,
  durability integer not null default 100,
  extra jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create trigger item_templates_set_updated_at before update on item_templates
  for each row execute function set_updated_at();
create trigger inventory_items_set_updated_at before update on inventory_items
  for each row execute function set_updated_at();
