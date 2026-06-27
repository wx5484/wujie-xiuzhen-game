-- 0.2.13 follow-up: normalize dominator templates and clear stale generated rolls.

with dominator_templates(id, name, kind, slot, price, stats) as (
  values
    (
      'dominator_blade', '主宰神刃', 'weapon', 'weapon', 1500000,
      '{"atk":1280,"atk_pct":14,"crit_pct":10,"heavy_hit_pct":12,"boss_damage_pct":10,"ignore_def_pct":8,"guaranteed_hit_pct":5,"life_steal_pct":4,"tier":12,"score":4200,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}'::jsonb
    ),
    (
      'dominator_armor', '主宰神甲', 'armor', 'chest', 1450000,
      '{"def":1280,"mdef":520,"def_pct":12,"damage_reduce_pct":6,"paralyze_resist_pct":6,"petrify_resist_pct":6,"hp_pct":8,"mp_pct":6,"tier":12,"score":4100,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}'::jsonb
    ),
    (
      'dominator_helm', '主宰神盔', 'armor', 'head', 1200000,
      '{"def":760,"mdef":280,"def_pct":8,"damage_reduce_pct":4,"paralyze_resist_pct":5,"petrify_resist_pct":5,"hp_pct":5,"mp_pct":4,"tier":12,"score":3180,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}'::jsonb
    ),
    (
      'dominator_boots', '主宰神靴', 'armor', 'feet', 1180000,
      '{"def":700,"mdef":260,"def_pct":7,"damage_reduce_pct":4,"paralyze_resist_pct":5,"petrify_resist_pct":5,"hp_pct":4,"mp_pct":4,"tier":12,"score":3020,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}'::jsonb
    ),
    (
      'dominator_belt', '主宰神带', 'armor', 'waist', 1160000,
      '{"def":720,"mdef":300,"def_pct":7,"damage_reduce_pct":4,"paralyze_resist_pct":5,"petrify_resist_pct":5,"hp_pct":6,"mp_pct":3,"tier":12,"score":3100,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}'::jsonb
    ),
    (
      'dominator_necklace', '主宰项链', 'accessory', 'neck', 1220000,
      '{"atk":560,"mag":560,"luck":220,"crit_pct":10,"crit_damage_pct":18,"skill_damage_pct":8,"boss_damage_pct":8,"mana_steal_pct":5,"tier":12,"score":3380,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}'::jsonb
    ),
    (
      'dominator_bracelet', '主宰手镯', 'accessory', 'bracelet_left', 1200000,
      '{"atk":520,"mag":420,"def":260,"mdef":180,"luck":210,"crit_pct":8,"heavy_hit_pct":8,"life_steal_pct":5,"tier":12,"score":3260,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}'::jsonb
    ),
    (
      'dominator_ring', '主宰戒指', 'accessory', 'ring_left', 1240000,
      '{"atk":620,"mag":470,"luck":230,"crit_pct":12,"crit_damage_pct":22,"heavy_hit_pct":8,"life_steal_pct":4,"mana_steal_pct":4,"tier":12,"score":3500,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}'::jsonb
    )
)
update item_templates it
set
  name = dt.name,
  kind = dt.kind,
  slot = dt.slot,
  rarity = 'mythic',
  price = dt.price,
  stackable = false,
  stats = dt.stats,
  flags = '{"set":"dominator","exclusive_source":"world_boss_eternal_abyss_demon","equipment_standard":"dominator_necklace_560_scale"}'::jsonb,
  updated_at = now()
from dominator_templates dt
where it.id = dt.id;

update inventory_items
set extra = extra - 'generated_stats'
where template_id like 'dominator_%'
  and extra ? 'generated_stats';
