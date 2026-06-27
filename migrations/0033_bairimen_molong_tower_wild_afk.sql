alter table character_challenge_state
  add column if not exists tower_available_at timestamptz not null default now();

update character_challenge_state
set tower_floor = least(greatest(tower_floor, 1), 200);

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('bq_plains', 'bairimen', '白日门', '比奇边境的高阶安全据点，通往赤月峡谷方向。推荐补给后再深入。', '{"比奇平原":"plains","山谷密道":"valley_secret"}', '[]', true),
  ('bq_plains', 'valley_secret', '山谷密道', '石壁潮湿狭窄，月魔蜘蛛沿着洞顶游走。推荐 70-78 级探索。', '{"白日门":"bairimen","抉择之地":"choice_land"}', '["moon_spider","blood_giant"]', false),
  ('bq_plains', 'choice_land', '抉择之地', '岔路尽头有赤月气息渗出，血巨人和月魔蜘蛛守住道路。推荐 78-86 级探索。', '{"山谷密道":"valley_secret","恶魔祭坛":"demon_altar"}', '["moon_spider","blood_giant","redmoon_priest"]', false),
  ('bq_plains', 'demon_altar', '恶魔祭坛', '祭坛上残火不灭，赤月祭司和双头金刚巡守周围。推荐 86-94 级探索。', '{"抉择之地":"choice_land","赤月峡谷":"redmoon_canyon"}', '["redmoon_priest","twin_head_guard"]', false),
  ('bq_plains', 'redmoon_canyon', '赤月峡谷', '深谷中赤月恶魔的威压直逼神魂，是白日门路线尽头。推荐 94-100 级探索。', '{"恶魔祭坛":"demon_altar"}', '["redmoon_priest","twin_head_guard","redmoon_demon"]', false),

  ('cangyue', 'molong_city', '魔龙城', '苍月岛远征军建立的高阶安全区，城外就是魔龙东郊。推荐 100 级后进入。', '{"苍月岛海岸":"island","魔龙东郊":"molong_east"}', '[]', true),
  ('cangyue', 'molong_east', '魔龙东郊', '城外荒土遍布魔龙守卫和魔龙枪兵。推荐 100-112 级探索。', '{"魔龙城":"molong_city","东林间胜地":"east_forest"}', '["molong_guard","molong_lancer"]', false),
  ('cangyue', 'east_forest', '东林间胜地', '林间雾气遮住视线，魔龙猎手会从树影中突袭。推荐 112-124 级探索。', '{"魔龙东郊":"molong_east","魔龙旧寨":"old_molong_village"}', '["molong_lancer","molong_hunter"]', false),
  ('cangyue', 'old_molong_village', '魔龙旧寨', '旧寨残垣里还残留魔龙军阵，魔龙统领在此集结残部。推荐 124-136 级探索。', '{"东林间胜地":"east_forest","魔龙沼泽":"molong_swamp"}', '["molong_hunter","molong_general"]', false),
  ('cangyue', 'molong_swamp', '魔龙沼泽', '黑水沼泽会吞没脚步，魔龙战将守着血域入口。推荐 136-146 级探索。', '{"魔龙旧寨":"old_molong_village","魔龙血域":"molong_bloodland"}', '["molong_general","molong_warrior"]', false),
  ('cangyue', 'molong_bloodland', '魔龙血域', '血域尽头魔龙教主盘踞，普通探索也有极高死亡风险。推荐 146-150 级探索。', '{"魔龙沼泽":"molong_swamp"}', '["molong_warrior","molong_blood_demon","molong_lord"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set exits = exits || '{"白日门":"bairimen"}'::jsonb
where zone_id = 'bq_plains' and id = 'plains';

update world_rooms
set exits = exits || '{"魔龙城":"molong_city"}'::jsonb
where zone_id = 'cangyue' and id = 'island';

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('moon_spider', '月魔蜘蛛', 70, 18000, 620, 260, 18000, 3600, false, 300),
  ('blood_giant', '血巨人', 78, 26000, 760, 340, 26000, 5000, false, 360),
  ('redmoon_priest', '赤月祭司', 86, 34000, 920, 430, 36000, 7000, false, 420),
  ('twin_head_guard', '双头金刚', 94, 90000, 1250, 620, 90000, 16000, true, 1200),
  ('redmoon_demon', '赤月恶魔', 100, 180000, 1550, 760, 180000, 32000, true, 1800),

  ('molong_guard', '魔龙守卫', 100, 50000, 1350, 680, 52000, 9500, false, 420),
  ('molong_lancer', '魔龙枪兵', 110, 72000, 1650, 820, 70000, 12500, false, 480),
  ('molong_hunter', '魔龙猎手', 120, 95000, 1950, 980, 92000, 16000, false, 540),
  ('molong_general', '魔龙统领', 132, 140000, 2350, 1200, 130000, 22000, false, 600),
  ('molong_warrior', '魔龙战将', 142, 180000, 2700, 1450, 170000, 28000, false, 660),
  ('molong_blood_demon', '魔龙血魔', 150, 260000, 3150, 1650, 240000, 42000, true, 1800),
  ('molong_lord', '魔龙教主', 150, 420000, 3400, 1750, 360000, 65000, true, 2400)
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
