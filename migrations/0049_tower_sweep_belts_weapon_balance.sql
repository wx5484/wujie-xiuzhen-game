-- 0.2.9: tower sweep support data, belt gaps, and mid-game weapon balance.

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  (
    'belt_woma_t3',
    '沃玛腰带',
    'armor',
    'waist',
    'epic',
    3800,
    false,
    '{"def":36,"mdef":12,"def_pct":2,"damage_reduce_pct":1,"tier":3,"score":110,"affix_count":4,"set":"woma"}',
    '{"set":"woma","exclusive_boss":"woma_lord","quality_standard":"legend_equipment_2026"}'
  ),
  (
    'belt_zuma_t4',
    '祖玛腰带',
    'armor',
    'waist',
    'epic',
    9400,
    false,
    '{"def":74,"mdef":24,"def_pct":2,"damage_reduce_pct":1,"tier":4,"score":232,"affix_count":4,"set":"zuma"}',
    '{"set":"zuma","exclusive_boss":"zuma_lord","quality_standard":"legend_equipment_2026"}'
  ),
  (
    'belt_dragon_t4',
    '龙纹腰带',
    'armor',
    'waist',
    'legendary',
    11800,
    false,
    '{"def":92,"mdef":30,"def_pct":5,"damage_reduce_pct":2,"paralyze_resist_pct":1,"petrify_resist_pct":1,"tier":4,"score":286,"affix_count":6,"special_mechanism":"龙纹护体"}',
    '{"quality_standard":"legend_equipment_2026"}'
  ),
  (
    'belt_mirage_t5',
    '幻境腰带',
    'armor',
    'waist',
    'epic',
    26000,
    false,
    '{"def":108,"mdef":40,"def_pct":2,"damage_reduce_pct":1,"tier":5,"score":365,"affix_count":5,"set":"mirage"}',
    '{"set":"mirage","exclusive_boss":"bull_king","quality_standard":"legend_equipment_2026"}'
  )
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = item_templates.flags || excluded.flags;

update item_templates
set
  price = 12800,
  stats = '{"atk":138,"atk_pct":8,"crit_pct":4,"heavy_hit_pct":3,"tier":4,"score":360,"affix_count":6,"special_mechanism":"裁决重击","skill_flame_blade_bonus":1}'::jsonb,
  flags = flags || '{"quality_standard":"legend_equipment_2026","balance_patch":"0049_mid_weapon"}'::jsonb
where id = 'blade_dragon_t4';

update item_templates
set
  price = 15000,
  stats = '{"atk":128,"atk_pct":5,"crit_pct":3,"heavy_hit_pct":5,"tier":4,"score":330,"affix_count":6}'::jsonb,
  flags = flags || '{"quality_standard":"legend_equipment_2026","balance_patch":"0049_mid_weapon"}'::jsonb
where id = 'blade_purgatory_t4';

update item_templates
set
  stats = stats || '{"atk":62,"atk_pct":3,"crit_pct":1,"heavy_hit_pct":1,"score":138,"affix_count":6}'::jsonb,
  flags = flags || '{"quality_standard":"legend_equipment_2026","balance_patch":"0049_mid_weapon"}'::jsonb
where id = 'blade_flame_t3';
