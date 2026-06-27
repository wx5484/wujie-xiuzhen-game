insert into skills (id, name, class, min_level, mp_cost, cooldown_ms, config) values
  ('basic_sword', '攻杀剑术', 'warrior', 1, 0, 0, '{"kind":"passive","type":"攻击被动","desc":"挂机提升熟练度，直接继承为攻击、防御和暴击。","atk_bonus":8,"def_bonus":2,"crit_bonus":1}'),
  ('fireball', '火球精通', 'mage', 1, 0, 0, '{"kind":"passive","type":"法术被动","desc":"挂机提升熟练度，直接继承为魔法、魔法上限和少量暴击。","mag_bonus":10,"mp_bonus":30,"crit_bonus":1}'),
  ('healing_charm', '治愈心法', 'taoist', 1, 0, 0, '{"kind":"passive","type":"生存被动","desc":"挂机提升熟练度，直接继承为生命、魔防和道术魔法。","hp_bonus":80,"mdef_bonus":4,"mag_bonus":4}'),
  ('shadow_step', '影袭身法', 'assassin', 1, 0, 0, '{"kind":"passive","type":"身法被动","desc":"挂机提升熟练度，直接继承为攻击、敏捷和暴击。","atk_bonus":6,"dex_bonus":5,"crit_bonus":3}'),
  ('focus_breath', '凝神诀', 'all', 3, 0, 0, '{"kind":"passive","type":"通用被动","desc":"挂机提升熟练度，直接继承为攻魔、防御、生命和魔法。","atk_bonus":2,"mag_bonus":2,"def_bonus":2,"mdef_bonus":2,"hp_bonus":30,"mp_bonus":20,"crit_bonus":1}')
on conflict (id) do update set
  name = excluded.name,
  class = excluded.class,
  min_level = excluded.min_level,
  mp_cost = excluded.mp_cost,
  cooldown_ms = excluded.cooldown_ms,
  config = excluded.config;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats) values
  ('pill_cultivate', '培养丹', 'consumable', null, 'rare', 120, true, '{"exp":260}'),
  ('pill_insight', '悟性丹', 'consumable', null, 'epic', 500, true, '{"skill_proficiency":80}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats;
