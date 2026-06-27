alter table vip_potion_settings
  add column if not exists auto_decompose_enabled boolean not null default false;

alter table vip_potion_settings
  add column if not exists auto_decompose_max_tier integer not null default 0
    check (auto_decompose_max_tier between 0 and 12);

update item_templates
set price = 10,
    flags = flags || '{"shop":"yuanbao","yuanbao_price":10}'::jsonb
where id = 'potion_full';

alter table skills
  add column if not exists max_level integer not null default 100;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  (
    'vip_month_card',
    '会员资格（30天）',
    'service',
    null,
    'rare',
    300,
    true,
    '{"vip_days":30}'::jsonb,
    '{"shop":"yuanbao","yuanbao_price":300,"service":"vip_month"}'::jsonb
  ),
  (
    'belt_taizi_small',
    '小太子奶',
    'armor',
    'waist',
    'legendary',
    1000,
    false,
    '{"atk_pct":1,"mag_pct":1,"hp_pct":1,"mp_pct":1,"life_steal_pct":1,"mana_steal_pct":1,"tier":0,"score":260,"special_mechanism":"小太子奶"}'::jsonb,
    '{"shop":"yuanbao","yuanbao_price":1000}'::jsonb
  ),
  (
    'yuanbao_glory',
    '荣耀',
    'relic',
    null,
    'legendary',
    1,
    false,
    '{"tier":6,"score":0,"yuanbao_decompose":1,"decompose_only":true}'::jsonb,
    '{"source":"touch_dragon","decompose_only":true}'::jsonb
  ),
  (
    'yuanbao_legacy',
    '传承',
    'relic',
    null,
    'legendary',
    2,
    false,
    '{"tier":8,"score":0,"yuanbao_decompose":2,"decompose_only":true}'::jsonb,
    '{"source":"nether_lord","decompose_only":true}'::jsonb
  ),
  (
    'ring_blood_shadow',
    '血色幽影',
    'accessory',
    'ring_left',
    'mythic',
    1000000000,
    false,
    '{"atk":1000,"mag":1000,"crit_pct":50,"crit_damage_pct":100,"life_steal_pct":1,"mana_steal_pct":1,"death_drop_immune":true,"tier":12,"score":5200,"special_mechanism":"血色幽影"}'::jsonb,
    '{"guild_shop":"blood_shadow"}'::jsonb
  ),
  (
    'potion_big_taizi',
    '大太子奶',
    'consumable',
    null,
    'rare',
    0,
    true,
    '{"hp_pct":50,"mp_pct":50}'::jsonb,
    '{"guild_shop":"big_taizi"}'::jsonb
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

update skills
set max_level = 100,
    config = config || '{"desc":"特殊被动。每级提升攻击、魔法、防御、生命和暴击，100级仍按高阶成长参与战斗。","atk_bonus":4,"mag_bonus":4,"def_bonus":3,"mdef_bonus":3,"hp_bonus":120,"mp_bonus":60,"crit_pct_bonus":1}'::jsonb
where id = 'talent_battle_instinct';

update item_templates
set flags = flags || '{"deprecated":true,"reason":"replaced_by_direct_materials"}'::jsonb
where id in ('pet_food_pack', 'treasure_shard_pack', 'cultivation_pill_pack', 'guardian_gem', 'battle_gem');

delete from item_templates it
where it.id in ('pet_food_pack', 'treasure_shard_pack', 'cultivation_pill_pack', 'guardian_gem', 'battle_gem')
  and not exists (select 1 from inventory_items ii where ii.template_id = it.id)
  and not exists (select 1 from mail_attachments ma where ma.item_template_id = it.id);

update quest_templates
set rewards = replace(replace(replace(replace(replace(rewards::text,
    'pet_food_pack', 'pet_food'),
    'treasure_shard_pack', 'treasure_shard'),
    'cultivation_pill_pack', 'cultivation_pill'),
    'guardian_gem', 'stone_refine'),
    'battle_gem', 'stone_hongmeng')::jsonb
where rewards::text like '%pet_food_pack%'
   or rewards::text like '%treasure_shard_pack%'
   or rewards::text like '%cultivation_pill_pack%'
   or rewards::text like '%guardian_gem%'
   or rewards::text like '%battle_gem%';

update activities
set config = jsonb_set(config #- '{reward,yuanbao}', '{reward,title}', '"屠龙先锋"', true)
where code = 'boss_first_kill';

update world_rooms
set description = description || ' 商人协会在此收购悟性丹，可用10个悟性丹兑换法宝碎片、修炼丹或灵兽粮。'
where zone_id = 'redmoon' and id = 'bairimen_gate' and description not like '%商人协会%';

update world_rooms
set description = description || ' 武学宗师在土城传授特殊技能战斗本能，可用技能书残页逐级突破。'
where zone_id = 'mengzhong' and id = 'town' and description not like '%武学宗师%';

delete from guild_applications
where guild_id in (select id from guilds where name in ('沙城远征队', '比奇守卫军'));

delete from guild_members
where guild_id in (select id from guilds where name in ('沙城远征队', '比奇守卫军'));

delete from guilds
where name in ('沙城远征队', '比奇守卫军');
