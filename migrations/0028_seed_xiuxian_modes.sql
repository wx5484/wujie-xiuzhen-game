create table if not exists character_challenge_state (
  character_id bigint primary key references characters(id) on delete cascade,
  tower_floor integer not null default 1,
  secret_realm_available_at timestamptz not null default now(),
  world_boss_available_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

drop trigger if exists character_challenge_state_set_updated_at on character_challenge_state;
create trigger character_challenge_state_set_updated_at before update on character_challenge_state
  for each row execute function set_updated_at();

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats) values
  ('stone_refine', '炼器石', 'material', null, 'uncommon', 30, true, '{}'),
  ('stone_hongmeng', '鸿蒙石', 'material', null, 'legendary', 800, true, '{}'),
  ('pill_insight', '悟性丹', 'material', null, 'epic', 500, true, '{}'),
  ('pill_cultivate', '培养丹', 'material', null, 'rare', 120, true, '{}'),
  ('blade_green_t1', '青锋剑', 'weapon', 'weapon', 'uncommon', 620, false, '{"atk":8,"dex":1,"tier":1,"score":18}'),
  ('blade_darkiron_t2', '玄铁剑', 'weapon', 'weapon', 'rare', 1400, false, '{"atk":18,"mag":2,"tier":2,"score":42}'),
  ('blade_flame_t3', '赤焰神兵', 'weapon', 'weapon', 'epic', 3600, false, '{"atk":36,"dex":4,"tier":3,"score":88}'),
  ('blade_dragon_t4', '龙吟镇岳剑', 'weapon', 'weapon', 'legendary', 9000, false, '{"atk":72,"mag":10,"dex":8,"tier":4,"score":188}'),
  ('armor_jade_t1', '青玉法衣', 'armor', 'chest', 'uncommon', 680, false, '{"def":7,"hp":65,"tier":1,"score":20}'),
  ('armor_cloud_t2', '云纹玄甲', 'armor', 'chest', 'rare', 1500, false, '{"def":16,"hp":150,"mdef":3,"tier":2,"score":48}'),
  ('armor_star_t3', '星辉战袍', 'armor', 'chest', 'epic', 3800, false, '{"def":34,"hp":360,"mdef":8,"tier":3,"score":104}'),
  ('armor_phoenix_t4', '凤舞九天甲', 'armor', 'chest', 'legendary', 9600, false, '{"def":72,"hp":780,"mdef":18,"tier":4,"score":226}'),
  ('ring_moon_t1', '月华指环', 'accessory', 'ring_left', 'uncommon', 760, false, '{"atk":4,"hp":35,"dex":2,"tier":1,"score":17}'),
  ('ring_sea_t2', '碧海灵戒', 'accessory', 'ring_left', 'rare', 1700, false, '{"atk":9,"mag":6,"hp":90,"dex":4,"tier":2,"score":45}'),
  ('ring_thunder_t3', '紫霄雷戒', 'accessory', 'ring_left', 'epic', 4200, false, '{"atk":20,"mag":14,"hp":210,"dex":8,"tier":3,"score":103}'),
  ('ring_sun_t4', '大日鸿蒙戒', 'accessory', 'ring_left', 'legendary', 10800, false, '{"atk":42,"mag":30,"hp":460,"dex":15,"tier":4,"score":232}'),
  ('boss_relic_blade', '赤焰凰血剑', 'weapon', 'weapon', 'supreme', 30000, false, '{"atk":128,"mag":24,"dex":16,"tier":5,"score":360}'),
  ('boss_relic_armor', '九转玄火甲', 'armor', 'chest', 'supreme', 30000, false, '{"def":126,"mdef":42,"hp":1500,"tier":5,"score":390}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats;
