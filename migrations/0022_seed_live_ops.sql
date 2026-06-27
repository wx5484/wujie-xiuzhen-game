insert into activities (code, name, enabled, config) values
  ('daily_hunt', '每日猎魔', true, '{"target":"击杀任意怪物 20 次","reward":{"gold":500,"item":"potion_small"}}'),
  ('boss_first_kill', '触龙神首杀', true, '{"target":"挑战触龙神巢穴","reward":{"yuanbao":100,"title":"屠龙先锋"}}')
on conflict (code) do update set
  name = excluded.name,
  enabled = excluded.enabled,
  config = excluded.config;

insert into guilds (name, notice, level, funds, sabak_owner) values
  ('比奇守卫军', '测试服默认行会，GM 可通过后台观察行会运营数据。', 1, 10000, false),
  ('沙城远征队', '沙巴克玩法入口样例。', 3, 50000, true)
on conflict (name) do update set
  notice = excluded.notice,
  level = excluded.level,
  funds = excluded.funds,
  sabak_owner = excluded.sabak_owner;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('bq_plains', 'plains', '比奇平原', '开阔草地上小怪徘徊，南边能回城，北边通往森林，西边通向触龙神巢穴。', '{"south":"bq_town:gate","north":"forest","east":"wolfden","west":"dragon_lair"}', '["chicken","deer","sheep"]', false),
  ('bq_plains', 'dragon_lair', '触龙神巢穴', '地底热风从裂缝喷出，触龙神盘踞在祭坛中央。这里是当前版本的世界 BOSS 目标。', '{"east":"plains"}', '["bug_queen"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;
