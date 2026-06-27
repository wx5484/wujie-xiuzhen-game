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
