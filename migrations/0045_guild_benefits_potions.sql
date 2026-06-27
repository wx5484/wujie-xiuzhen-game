create table if not exists guild_benefit_claims (
  guild_id bigint not null references guilds(id) on delete cascade,
  character_id bigint not null references characters(id) on delete cascade,
  level integer not null check (level between 1 and 10),
  period_key text not null default current_date::text,
  claimed_at timestamptz not null default now(),
  primary key (guild_id, character_id, period_key)
);

create index if not exists guild_benefit_claims_character_idx
  on guild_benefit_claims (character_id, period_key);

update guilds
set funds = 2500,
    level = 3
where name = '比奇守卫军'
  and funds >= 18000;

update guilds
set funds = 7000,
    level = 5
where name = '沙城远征队'
  and funds >= 62000;

update guilds
set level = case
  when funds >= 36500 then 10
  when funds >= 26000 then 9
  when funds >= 19000 then 8
  when funds >= 14000 then 7
  when funds >= 10000 then 6
  when funds >= 7000 then 5
  when funds >= 4500 then 4
  when funds >= 2500 then 3
  when funds >= 1000 then 2
  else 1
end;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('potion_small', '生命药剂', 'consumable', null, 'common', 50, true, '{"hp":120}', '{"shop":"supply"}'),
  ('potion_mana', '魔法药剂', 'consumable', null, 'common', 100, true, '{"mp":100}', '{"shop":"supply"}'),
  ('potion_large', '强效生命药剂', 'consumable', null, 'uncommon', 300, true, '{"hp":500}', '{"shop":"supply"}'),
  ('potion_mana_large', '强效魔法药剂', 'consumable', null, 'uncommon', 400, true, '{"mp":400}', '{"shop":"supply"}'),
  ('potion_sun', '太阳药剂', 'consumable', null, 'rare', 1000, true, '{"hp_pct":15,"mp_pct":10}', '{"shop":"supply"}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = item_templates.flags || excluded.flags;
