-- 0.2.12: teleport-friendly safe zones, 100-level skill curve support, and stronger mid-game growth systems.

insert into skills (id, name, class, min_level, mp_cost, cooldown_ms, config) values
  (
    'talent_battle_instinct',
    '战斗本能',
    'all',
    1,
    0,
    0,
    '{"kind":"passive","type":"通用天赋","desc":"无技能书通用被动，安全区挂机可训练，提供全职业基础战斗属性。","source":"角色 1 级后直接学习","requires_book":false,"atk_bonus":4,"mag_bonus":4,"def_bonus":3,"mdef_bonus":3,"hp_bonus":120,"mp_bonus":60,"crit_bonus":1}'
  )
on conflict (id) do update set
  name = excluded.name,
  class = excluded.class,
  min_level = excluded.min_level,
  mp_cost = excluded.mp_cost,
  cooldown_ms = excluded.cooldown_ms,
  config = excluded.config;

update skills
set config = config || '{"hp_bonus":80,"mdef_bonus":8}'::jsonb
where id = 'ghost_body';

update skills
set config = config || '{"mp_bonus":140,"hp_bonus":120,"mdef_bonus":12}'::jsonb
where id = 'magic_shield';

update skills
set config = config || '{"hp_bonus":120,"def_bonus":8,"mdef_bonus":8}'::jsonb
where id = 'summon_guard';

update skills
set config = config || '{"def_bonus":10,"mdef_bonus":10,"hp_bonus":100}'::jsonb
where id = 'iron_will';

update skills
set config = config || '{"hp_bonus":180,"mag_bonus":12,"def_bonus":8,"mdef_bonus":8}'::jsonb
where id = 'blood_mark';

update skills
set config = config || '{"atk_bonus":10,"luck_bonus":2}'::jsonb
where id = 'basic_sword';

update skills
set config = config || '{"mag_bonus":10,"luck_bonus":2}'::jsonb
where id = 'focus_breath';

insert into pet_templates (id, name, rarity, base_hp, base_atk, skills) values
  ('pet_white_tiger', '白虎幼崽', 'rare', 620, 85, '["撕咬","护主","巡猎","破甲"]'),
  ('pet_fire_bird', '烈焰灵鸟', 'epic', 520, 118, '["火羽","灼烧","灵压","燃魂"]')
on conflict (id) do update set
  name = excluded.name,
  rarity = excluded.rarity,
  base_hp = excluded.base_hp,
  base_atk = excluded.base_atk,
  skills = excluded.skills;

insert into treasure_templates (id, name, family, passive, config) values
  (
    'treasure_dragon_seal',
    '龙纹印',
    'dragon',
    '主攻法宝：大幅提升攻击、魔法、生命、幸运和暴击，适合中期推进祖玛、苍月和首领战。',
    '{"atk_pct":12,"mag_pct":12,"hp_pct":8,"mp_pct":4,"crit_pct":4,"luck":20,"boss_drop_pct":4}'
  ),
  (
    'treasure_guard_mirror',
    '玄光镜',
    'guard',
    '守护法宝：大幅提升生命、物防、魔防和魔法上限，适合挂机、越级探索和高阶 Boss。',
    '{"hp_pct":16,"def_pct":10,"mdef_pct":12,"mp_pct":8,"damage_reduce_pct":3}'
  )
on conflict (id) do update set
  name = excluded.name,
  family = excluded.family,
  passive = excluded.passive,
  config = excluded.config;

insert into character_system_unlocks (character_id, system_id, source_mob_id)
select character_id, 'pet', 'woma_lord'
from character_system_unlocks
where system_id = 'treasure'
on conflict (character_id, system_id) do nothing;
