create table trades (
  id bigserial primary key,
  from_character_id bigint not null references characters(id) on delete cascade,
  to_character_id bigint not null references characters(id) on delete cascade,
  status text not null default 'open' check (status in ('open', 'locked', 'completed', 'cancelled')),
  offer_from jsonb not null default '{}'::jsonb,
  offer_to jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table consignments (
  id bigserial primary key,
  seller_character_id bigint not null references characters(id) on delete cascade,
  item_id bigint not null references inventory_items(id),
  price_yuanbao bigint not null default 0,
  fee_yuanbao bigint not null default 0,
  status text not null default 'listed' check (status in ('listed', 'sold', 'cancelled', 'expired')),
  expires_at timestamptz not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table consignment_history (
  id bigserial primary key,
  consignment_id bigint not null references consignments(id) on delete cascade,
  buyer_character_id bigint references characters(id) on delete set null,
  seller_character_id bigint not null references characters(id) on delete cascade,
  item_snapshot jsonb not null default '{}'::jsonb,
  price_yuanbao bigint not null,
  completed_at timestamptz not null default now()
);

create trigger trades_set_updated_at before update on trades
  for each row execute function set_updated_at();
create trigger consignments_set_updated_at before update on consignments
  for each row execute function set_updated_at();
