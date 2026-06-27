-- 0.2.13: challenge rebalance, stargazer reward, special equipment restore.

create table if not exists character_stargazer_visits (
  character_id bigint primary key references characters(id) on delete cascade,
  visits bigint not null default 0,
  awarded boolean not null default false,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

drop trigger if exists character_stargazer_visits_set_updated_at on character_stargazer_visits;
create trigger character_stargazer_visits_set_updated_at before update on character_stargazer_visits
  for each row execute function set_updated_at();

with zone_mobs as (
  select distinct
    wr.zone_id,
    jsonb_array_elements_text(wr.spawns) as mob_id
  from world_rooms wr
  where wr.zone_id in ('xiuzhen', 'feisheng', 'ancient_secret')
),
mob_mult as (
  select
    mob_id,
    max(case
      when zone_id = 'xiuzhen' then 1.15
      when zone_id = 'feisheng' then 1.35
      when zone_id = 'ancient_secret' then 1.75
      else 1.0
    end) as multiplier
  from zone_mobs
  where mob_id <> 'world_boss_eternal_abyss_demon'
  group by mob_id
)
update mob_templates mt
set
  max_hp = greatest(1, round(mt.max_hp::numeric * mm.multiplier))::bigint,
  atk = greatest(1, round(mt.atk::numeric * mm.multiplier))::bigint,
  def = greatest(0, round(mt.def::numeric * mm.multiplier))::bigint,
  version = mt.version + 1,
  updated_at = now()
from mob_mult mm
where mt.id = mm.mob_id;

update mob_templates
set
  max_hp = greatest(1, max_hp * 2),
  atk = greatest(1, atk * 2),
  def = greatest(0, def * 2),
  version = version + 1,
  updated_at = now()
where id = 'world_boss_eternal_abyss_demon';

update mob_spawns ms
set hp = least(greatest(ms.hp, 1), mt.max_hp)
from mob_templates mt
where ms.template_id = mt.id;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  (
    'bracelet_star_devourer',
    '噬星镯',
    'accessory',
    'bracelet_left',
    'ultimate',
    0,
    false,
    '{"special_mechanism":"噬星","special_mechanism_extra":"击杀怪物时有0.05%概率永久增加1点随机基础属性","star_devourer_kill_growth_pct":0.05}'::jsonb,
    '{"exclusive_source":"stargazer_observatory_1000","bind_on_grant":true}'::jsonb
  ),
  (
    'belt_taizi_small',
    '小太子奶',
    'accessory',
    'bracelet_left',
    'mythic',
    2000000,
    false,
    '{"hp":1400,"mp":1400,"atk":1200,"mag":1200,"life_steal_pct":1,"mana_steal_pct":1,"paralyze_resist_pct":30,"petrify_resist_pct":30,"origin_revive_cd_seconds":30,"tier":13,"score":9000,"special_mechanism":"原地复活","special_mechanism_extra":"30秒冷却"}'::jsonb,
    '{"shop":"yuanbao","yuanbao_price":1500,"exclusive_source":"yuanbao_shop","quality_standard":"special_mythic_2026"}'::jsonb
  ),
  (
    'ring_blood_shadow',
    '血色幽影',
    'accessory',
    'ring_left',
    'mythic',
    4000000,
    false,
    '{"hp":2000,"mp":2000,"atk":2000,"mag":2000,"crit_pct":50,"crit_damage_pct":150,"life_steal_pct":1,"mana_steal_pct":1,"death_drop_immune":true,"tier":13,"score":12000,"special_mechanism":"不朽防爆"}'::jsonb,
    '{"guild_shop":"blood_shadow","contribution_price":1000000,"exclusive_source":"guild_shop","quality_standard":"special_mythic_2026"}'::jsonb
  )
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;

update item_templates
set
  stats = stats || '{"yuanbao_decompose":1,"decompose_only":true}'::jsonb,
  flags = flags || '{"source":"boss_boundary_stonemaw","decompose_only":true}'::jsonb,
  updated_at = now()
where id = 'yuanbao_glory';

update item_templates
set
  stats = stats || '{"yuanbao_decompose":2,"decompose_only":true}'::jsonb,
  flags = flags || '{"source":"boss_icefield_overlord","decompose_only":true}'::jsonb,
  updated_at = now()
where id = 'yuanbao_legacy';
