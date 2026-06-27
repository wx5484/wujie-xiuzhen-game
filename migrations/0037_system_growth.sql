create table if not exists cultivation_states (
  character_id bigint primary key references characters(id) on delete cascade,
  layer integer not null default 1,
  progress bigint not null default 0,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create trigger cultivation_states_set_updated_at before update on cultivation_states
  for each row execute function set_updated_at();

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('pet_food', '灵兽粮', 'material', null, 'common', 120, true, '{"system":"pet"}', '{"bind_on_reward":true}'),
  ('treasure_shard', '法宝碎片', 'material', null, 'uncommon', 180, true, '{"system":"treasure"}', '{"bind_on_reward":true}'),
  ('cultivation_pill', '修炼丹', 'material', null, 'rare', 360, true, '{"system":"cultivation"}', '{"bind_on_reward":true}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;
