insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats) values
  ('potion_small', '小红药', 'consumable', null, 'common', 35, true, '{"hp":60}'),
  ('potion_mana', '小蓝药', 'consumable', null, 'common', 45, true, '{"mp":50}'),
  ('scroll_return', '回城卷', 'consumable', null, 'common', 200, true, '{"teleport":{"zone":"bq_town","room":"gate"}}'),
  ('sword_wood', '木剑', 'weapon', 'weapon', 'common', 120, false, '{"atk":2}'),
  ('armor_cloth', '布衣', 'armor', 'chest', 'common', 160, false, '{"def":2,"hp":20}'),
  ('ring_copper', '青铜戒指', 'accessory', 'ring_left', 'uncommon', 260, false, '{"atk":1,"dex":1}'),
  ('book_fireball', '火球术', 'book', null, 'rare', 900, false, '{"skill":"fireball"}'),
  ('woma_horn', '沃玛号角', 'material', null, 'epic', 3000, false, '{}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats;

insert into world_zones (id, name) values
  ('bq_town', '比奇城'),
  ('bq_plains', '比奇野外')
on conflict (id) do update set name = excluded.name;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('bq_town', 'gate', '比奇城门', '守卫盯着大道，行人络绎不绝。向北就是练级的野外。', '{"north":"bq_plains:plains","east":"market"}', '[]', true),
  ('bq_town', 'market', '集市街', '商人叫卖着药水与装备，练功木人静立一旁。', '{"west":"gate"}', '[]', true),
  ('bq_plains', 'plains', '比奇平原', '开阔草地上小怪徘徊，南边能回城，北边通往森林。', '{"south":"bq_town:gate","north":"forest","east":"wolfden"}', '["chicken","deer","sheep"]', false),
  ('bq_plains', 'forest', '森林小径', '高树投下阴影，狼群、毒蜂和钉耙猫在这里潜伏。', '{"south":"plains","north":"ruins"}', '["wolf","bee","cat"]', false),
  ('bq_plains', 'wolfden', '狼穴', '遍地白骨，寒冷的嚎叫从洞口深处回荡。', '{"west":"plains"}', '["wolf","wolf","wolf"]', false),
  ('bq_plains', 'ruins', '废弃遗迹', '残破石柱围住古老祭坛，骷髅和蜘蛛守着更深处的入口。', '{"south":"forest"}', '["skeleton","spider"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('chicken', '鸡', 1, 25, 3, 0, 8, 5, false, 45),
  ('deer', '鹿', 2, 35, 4, 1, 12, 8, false, 60),
  ('sheep', '羊', 3, 48, 5, 1, 18, 12, false, 60),
  ('bee', '毒蜂', 4, 62, 8, 2, 26, 18, false, 75),
  ('cat', '钉耙猫', 4, 70, 9, 2, 30, 20, false, 80),
  ('wolf', '森林狼', 5, 90, 11, 3, 35, 24, false, 90),
  ('spider', '洞穴蜘蛛', 8, 130, 16, 5, 60, 40, false, 110),
  ('skeleton', '骷髅', 9, 160, 18, 6, 80, 55, false, 120),
  ('bug_queen', '触龙神', 35, 12000, 260, 90, 12000, 4000, true, 3600)
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
