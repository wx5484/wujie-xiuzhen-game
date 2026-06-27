-- 0.2.12: real same-room PK targets, final T1-T17 equipment cleanup and potion rebalance.

with old_templates as (
  select id
  from item_templates
  where kind in ('weapon', 'armor', 'accessory')
    and id not like 't__\_%' escape '\'
    and id not like 'dominator_%'
    and id not in ('belt_taizi_small', 'ring_blood_shadow')
)
update consignments
set status = 'cancelled',
    item_id = null,
    updated_at = now()
where item_id in (
  select ii.id
  from inventory_items ii
  join old_templates ot on ot.id = ii.template_id
);

with old_templates as (
  select id
  from item_templates
  where kind in ('weapon', 'armor', 'accessory')
    and id not like 't__\_%' escape '\'
    and id not like 'dominator_%'
    and id not in ('belt_taizi_small', 'ring_blood_shadow')
)
update mail_attachments
set item_template_id = null,
    updated_at = now()
where item_template_id in (select id from old_templates);

with old_templates as (
  select id
  from item_templates
  where kind in ('weapon', 'armor', 'accessory')
    and id not like 't__\_%' escape '\'
    and id not like 'dominator_%'
    and id not in ('belt_taizi_small', 'ring_blood_shadow')
)
delete from inventory_items
where template_id in (select id from old_templates);

delete from item_templates
where kind in ('weapon', 'armor', 'accessory')
  and id not like 't__\_%' escape '\'
  and id not like 'dominator_%'
  and id not in ('belt_taizi_small', 'ring_blood_shadow');

with keep_books(id) as (
  values
    ('book_sword_wanfa'), ('book_sword_dayan'), ('book_sword_clear_heart'), ('book_sword_zhuxian'),
    ('book_spell_xuan_light'), ('book_spell_resonance'), ('book_spell_chaos_orb'), ('book_spell_thunder_array'),
    ('book_soul_karma_fire'), ('book_soul_reincarnation'), ('book_soul_all_unity'), ('book_soul_yanluo_prison')
),
old_books as (
  select it.id
  from item_templates it
  left join keep_books kb on kb.id = it.id
  where it.kind = 'book' and kb.id is null
)
update consignments
set status = 'cancelled',
    item_id = null,
    updated_at = now()
where item_id in (
  select ii.id
  from inventory_items ii
  join old_books ob on ob.id = ii.template_id
);

with keep_books(id) as (
  values
    ('book_sword_wanfa'), ('book_sword_dayan'), ('book_sword_clear_heart'), ('book_sword_zhuxian'),
    ('book_spell_xuan_light'), ('book_spell_resonance'), ('book_spell_chaos_orb'), ('book_spell_thunder_array'),
    ('book_soul_karma_fire'), ('book_soul_reincarnation'), ('book_soul_all_unity'), ('book_soul_yanluo_prison')
),
old_books as (
  select it.id
  from item_templates it
  left join keep_books kb on kb.id = it.id
  where it.kind = 'book' and kb.id is null
)
update mail_attachments
set item_template_id = null,
    updated_at = now()
where item_template_id in (select id from old_books);

with keep_books(id) as (
  values
    ('book_sword_wanfa'), ('book_sword_dayan'), ('book_sword_clear_heart'), ('book_sword_zhuxian'),
    ('book_spell_xuan_light'), ('book_spell_resonance'), ('book_spell_chaos_orb'), ('book_spell_thunder_array'),
    ('book_soul_karma_fire'), ('book_soul_reincarnation'), ('book_soul_all_unity'), ('book_soul_yanluo_prison')
),
old_books as (
  select it.id
  from item_templates it
  left join keep_books kb on kb.id = it.id
  where it.kind = 'book' and kb.id is null
)
delete from inventory_items
where template_id in (select id from old_books);

with keep_books(id) as (
  values
    ('book_sword_wanfa'), ('book_sword_dayan'), ('book_sword_clear_heart'), ('book_sword_zhuxian'),
    ('book_spell_xuan_light'), ('book_spell_resonance'), ('book_spell_chaos_orb'), ('book_spell_thunder_array'),
    ('book_soul_karma_fire'), ('book_soul_reincarnation'), ('book_soul_all_unity'), ('book_soul_yanluo_prison')
)
delete from item_templates it
where it.kind = 'book'
  and not exists (select 1 from keep_books kb where kb.id = it.id);

with tier_defs(tier, series, set_id) as (
  values
    (1, '凡尘系列', null),
    (2, '玄铁系列', null),
    (3, '青云套装', 'qingyun'),
    (4, '地煞系列', null),
    (5, '天罡系列', null),
    (6, '纯阳套装', 'pureyang'),
    (7, '星陨系列', null),
    (8, '幽冥系列', null),
    (9, '九霄套装', 'jiuxiao'),
    (10, '神渊系列', null),
    (11, '天狐系列', null),
    (12, '混沌套装', 'chaos'),
    (13, '太初系列', null),
    (14, '洪荒系列', null),
    (15, '造化套装', 'zaohua'),
    (16, '涅槃系列', null),
    (17, '鸿蒙系列', null)
),
slot_defs(slot_id, kind, slot, slot_class) as (
  values
    ('weapon', 'weapon', 'weapon', 'weapon'),
    ('chest', 'armor', 'chest', 'armor'),
    ('head', 'armor', 'head', 'armor'),
    ('neck', 'accessory', 'neck', 'accessory'),
    ('bracelet', 'accessory', 'bracelet_left', 'accessory'),
    ('ring', 'accessory', 'ring_left', 'accessory'),
    ('waist', 'accessory', 'waist', 'accessory'),
    ('feet', 'armor', 'feet', 'armor')
),
piece_names(tier, slot_id, name) as (
  values
    (1, 'weapon', '破旧铁剑'), (1, 'chest', '粗布麻衣'), (1, 'head', '凡尘头巾'), (1, 'neck', '凡尘项链'), (1, 'bracelet', '凡尘手镯'), (1, 'ring', '凡尘戒指'), (1, 'waist', '凡尘腰带'), (1, 'feet', '凡尘布靴'),
    (2, 'weapon', '精炼长刃'), (2, 'chest', '玄铁重铠'), (2, 'head', '玄铁头盔'), (2, 'neck', '玄铁项链'), (2, 'bracelet', '玄铁手镯'), (2, 'ring', '玄铁戒指'), (2, 'waist', '玄铁腰带'), (2, 'feet', '玄铁战靴'),
    (3, 'weapon', '青云剑'), (3, 'chest', '青云法衣'), (3, 'head', '青云冠'), (3, 'neck', '青云链'), (3, 'bracelet', '青云镯'), (3, 'ring', '青云戒'), (3, 'waist', '青云束带'), (3, 'feet', '青云履'),
    (4, 'weapon', '地煞刃'), (4, 'chest', '地煞战衣'), (4, 'head', '地煞盔'), (4, 'neck', '地煞链'), (4, 'bracelet', '地煞镯'), (4, 'ring', '地煞戒'), (4, 'waist', '地煞腰封'), (4, 'feet', '地煞靴'),
    (5, 'weapon', '天罡剑'), (5, 'chest', '天罡战甲'), (5, 'head', '天罡冠'), (5, 'neck', '天罡项链'), (5, 'bracelet', '天罡手镯'), (5, 'ring', '天罡戒指'), (5, 'waist', '天罡腰带'), (5, 'feet', '天罡战靴'),
    (6, 'weapon', '纯阳神剑'), (6, 'chest', '纯阳道袍'), (6, 'head', '纯阳冠'), (6, 'neck', '纯阳链'), (6, 'bracelet', '纯阳镯'), (6, 'ring', '纯阳戒'), (6, 'waist', '纯阳腰封'), (6, 'feet', '纯阳履'),
    (7, 'weapon', '星陨刃'), (7, 'chest', '星陨甲'), (7, 'head', '星陨盔'), (7, 'neck', '星陨链'), (7, 'bracelet', '星陨镯'), (7, 'ring', '星陨戒'), (7, 'waist', '星陨束带'), (7, 'feet', '星陨靴'),
    (8, 'weapon', '幽冥刃'), (8, 'chest', '幽冥甲'), (8, 'head', '幽冥盔'), (8, 'neck', '幽冥链'), (8, 'bracelet', '幽冥镯'), (8, 'ring', '幽冥戒'), (8, 'waist', '幽冥腰封'), (8, 'feet', '幽冥靴'),
    (9, 'weapon', '九霄雷剑'), (9, 'chest', '九霄法衣'), (9, 'head', '九霄冠'), (9, 'neck', '九霄链'), (9, 'bracelet', '九霄镯'), (9, 'ring', '九霄戒'), (9, 'waist', '九霄束带'), (9, 'feet', '九霄履'),
    (10, 'weapon', '神渊断界刃'), (10, 'chest', '神渊战衣'), (10, 'head', '神渊盔'), (10, 'neck', '神渊链'), (10, 'bracelet', '神渊镯'), (10, 'ring', '神渊戒'), (10, 'waist', '神渊腰封'), (10, 'feet', '神渊靴'),
    (11, 'weapon', '天狐幻刃'), (11, 'chest', '天狐灵衣'), (11, 'head', '天狐冠'), (11, 'neck', '天狐链'), (11, 'bracelet', '天狐镯'), (11, 'ring', '天狐戒'), (11, 'waist', '天狐腰带'), (11, 'feet', '天狐靴'),
    (12, 'weapon', '混沌神兵'), (12, 'chest', '混沌战甲'), (12, 'head', '混沌盔'), (12, 'neck', '混沌链'), (12, 'bracelet', '混沌镯'), (12, 'ring', '混沌戒'), (12, 'waist', '混沌腰封'), (12, 'feet', '混沌靴'),
    (13, 'weapon', '太初源刃'), (13, 'chest', '太初源甲'), (13, 'head', '太初冠'), (13, 'neck', '太初链'), (13, 'bracelet', '太初镯'), (13, 'ring', '太初戒'), (13, 'waist', '太初腰带'), (13, 'feet', '太初靴'),
    (14, 'weapon', '洪荒战戟'), (14, 'chest', '洪荒战衣'), (14, 'head', '洪荒盔'), (14, 'neck', '洪荒链'), (14, 'bracelet', '洪荒镯'), (14, 'ring', '洪荒戒'), (14, 'waist', '洪荒腰封'), (14, 'feet', '洪荒靴'),
    (15, 'weapon', '造化神剑'), (15, 'chest', '造化仙衣'), (15, 'head', '造化冠'), (15, 'neck', '造化链'), (15, 'bracelet', '造化镯'), (15, 'ring', '造化戒'), (15, 'waist', '造化束带'), (15, 'feet', '造化履'),
    (16, 'weapon', '涅槃火刃'), (16, 'chest', '涅槃火甲'), (16, 'head', '涅槃火冠'), (16, 'neck', '涅槃火链'), (16, 'bracelet', '涅槃火镯'), (16, 'ring', '涅槃火戒'), (16, 'waist', '涅槃火带'), (16, 'feet', '涅槃火靴'),
    (17, 'weapon', '鸿蒙道兵'), (17, 'chest', '鸿蒙道甲'), (17, 'head', '鸿蒙道冠'), (17, 'neck', '鸿蒙道链'), (17, 'bracelet', '鸿蒙道镯'), (17, 'ring', '鸿蒙道戒'), (17, 'waist', '鸿蒙道带'), (17, 'feet', '鸿蒙道靴')
),
generated as (
  select
    't' || lpad(td.tier::text, 2, '0') || '_' || sd.slot_id as id,
    pn.name,
    sd.kind,
    sd.slot,
    case
      when td.tier <= 1 then 'common'
      when td.tier = 2 then 'uncommon'
      when td.tier between 3 and 4 then 'rare'
      when td.tier between 5 and 7 then 'epic'
      when td.tier between 8 and 11 then 'legendary'
      when td.tier between 12 and 14 then 'mythic'
      when td.tier between 15 and 16 then 'supreme'
      else 'ultimate'
    end as rarity,
    (td.tier * td.tier * 5000)::bigint as price,
    jsonb_strip_nulls(jsonb_build_object(
      'tier', td.tier,
      'series', td.series,
      'score', round((case when sd.slot_class = 'armor' then 250 else 120 end) * power(td.tier::double precision, 2.55))::bigint,
      'atk', case when sd.slot_class in ('weapon', 'accessory') then round(50 * power(td.tier::double precision, 2.8))::bigint end,
      'mag', case when sd.slot_class in ('weapon', 'accessory') then round(50 * power(td.tier::double precision, 2.8))::bigint end,
      'hp', case when sd.slot_class = 'armor' then round(250 * power(td.tier::double precision, 2.8))::bigint end,
      'def', case when sd.slot_class = 'armor' then round(20 * power(td.tier::double precision, 2.5))::bigint end,
      'mdef', case when sd.slot_class = 'armor' then round(20 * power(td.tier::double precision, 2.5))::bigint end,
      'life_steal_pct', case when td.tier = 4 and sd.slot_class in ('weapon', 'accessory') then 1 end,
      'crit_pct', case when td.tier = 5 and sd.slot_class in ('weapon', 'accessory') then 2 end,
      'ignore_def_pct', case when td.tier = 8 and sd.slot_class in ('weapon', 'accessory') then least(floor((td.tier - 5) * 1.5), 100)::bigint end,
      'crit_damage_reduce_pct', case when td.tier = 8 and sd.slot_class = 'armor' then 5 end,
      'damage_deepen_pct', case when td.tier = 10 then 5 end,
      'luck', case when td.tier = 11 then 120 end,
      'boss_damage_pct', case when td.tier = 14 then 10 end,
      'origin_revive_cd_seconds', case when td.tier = 16 and sd.slot_id = 'chest' then 3600 end,
      'max_percent_affix_cap', case when td.tier >= 6 then floor((td.tier - 5) * 1.5)::bigint end,
      'set', td.set_id
    )) as stats,
    jsonb_strip_nulls(jsonb_build_object(
      'set', td.set_id,
      'drop_tier', td.tier,
      'slot_class', sd.slot_class,
      'equipment_standard', 'tier_1_17_dynamic_2026'
    )) as flags
  from tier_defs td
  cross join slot_defs sd
  join piece_names pn on pn.tier = td.tier and pn.slot_id = sd.slot_id
)
insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags)
select id, name, kind, slot, rarity, price, false, stats, flags
from generated
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('potion_small', '生命药剂', 'consumable', null, 'common', 1000, true, '{"hp":1000}'::jsonb, '{"shop":"supply"}'::jsonb),
  ('potion_mana', '魔法药剂', 'consumable', null, 'common', 1000, true, '{"mp":1000}'::jsonb, '{"shop":"supply"}'::jsonb),
  ('potion_large', '生命精华', 'consumable', null, 'uncommon', 10000, true, '{"hp":5000}'::jsonb, '{"shop":"supply"}'::jsonb),
  ('potion_mana_large', '魔力精华', 'consumable', null, 'uncommon', 10000, true, '{"mp":1000}'::jsonb, '{"shop":"supply"}'::jsonb),
  ('potion_sun', '小还丹', 'consumable', null, 'rare', 100000, true, '{"hp_pct":10,"mp_pct":10}'::jsonb, '{"shop":"supply"}'::jsonb),
  ('potion_dahuan', '大还丹', 'consumable', null, 'epic', 300000, true, '{"hp_pct":10,"mp_pct":10}'::jsonb, '{"shop":"supply"}'::jsonb),
  ('potion_big_taizi', '护脉丹', 'consumable', null, 'epic', 80, true, '{"hp_pct":40,"mp_pct":40}'::jsonb, '{"shop":"guild","contribution_price":80}'::jsonb),
  ('potion_jiuzhuan', '九转还魂丹', 'consumable', null, 'legendary', 200, true, '{"hp_pct":60,"mp_pct":60}'::jsonb, '{"shop":"guild","contribution_price":200}'::jsonb),
  ('potion_full', '造化丹', 'consumable', null, 'legendary', 10, true, '{"full_restore":true}'::jsonb, '{"shop":"yuanbao","yuanbao_price":10}'::jsonb)
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;
