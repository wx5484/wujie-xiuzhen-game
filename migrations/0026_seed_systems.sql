insert into pet_templates (id, name, rarity, base_hp, base_atk, skills) values
  ('pet_white_tiger', '白虎幼崽', 'rare', 180, 22, '["撕咬","护主"]'),
  ('pet_fire_bird', '烈焰灵鸟', 'epic', 140, 35, '["火羽","灼烧"]')
on conflict (id) do update set
  name = excluded.name,
  rarity = excluded.rarity,
  base_hp = excluded.base_hp,
  base_atk = excluded.base_atk,
  skills = excluded.skills;

insert into treasure_templates (id, name, family, passive, config) values
  ('treasure_dragon_seal', '龙纹印', 'dragon', '攻击与魔法提升，参与 BOSS 战额外增加掉落期望。', '{"atk_pct":3,"mag_pct":3,"boss_drop_pct":2}'),
  ('treasure_guard_mirror', '玄光镜', 'guard', '提升生命与魔防，适合挂机和沙巴克守城。', '{"hp_pct":5,"mdef_pct":4}')
on conflict (id) do update set
  name = excluded.name,
  family = excluded.family,
  passive = excluded.passive,
  config = excluded.config;

insert into sabak_campaigns
  (signup_starts_at, battle_starts_at, battle_ends_at, defending_guild_id, tax_rate_pct, status)
select
  now() - interval '1 day',
  now() + interval '1 day',
  now() + interval '1 day 2 hours',
  (select id from guilds where sabak_owner = true order by id asc limit 1),
  5,
  'signup'
where not exists (select 1 from sabak_campaigns);
