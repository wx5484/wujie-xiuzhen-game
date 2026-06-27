-- 0.2.11: skill shops, same-room player visibility and playable monster curve.

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  (
    'xiuzhen',
    'tianshui_academy',
    '天水书院',
    '天水古城内的书院，可购买各职业初期技能。',
    '{"天水古城":"tianshui_city"}',
    '[]',
    true
  ),
  (
    'feisheng',
    'void_market',
    '虚空市集',
    '虚空要塞内的魔能市集，可购买各职业中期技能。',
    '{"虚空要塞":"void_fortress"}',
    '[]',
    true
  )
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe,
  version = world_rooms.version + 1,
  updated_at = now();

update world_rooms
set
  exits = exits || '{"天水书院":"tianshui_academy"}'::jsonb,
  version = version + 1,
  updated_at = now()
where zone_id = 'xiuzhen' and id = 'tianshui_city';

update world_rooms
set
  exits = exits || '{"虚空市集":"void_market"}'::jsonb,
  version = version + 1,
  updated_at = now()
where zone_id = 'feisheng' and id = 'void_fortress';

with curve as (
  select
    id,
    level,
    boss,
    greatest(25, round(
      case
        when level <= 40 then 30.0 + 18.0 * level::double precision + 8.0 * power(level::double precision, 1.35)
        when level <= 120 then 1800.0 + 95.0 * power((level - 40)::double precision, 1.45)
        when level <= 200 then 52000.0 + 520.0 * power((level - 120)::double precision, 1.55)
        when level <= 300 then 520000.0 + 1250.0 * power((level - 200)::double precision, 1.55)
        when level <= 380 then 2100000.0 + 3000.0 * power((level - 300)::double precision, 1.60)
        when level <= 460 then 5500000.0 + 6000.0 * power((level - 380)::double precision, 1.62)
        else 13000000.0 + 18000.0 * power((level - 460)::double precision, 1.65)
      end
    )::bigint) as normal_hp,
    greatest(3, round(
      case
        when level <= 40 then 4.0 + 2.6 * level::double precision + 0.28 * power(level::double precision, 1.25)
        when level <= 120 then 110.0 + 7.2 * power((level - 40)::double precision, 1.08)
        when level <= 200 then 950.0 + 12.5 * power((level - 120)::double precision, 1.08)
        when level <= 300 then 2500.0 + 22.0 * power((level - 200)::double precision, 1.08)
        when level <= 380 then 6000.0 + 45.0 * power((level - 300)::double precision, 1.08)
        when level <= 460 then 11000.0 + 78.0 * power((level - 380)::double precision, 1.08)
        else 18000.0 + 135.0 * power((level - 460)::double precision, 1.08)
      end
    )::bigint) as normal_atk,
    greatest(0, round(
      case
        when level <= 40 then 1.0 + 0.55 * level::double precision + 0.08 * power(level::double precision, 1.20)
        when level <= 120 then 40.0 + 2.4 * power((level - 40)::double precision, 1.05)
        when level <= 200 then 380.0 + 4.2 * power((level - 120)::double precision, 1.08)
        when level <= 300 then 900.0 + 8.5 * power((level - 200)::double precision, 1.08)
        when level <= 380 then 2100.0 + 16.0 * power((level - 300)::double precision, 1.08)
        when level <= 460 then 3900.0 + 28.0 * power((level - 380)::double precision, 1.08)
        else 6500.0 + 48.0 * power((level - 460)::double precision, 1.08)
      end
    )::bigint) as normal_def
  from mob_templates
  where level between 1 and 600
)
update mob_templates mt
set
  max_hp = case
    when mt.id = 'world_boss_eternal_abyss_demon' then 620000000
    when curve.boss then curve.normal_hp * 6
    else curve.normal_hp
  end,
  atk = case
    when mt.id = 'world_boss_eternal_abyss_demon' then 76000
    when curve.boss then curve.normal_atk * 155 / 100
    else curve.normal_atk
  end,
  def = case
    when mt.id = 'world_boss_eternal_abyss_demon' then 32000
    when curve.boss then curve.normal_def * 125 / 100
    else curve.normal_def
  end,
  version = mt.version + 1,
  updated_at = now()
from curve
where mt.id = curve.id;

update mob_spawns ms
set hp = least(greatest(ms.hp, 1), mt.max_hp)
from mob_templates mt
where ms.template_id = mt.id;
