-- 0.2.8: equipment quality standard and classic set templates.
-- Weapons are rebuilt as attack-only combat items, armor as defense-only combat items.

update item_templates
set rarity = case when rarity in ('supreme', 'ultimate') then 'legendary' else rarity end
where kind in ('weapon', 'armor', 'accessory');

update item_templates
set stats = jsonb_strip_nulls(jsonb_build_object(
  'atk', greatest(coalesce((stats->>'atk')::bigint, 0) + coalesce((stats->>'mag')::bigint, 0), 1),
  'atk_pct', case when rarity = 'legendary' then 6 when rarity = 'epic' then 3 else null end,
  'crit_pct', case when rarity in ('epic', 'legendary') then coalesce((stats->>'crit_pct')::bigint, 0) else null end,
  'heavy_hit_pct', case when rarity in ('epic', 'legendary') then coalesce((stats->>'heavy_hit_pct')::bigint, 0) else null end,
  'paralyze_pct', case when rarity = 'legendary' then coalesce((stats->>'paralyze_pct')::bigint, 0) else null end,
  'petrify_pct', case when rarity = 'legendary' then coalesce((stats->>'petrify_pct')::bigint, 0) else null end,
  'life_steal_pct', case when rarity = 'legendary' then coalesce((stats->>'life_steal_pct')::bigint, 0) else null end,
  'mana_steal_pct', case when rarity = 'legendary' then coalesce((stats->>'mana_steal_pct')::bigint, 0) else null end,
  'tier', greatest(coalesce((stats->>'tier')::bigint, 1), 1),
  'score', greatest(coalesce((stats->>'score')::bigint, 1), 1),
  'affix_count', case rarity
    when 'common' then 0
    when 'uncommon' then 2
    when 'rare' then 4
    when 'epic' then 6
    when 'legendary' then 6
    else 0
  end,
  'special_mechanism', case when rarity = 'legendary' then coalesce(stats->>'special_mechanism', '传奇特殊机制') else null end
))
where kind = 'weapon';

update item_templates
set stats = jsonb_strip_nulls(jsonb_build_object(
  'def', greatest(coalesce((stats->>'def')::bigint, 0) + coalesce((stats->>'hp')::bigint, 0) / 90, 1),
  'mdef', greatest(coalesce((stats->>'mdef')::bigint, 0) + coalesce((stats->>'mp')::bigint, 0) / 90, 0),
  'def_pct', case when rarity = 'legendary' then 5 when rarity = 'epic' then 2 else null end,
  'damage_reduce_pct', case when rarity in ('epic', 'legendary') then 1 else null end,
  'paralyze_resist_pct', case when rarity = 'legendary' then 1 else null end,
  'petrify_resist_pct', case when rarity = 'legendary' then 1 else null end,
  'tier', greatest(coalesce((stats->>'tier')::bigint, 1), 1),
  'score', greatest(coalesce((stats->>'score')::bigint, 1), 1),
  'affix_count', case rarity
    when 'common' then 0
    when 'uncommon' then 2
    when 'rare' then 4
    when 'epic' then 6
    when 'legendary' then 6
    else 0
  end,
  'special_mechanism', case when rarity = 'legendary' then coalesce(stats->>'special_mechanism', '传奇防护机制') else null end
))
where kind = 'armor';

update item_templates
set stats = stats || jsonb_build_object(
  'affix_count', case rarity
    when 'common' then 0
    when 'uncommon' then 2
    when 'rare' then 4
    when 'epic' then 6
    when 'legendary' then 6
    else 0
  end
)
where kind = 'accessory';

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('armor_woma_t3', '沃玛战衣', 'armor', 'chest', 'epic', 4300, false, '{"def":54,"mdef":16,"tier":3,"score":128,"affix_count":5,"set":"woma"}', '{"set":"woma","exclusive_boss":"woma_lord","quality_standard":"legend_equipment_2026"}'),
  ('helm_woma_t3', '沃玛头盔', 'armor', 'head', 'epic', 3900, false, '{"def":34,"mdef":10,"tier":3,"score":96,"affix_count":4,"set":"woma"}', '{"set":"woma","exclusive_boss":"woma_lord","quality_standard":"legend_equipment_2026"}'),
  ('neck_woma_t3', '沃玛项链', 'accessory', 'neck', 'epic', 4200, false, '{"atk":20,"mag":18,"luck":8,"tier":3,"score":108,"affix_count":4,"set":"woma"}', '{"set":"woma","exclusive_boss":"woma_lord","quality_standard":"legend_equipment_2026"}'),
  ('bracelet_woma_t3', '沃玛手镯', 'accessory', 'bracelet_left', 'epic', 4100, false, '{"atk":18,"def":8,"mdef":6,"luck":6,"tier":3,"score":102,"affix_count":4,"set":"woma"}', '{"set":"woma","exclusive_boss":"woma_lord","quality_standard":"legend_equipment_2026"}'),
  ('boots_woma_t3', '沃玛战靴', 'armor', 'feet', 'epic', 3600, false, '{"def":24,"mdef":8,"tier":3,"score":88,"affix_count":4,"set":"woma"}', '{"set":"woma","exclusive_boss":"woma_lord","quality_standard":"legend_equipment_2026"}'),

  ('armor_zuma_t4', '祖玛战甲', 'armor', 'chest', 'epic', 11600, false, '{"def":104,"mdef":30,"tier":4,"score":292,"affix_count":5,"set":"zuma"}', '{"set":"zuma","exclusive_boss":"zuma_lord","quality_standard":"legend_equipment_2026"}'),
  ('helm_zuma_t4', '祖玛头盔', 'armor', 'head', 'epic', 9200, false, '{"def":70,"mdef":14,"tier":4,"score":216,"affix_count":4,"set":"zuma"}', '{"set":"zuma","exclusive_boss":"zuma_lord","quality_standard":"legend_equipment_2026"}'),
  ('neck_zuma_t4', '祖玛项链', 'accessory', 'neck', 'epic', 9800, false, '{"atk":44,"mag":34,"luck":12,"crit_pct":2,"tier":4,"score":244,"affix_count":5,"set":"zuma"}', '{"set":"zuma","exclusive_boss":"zuma_lord","quality_standard":"legend_equipment_2026"}'),
  ('bracelet_zuma_t4', '祖玛手镯', 'accessory', 'bracelet_left', 'epic', 9400, false, '{"atk":38,"mag":28,"luck":15,"tier":4,"score":224,"affix_count":4,"set":"zuma"}', '{"set":"zuma","exclusive_boss":"zuma_lord","quality_standard":"legend_equipment_2026"}'),
  ('ring_zuma_t4', '祖玛戒指', 'accessory', 'ring_left', 'epic', 9800, false, '{"atk":52,"mag":38,"luck":14,"crit_pct":2,"tier":4,"score":266,"affix_count":5,"set":"zuma"}', '{"set":"zuma","exclusive_boss":"zuma_lord","quality_standard":"legend_equipment_2026"}'),
  ('boots_zuma_t4', '祖玛战靴', 'armor', 'feet', 'epic', 9000, false, '{"def":58,"mdef":18,"tier":4,"score":205,"affix_count":4,"set":"zuma"}', '{"set":"zuma","exclusive_boss":"zuma_lord","quality_standard":"legend_equipment_2026"}'),

  ('armor_mirage_t5', '幻境战甲', 'armor', 'chest', 'epic', 34000, false, '{"def":154,"mdef":58,"tier":5,"score":470,"affix_count":5,"set":"mirage"}', '{"set":"mirage","exclusive_boss":"bull_king","quality_standard":"legend_equipment_2026"}'),
  ('helm_mirage_t5', '幻境头盔', 'armor', 'head', 'epic', 30000, false, '{"def":96,"mdef":34,"tier":5,"score":350,"affix_count":4,"set":"mirage"}', '{"set":"mirage","exclusive_boss":"bull_king","quality_standard":"legend_equipment_2026"}'),
  ('neck_mirage_t5', '幻境项链', 'accessory', 'neck', 'epic', 30000, false, '{"atk":72,"mag":72,"luck":28,"crit_pct":3,"tier":5,"score":430,"affix_count":5,"set":"mirage"}', '{"set":"mirage","exclusive_boss":"bull_king","quality_standard":"legend_equipment_2026"}'),
  ('bracelet_mirage_t5', '幻境手镯', 'accessory', 'bracelet_left', 'epic', 26000, false, '{"atk":58,"def":30,"mdef":22,"luck":26,"tier":5,"score":360,"affix_count":5,"set":"mirage"}', '{"set":"mirage","exclusive_boss":"bull_king","quality_standard":"legend_equipment_2026"}'),
  ('boots_mirage_t5', '幻境战靴', 'armor', 'feet', 'epic', 24000, false, '{"def":84,"mdef":28,"tier":5,"score":320,"affix_count":4,"set":"mirage"}', '{"set":"mirage","exclusive_boss":"bull_king","quality_standard":"legend_equipment_2026"}'),

  ('blade_redmoon_t8', '赤月之刃', 'weapon', 'weapon', 'legendary', 150000, false, '{"atk":390,"atk_pct":8,"crit_pct":6,"heavy_hit_pct":6,"life_steal_pct":2,"boss_damage_pct":5,"tier":8,"score":1220,"affix_count":6,"special_mechanism":"赤月之怒","set":"redmoon"}', '{"set":"redmoon","exclusive_boss":"redmoon_demon","quality_standard":"legend_equipment_2026"}'),
  ('armor_redmoon_t8', '赤月战甲', 'armor', 'chest', 'legendary', 148000, false, '{"def":390,"mdef":150,"def_pct":6,"damage_reduce_pct":2,"paralyze_resist_pct":2,"petrify_resist_pct":1,"tier":8,"score":1220,"affix_count":6,"special_mechanism":"赤月护体","set":"redmoon"}', '{"set":"redmoon","exclusive_boss":"redmoon_demon","quality_standard":"legend_equipment_2026"}'),
  ('helm_redmoon_t8', '赤月头盔', 'armor', 'head', 'legendary', 132000, false, '{"def":236,"mdef":76,"def_pct":4,"damage_reduce_pct":1,"paralyze_resist_pct":1,"petrify_resist_pct":1,"tier":8,"score":900,"affix_count":6,"special_mechanism":"赤月护体","set":"redmoon"}', '{"set":"redmoon","exclusive_boss":"redmoon_demon","quality_standard":"legend_equipment_2026"}'),
  ('neck_redmoon_t8', '赤月项链', 'accessory', 'neck', 'legendary', 136000, false, '{"atk":168,"mag":168,"luck":70,"crit_pct":6,"heavy_hit_pct":3,"mana_steal_pct":3,"tier":8,"score":980,"affix_count":6,"special_mechanism":"赤月汲取","set":"redmoon"}', '{"set":"redmoon","exclusive_boss":"redmoon_demon","quality_standard":"legend_equipment_2026"}'),
  ('bracelet_redmoon_t8', '赤月手镯', 'accessory', 'bracelet_left', 'legendary', 134000, false, '{"atk":158,"def":96,"mdef":72,"luck":72,"crit_pct":5,"life_steal_pct":2,"tier":8,"score":960,"affix_count":6,"special_mechanism":"赤月汲取","set":"redmoon"}', '{"set":"redmoon","exclusive_boss":"redmoon_demon","quality_standard":"legend_equipment_2026"}'),
  ('ring_redmoon_t8', '赤月戒指', 'accessory', 'ring_left', 'legendary', 138000, false, '{"atk":176,"mag":130,"luck":74,"crit_pct":7,"crit_damage_pct":10,"mana_steal_pct":3,"tier":8,"score":1010,"affix_count":6,"special_mechanism":"赤月暴击","set":"redmoon"}', '{"set":"redmoon","exclusive_boss":"redmoon_demon","quality_standard":"legend_equipment_2026"}'),
  ('boots_redmoon_t8', '赤月战靴', 'armor', 'feet', 'legendary', 128000, false, '{"def":206,"mdef":66,"def_pct":4,"damage_reduce_pct":1,"paralyze_resist_pct":1,"petrify_resist_pct":1,"tier":8,"score":850,"affix_count":6,"special_mechanism":"赤月护体","set":"redmoon"}', '{"set":"redmoon","exclusive_boss":"redmoon_demon","quality_standard":"legend_equipment_2026"}'),

  ('blade_molong_t10', '魔龙神刃', 'weapon', 'weapon', 'legendary', 260000, false, '{"atk":690,"atk_pct":10,"crit_pct":8,"heavy_hit_pct":10,"paralyze_pct":2,"normal_mob_execute_pct":1,"tier":10,"score":2100,"affix_count":6,"special_mechanism":"魔龙降临","set":"molong"}', '{"set":"molong","exclusive_boss":"molong_lord","quality_standard":"legend_equipment_2026"}'),
  ('armor_molong_t10', '魔龙神甲', 'armor', 'chest', 'legendary', 252000, false, '{"def":690,"mdef":260,"def_pct":8,"damage_reduce_pct":3,"paralyze_resist_pct":2,"petrify_resist_pct":2,"tier":10,"score":2100,"affix_count":6,"special_mechanism":"魔龙护体","set":"molong"}', '{"set":"molong","exclusive_boss":"molong_lord","quality_standard":"legend_equipment_2026"}'),
  ('helm_molong_t10', '魔龙神盔', 'armor', 'head', 'legendary', 230000, false, '{"def":398,"mdef":132,"def_pct":6,"damage_reduce_pct":2,"paralyze_resist_pct":2,"petrify_resist_pct":1,"tier":10,"score":1580,"affix_count":6,"special_mechanism":"魔龙护体","set":"molong"}', '{"set":"molong","exclusive_boss":"molong_lord","quality_standard":"legend_equipment_2026"}'),
  ('neck_molong_t10', '魔龙项链', 'accessory', 'neck', 'legendary', 238000, false, '{"atk":306,"mag":330,"luck":132,"crit_pct":8,"heavy_hit_pct":6,"mana_steal_pct":6,"tier":10,"score":1760,"affix_count":6,"special_mechanism":"魔龙汲取","set":"molong"}', '{"set":"molong","exclusive_boss":"molong_lord","quality_standard":"legend_equipment_2026"}'),
  ('bracelet_molong_t10', '魔龙手镯', 'accessory', 'bracelet_left', 'legendary', 232000, false, '{"atk":302,"def":182,"mdef":138,"luck":138,"crit_pct":7,"heavy_hit_pct":8,"tier":10,"score":1740,"affix_count":6,"special_mechanism":"魔龙压制","set":"molong"}', '{"set":"molong","exclusive_boss":"molong_lord","quality_standard":"legend_equipment_2026"}'),
  ('ring_molong_t10', '魔龙戒指', 'accessory', 'ring_left', 'legendary', 240000, false, '{"atk":320,"mag":275,"luck":135,"crit_pct":9,"crit_damage_pct":12,"mana_steal_pct":6,"tier":10,"score":1800,"affix_count":6,"special_mechanism":"魔龙暴击","set":"molong"}', '{"set":"molong","exclusive_boss":"molong_lord","quality_standard":"legend_equipment_2026"}'),
  ('boots_molong_t10', '魔龙神靴', 'armor', 'feet', 'legendary', 226000, false, '{"def":360,"mdef":120,"def_pct":5,"damage_reduce_pct":2,"paralyze_resist_pct":1,"petrify_resist_pct":1,"tier":10,"score":1460,"affix_count":6,"special_mechanism":"魔龙护体","set":"molong"}', '{"set":"molong","exclusive_boss":"molong_lord","quality_standard":"legend_equipment_2026"}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = item_templates.flags || excluded.flags;
