-- 0.2.15: growth caps, tier 11-13 equipment, and special mythic gear mechanics.

alter table vip_potion_settings
  drop constraint if exists vip_potion_settings_auto_decompose_max_tier_check;

alter table vip_potion_settings
  add constraint vip_potion_settings_auto_decompose_max_tier_check
  check (auto_decompose_max_tier between 0 and 13);

create table if not exists character_mechanism_cooldowns (
  character_id bigint not null references characters(id) on delete cascade,
  mechanism text not null,
  ready_at timestamptz not null,
  updated_at timestamptz not null default now(),
  primary key (character_id, mechanism)
);

update item_templates
set name = replace(replace(replace(name, '神阶', '九阶'), '仙阶', '十阶'), '圣阶', '十一阶')
where name like '%神阶%' or name like '%仙阶%' or name like '%圣阶%';

update item_templates
set rarity = 'mythic',
    stats = jsonb_set(stats, '{tier}', '13'::jsonb, true),
    flags = flags || '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"tier13_2026"}'::jsonb
where id in (
  'dominator_blade',
  'dominator_armor',
  'dominator_helm',
  'dominator_boots',
  'dominator_belt',
  'dominator_necklace',
  'dominator_bracelet',
  'dominator_ring'
);

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('blade_huyue_t11', '天狐战刃', 'weapon', 'weapon', 'legendary', 620000, false,
   '{"atk":1450,"mag":450,"luck":260,"crit_pct":16,"ignore_def_pct":12,"guaranteed_hit_pct":10,"heavy_hit_pct":10,"tier":11,"score":4800,"affix_count":7,"special_mechanism":"天狐穿透"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),
  ('armor_huyue_t11', '天狐羽衣', 'armor', 'chest', 'legendary', 610000, false,
   '{"def":1200,"mdef":500,"hp":12000,"luck":140,"damage_reduce_pct":4,"paralyze_resist_pct":6,"petrify_resist_pct":6,"tier":11,"score":4700,"affix_count":7,"special_mechanism":"天狐闪影"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),
  ('helm_huyue_t11', '天狐灵盔', 'armor', 'head', 'legendary', 540000, false,
   '{"def":720,"mdef":260,"hp":5200,"luck":120,"guaranteed_hit_pct":5,"paralyze_resist_pct":4,"petrify_resist_pct":4,"tier":11,"score":3600,"affix_count":7,"special_mechanism":"天狐灵视"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),
  ('neck_huyue_t11', '天狐之坠', 'accessory', 'neck', 'legendary', 560000, false,
   '{"atk":650,"mag":650,"luck":260,"ignore_def_pct":8,"guaranteed_hit_pct":8,"crit_damage_pct":25,"tier":11,"score":3900,"affix_count":7,"special_mechanism":"天狐洞察"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),
  ('bracelet_huyue_t11', '天狐幻镯', 'accessory', 'bracelet_left', 'legendary', 550000, false,
   '{"atk":620,"mag":520,"def":280,"mdef":200,"luck":240,"ignore_def_pct":6,"guaranteed_hit_pct":6,"tier":11,"score":3820,"affix_count":7,"special_mechanism":"天狐幻步"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),
  ('ring_huyue_t11', '天狐幻戒', 'accessory', 'ring_left', 'legendary', 570000, false,
   '{"atk":720,"mag":600,"luck":260,"crit_pct":18,"crit_damage_pct":40,"ignore_def_pct":8,"tier":11,"score":4100,"affix_count":7,"special_mechanism":"天狐破法"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),
  ('belt_huyue_t11', '天狐束带', 'armor', 'waist', 'legendary', 520000, false,
   '{"def":680,"mdef":260,"hp":6500,"luck":110,"damage_reduce_pct":4,"guaranteed_hit_pct":4,"tier":11,"score":3400,"affix_count":7,"special_mechanism":"天狐轻身"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),
  ('boots_huyue_t11', '天狐踏云靴', 'armor', 'feet', 'legendary', 520000, false,
   '{"def":620,"mdef":220,"hp":5600,"luck":180,"guaranteed_hit_pct":6,"damage_reduce_pct":3,"tier":11,"score":3350,"affix_count":7,"special_mechanism":"天狐踏云"}'::jsonb,
   '{"exclusive_source":"huyue","quality_standard":"tier11_2026"}'::jsonb),

  ('blade_chaos_t12', '混沌开天斩', 'weapon', 'weapon', 'mythic', 920000, false,
   '{"atk":1900,"mag":900,"atk_pct":12,"crit_pct":18,"ignore_def_pct":10,"boss_damage_pct":10,"tier":12,"score":6500,"affix_count":8,"special_mechanism":"混沌开天"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),
  ('armor_chaos_t12', '混沌魔铠', 'armor', 'chest', 'mythic', 900000, false,
   '{"def":1700,"mdef":700,"hp":22000,"hp_pct":10,"damage_reduce_pct":12,"paralyze_resist_pct":8,"petrify_resist_pct":8,"tier":12,"score":6400,"affix_count":8,"special_mechanism":"混沌魔铠"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),
  ('helm_chaos_t12', '混沌虚空盔', 'armor', 'head', 'mythic', 800000, false,
   '{"def":1000,"mdef":380,"hp":9000,"hp_pct":6,"damage_reduce_pct":8,"paralyze_resist_pct":6,"petrify_resist_pct":6,"tier":12,"score":5000,"affix_count":8,"special_mechanism":"虚空护念"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),
  ('neck_chaos_t12', '混沌深渊链', 'accessory', 'neck', 'mythic', 820000, false,
   '{"atk":880,"mag":880,"hp":4000,"atk_pct":8,"crit_damage_pct":40,"boss_damage_pct":8,"tier":12,"score":5200,"affix_count":8,"special_mechanism":"深渊回响"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),
  ('bracelet_chaos_t12', '混沌破灭镯', 'accessory', 'bracelet_left', 'mythic', 810000, false,
   '{"atk":820,"mag":720,"def":420,"mdef":300,"hp":5000,"damage_reduce_pct":5,"ignore_def_pct":8,"tier":12,"score":5100,"affix_count":8,"special_mechanism":"混沌破灭"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),
  ('ring_chaos_t12', '混沌湮灭戒', 'accessory', 'ring_left', 'mythic', 840000, false,
   '{"atk":980,"mag":820,"hp":5000,"crit_pct":20,"crit_damage_pct":60,"ignore_def_pct":8,"tier":12,"score":5500,"affix_count":8,"special_mechanism":"混沌湮灭"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),
  ('belt_chaos_t12', '混沌锁魂带', 'armor', 'waist', 'mythic', 780000, false,
   '{"def":980,"mdef":420,"hp":11000,"hp_pct":8,"damage_reduce_pct":8,"paralyze_resist_pct":5,"petrify_resist_pct":5,"tier":12,"score":4850,"affix_count":8,"special_mechanism":"混沌锁魂"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),
  ('boots_chaos_t12', '混沌碎星靴', 'armor', 'feet', 'mythic', 780000, false,
   '{"def":900,"mdef":360,"hp":9000,"damage_reduce_pct":7,"guaranteed_hit_pct":8,"paralyze_resist_pct":5,"petrify_resist_pct":5,"tier":12,"score":4700,"affix_count":8,"special_mechanism":"混沌碎星"}'::jsonb,
   '{"exclusive_source":"chaos_abyss","quality_standard":"tier12_2026"}'::jsonb),

  ('belt_taizi_small', '小太子奶', 'accessory', 'bracelet_left', 'mythic', 2000000, false,
   '{"hp":1400,"mp":1400,"atk":1200,"mag":1200,"life_steal_pct":1,"mana_steal_pct":1,"paralyze_resist_pct":30,"petrify_resist_pct":30,"origin_revive_cd_seconds":30,"tier":13,"score":9000,"special_mechanism":"原地复活","special_mechanism_extra":"30秒冷却"}'::jsonb,
   '{"shop":"yuanbao","yuanbao_price":2000,"exclusive_source":"yuanbao_shop","quality_standard":"special_mythic_2026"}'::jsonb),
  ('ring_blood_shadow', '血色幽影', 'accessory', 'ring_left', 'mythic', 4000000, false,
   '{"hp":2000,"mp":2000,"atk":2000,"mag":2000,"crit_pct":50,"crit_damage_pct":150,"life_steal_pct":1,"mana_steal_pct":1,"death_drop_immune":true,"tier":13,"score":12000,"special_mechanism":"不朽防爆"}'::jsonb,
   '{"guild_shop":"blood_shadow","exclusive_source":"guild_shop","quality_standard":"special_mythic_2026"}'::jsonb)
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;

update inventory_items
set location = 'bag', slot = null
where template_id = 'belt_taizi_small' and location = 'equipped';

update treasures
set level = least(greatest(level, 1), 200),
    stage = least(greatest(stage, 1), 20);

update pets
set level = least(greatest(level, 1), 200);

update cultivation_states
set layer = least(greatest(layer, 1), 81);
