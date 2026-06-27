alter table consignments
  add column if not exists price_currency text not null default 'yuanbao',
  add column if not exists trade_tax_gold bigint not null default 0;

alter table consignment_history
  add column if not exists price_currency text not null default 'yuanbao',
  add column if not exists trade_tax_gold bigint not null default 0,
  add column if not exists seller_receives_gold bigint not null default 0;

update consignments
set price_currency = coalesce(nullif(price_currency, ''), 'yuanbao');

update consignment_history
set price_currency = coalesce(nullif(price_currency, ''), 'yuanbao');

insert into world_zones (id, name) values
  ('fengmo', '封魔谷')
on conflict (id) do update set name = excluded.name;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('fengmo', 'camp', '封魔营地', '远征者在山谷口搭起营火，适合从苍月岛补给后继续推进。', '{"苍月牛魔":"cangyue:bull_temple","封魔矿道":"mine_path"}', '[]', true),
  ('fengmo', 'mine_path', '封魔矿道', '残破矿道里魔气翻涌，虹魔蝙蝠和封魔卫兵不断涌出。推荐 50-56 级探索。', '{"封魔营地":"camp","霸者大厅":"overlord_hall"}', '["fengmo_bat","fengmo_guard","rainbow_spirit"]', false),
  ('fengmo', 'overlord_hall', '霸者大厅', '封魔旧殿中仍有战鼓回响，虹魔战士和虹魔猪卫巡守大厅。推荐 56-64 级探索。', '{"封魔矿道":"mine_path","封魔祭坛":"altar"}', '["rainbow_warrior","rainbow_boar","rainbow_guard"]', false),
  ('fengmo', 'altar', '封魔祭坛', '祭坛中央有虹魔教主残影，适合衔接白日门前的装备和突破材料积累。推荐 64-70 级探索。', '{"霸者大厅":"overlord_hall","白日门":"bq_plains:bairimen"}', '["rainbow_guard","rainbow_priest","rainbow_lord"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set exits = exits || '{"封魔谷":"fengmo:camp"}'::jsonb
where zone_id = 'cangyue' and id = 'bull_temple';

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('fengmo_bat', '虹魔蝙蝠', 52, 6800, 330, 170, 5600, 1850, false, 260),
  ('fengmo_guard', '封魔卫兵', 55, 8200, 380, 205, 6800, 2200, false, 300),
  ('rainbow_spirit', '虹魔妖灵', 58, 9600, 430, 230, 8200, 2550, false, 320),
  ('rainbow_warrior', '虹魔战士', 61, 12500, 500, 270, 10800, 3000, false, 340),
  ('rainbow_boar', '虹魔猪卫', 64, 15800, 570, 320, 13500, 3400, false, 360),
  ('rainbow_guard', '虹魔护法', 67, 21000, 650, 390, 17000, 4200, false, 390),
  ('rainbow_priest', '虹魔祭司', 70, 30000, 760, 460, 22000, 5200, false, 420),
  ('rainbow_lord', '虹魔教主', 72, 92000, 980, 620, 82000, 16000, true, 1500)
on conflict (id) do update set
  name = excluded.name,
  level = excluded.level,
  max_hp = excluded.max_hp,
  atk = excluded.atk,
  def = excluded.def,
  exp = excluded.exp,
  gold = excluded.gold,
  boss = excluded.boss,
  respawn_seconds = excluded.respawn_seconds;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('pet_food_pack', '灵兽粮包', 'bundle', null, 'uncommon', 480, true, '{"contains":"pet_food","quantity":3}', '{"bind_on_reward":true}'),
  ('treasure_shard_pack', '法宝碎片匣', 'bundle', null, 'rare', 720, true, '{"contains":"treasure_shard","quantity":3}', '{"bind_on_reward":true}'),
  ('cultivation_pill_pack', '修炼丹匣', 'bundle', null, 'rare', 1080, true, '{"contains":"cultivation_pill","quantity":3}', '{"bind_on_reward":true}'),
  ('guild_merit_token', '行会功勋令', 'token', null, 'uncommon', 300, true, '{"system":"guild","contribution":10}', '{"bind_on_reward":true}'),
  ('guardian_gem', '守御宝石', 'gem', null, 'rare', 900, true, '{"def":2,"mdef":2}', '{"bind_on_reward":true}'),
  ('battle_gem', '战意宝石', 'gem', null, 'rare', 900, true, '{"atk":2,"mag":2}', '{"bind_on_reward":true}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;

insert into quest_templates
  (id, category, name, description, min_level, sort_order, objectives, rewards)
values
  (
    'daily_guild_donate',
    'daily',
    '行会补给',
    '今日完成 1 次行会捐献，稳定获得少量界限突破材料。',
    1,
    320,
    '{"kind":"guild_donate","required":1}',
    '{"gold":120,"items":[{"template_id":"treasure_shard","quantity":1,"bind":true},{"template_id":"guild_merit_token","quantity":1,"bind":true}]}'
  ),
  (
    'daily_growth_supply',
    'daily',
    '每日修行',
    '今日击败 30 只怪物，为宠物、法宝和修炼积累稳定材料。',
    10,
    330,
    '{"kind":"kill_any","required":30}',
    '{"gold":220,"items":[{"template_id":"pet_food","quantity":1,"bind":true},{"template_id":"treasure_shard","quantity":1,"bind":true},{"template_id":"cultivation_pill","quantity":1,"bind":true}]}'
  )
on conflict (id) do update set
  category = excluded.category,
  name = excluded.name,
  description = excluded.description,
  min_level = excluded.min_level,
  sort_order = excluded.sort_order,
  objectives = excluded.objectives,
  rewards = excluded.rewards,
  enabled = true;

update quest_templates
set rewards = '{"gold":220,"items":[{"template_id":"potion_small","quantity":2,"bind":true},{"template_id":"pet_food","quantity":1,"bind":true},{"template_id":"treasure_shard","quantity":1,"bind":true}]}'
where id = 'daily_hunt_20';

update quest_templates
set rewards = '{"gold":120,"items":[{"template_id":"cultivation_pill","quantity":1,"bind":true}]}'
where id = 'daily_afk_settle';

update quest_templates
set rewards = '{"gold":160,"items":[{"template_id":"treasure_shard","quantity":1,"bind":true},{"template_id":"pet_food","quantity":1,"bind":true}]}'
where id = 'newbie_join_guild';
