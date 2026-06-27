insert into skills (id, name, class, min_level, mp_cost, cooldown_ms, config) values
  ('basic_sword', '攻杀剑术', 'warrior', 1, 2, 1000, '{"kind":"physical","power":1.25,"atk_bonus":4,"crit_bonus":1}'),
  ('fireball', '火球术', 'mage', 1, 8, 1200, '{"kind":"magical","power":1.65,"mag_bonus":6}'),
  ('healing_charm', '治愈术', 'taoist', 1, 6, 1500, '{"kind":"heal","heal_power":42,"mag_bonus":3,"crit_bonus":1}'),
  ('shadow_step', '影袭', 'assassin', 1, 4, 900, '{"kind":"physical","power":1.45,"atk_bonus":3,"crit_bonus":3}'),
  ('focus_breath', '凝神诀', 'all', 3, 3, 1000, '{"kind":"physical","power":1.15,"atk_bonus":1,"mag_bonus":1,"crit_bonus":1}')
on conflict (id) do update set
  name = excluded.name,
  class = excluded.class,
  min_level = excluded.min_level,
  mp_cost = excluded.mp_cost,
  cooldown_ms = excluded.cooldown_ms,
  config = excluded.config;
