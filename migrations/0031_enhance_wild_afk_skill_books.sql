insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('snake_valley', 'camp', '毒蛇山谷安全区', '山谷商队扎下的临时营地，火盆和解毒草药守住了一小块安全地带。', '{"毒蛇山谷入口":"valley_gate","山谷深处":"valley_depth"}', '[]', true),
  ('cangyue', 'safe_harbor', '苍月岛安全区', '海港木栈道外潮声不歇，药师和船夫在这里接应远征者。', '{"苍月岛海岸":"island","盟重土城":"mengzhong:town"}', '[]', true)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set exits = exits || '{"毒蛇营地":"camp"}'::jsonb
where zone_id = 'snake_valley' and id = 'valley_gate';

update world_rooms
set exits = exits || '{"苍月安全区":"safe_harbor"}'::jsonb
where zone_id = 'cangyue' and id = 'island';

insert into skills (id, name, class, min_level, mp_cost, cooldown_ms, config) values
  ('basic_sword', '攻杀剑术', 'warrior', 1, 0, 0, '{"kind":"passive","type":"战士基础","desc":"被动增加攻击、防御和少量暴击，10级前主要通过安全区挂机升级。","source":"比奇新手村、比奇森林、无尽塔低层","book_id":"book_basic_sword","requires_book":false,"atk_bonus":3,"def_bonus":1,"crit_bonus":0}'),
  ('warrior_body', '铁布衫', 'warrior', 8, 0, 0, '{"kind":"passive","type":"战士防护","desc":"被动增加物理防御和生命。","source":"比奇矿洞、尸王、无尽塔 5 层后","book_id":"book_warrior_body","requires_book":true,"def_bonus":3,"hp_bonus":25}'),
  ('halfmoon_blade', '半月弯刀', 'warrior', 18, 0, 0, '{"kind":"passive","type":"战士进阶","desc":"被动增加攻击和敏捷。","source":"毒蛇山谷、沃玛森林、沃玛神庙","book_id":"book_halfmoon_blade","requires_book":true,"atk_bonus":5,"dex_bonus":1}'),
  ('flame_blade', '烈火剑意', 'warrior', 28, 0, 0, '{"kind":"passive","type":"战士高阶","desc":"被动增加攻击、生命和暴击。","source":"盟重祖玛神庙、世界首领、苍月岛高阶怪物","book_id":"book_flame_blade","requires_book":true,"atk_bonus":8,"hp_bonus":20,"crit_bonus":1}'),

  ('fireball', '火球精通', 'mage', 1, 0, 0, '{"kind":"passive","type":"法师基础","desc":"被动增加魔法攻击、魔法上限和少量暴击。","source":"比奇新手村、比奇森林、无尽塔低层","book_id":"book_fireball","requires_book":false,"mag_bonus":4,"mp_bonus":12,"crit_bonus":0}'),
  ('magic_shield', '魔法盾理', 'mage', 8, 0, 0, '{"kind":"passive","type":"法师防护","desc":"被动增加魔法防御、生命和魔法上限。","source":"比奇矿洞、尸王、无尽塔 5 层后","book_id":"book_magic_shield","requires_book":true,"mdef_bonus":3,"hp_bonus":15,"mp_bonus":20}'),
  ('lightning_master', '雷电术精通', 'mage', 18, 0, 0, '{"kind":"passive","type":"法师进阶","desc":"被动增加魔法攻击和敏捷。","source":"毒蛇山谷、沃玛神庙","book_id":"book_lightning_master","requires_book":true,"mag_bonus":6,"dex_bonus":1}'),
  ('ice_roar', '冰咆哮心法', 'mage', 30, 0, 0, '{"kind":"passive","type":"法师高阶","desc":"被动增加魔法攻击、魔法上限和魔法防御。","source":"盟重祖玛神庙、苍月岛秘境、世界首领","book_id":"book_ice_roar","requires_book":true,"mag_bonus":8,"mp_bonus":30,"mdef_bonus":1}'),

  ('healing_charm', '治愈心法', 'taoist', 1, 0, 0, '{"kind":"passive","type":"道士基础","desc":"被动增加生命、魔法防御和少量道术魔法。","source":"比奇新手村、比奇森林、无尽塔低层","book_id":"book_healing_charm","requires_book":false,"hp_bonus":30,"mdef_bonus":2,"mag_bonus":1}'),
  ('spirit_talisman', '灵魂火符', 'taoist', 8, 0, 0, '{"kind":"passive","type":"道士攻击","desc":"被动增加魔法攻击和少量攻击。","source":"比奇矿洞、尸王、无尽塔 5 层后","book_id":"book_spirit_talisman","requires_book":true,"mag_bonus":4,"atk_bonus":1}'),
  ('summon_guard', '召唤契约', 'taoist', 18, 0, 0, '{"kind":"passive","type":"道士生存","desc":"被动增加生命、物理防御和魔法防御。","source":"毒蛇山谷、沃玛神庙","book_id":"book_summon_guard","requires_book":true,"hp_bonus":45,"def_bonus":2,"mdef_bonus":2}'),
  ('poison_lore', '施毒心经', 'taoist', 28, 0, 0, '{"kind":"passive","type":"道士高阶","desc":"被动增加魔法攻击、敏捷和暴击。","source":"盟重祖玛神庙、世界首领、苍月岛高阶怪物","book_id":"book_poison_lore","requires_book":true,"mag_bonus":5,"dex_bonus":2,"crit_bonus":1}'),

  ('shadow_step', '影袭身法', 'assassin', 1, 0, 0, '{"kind":"passive","type":"刺客基础","desc":"被动增加攻击、敏捷和暴击。","source":"比奇新手村、比奇森林、无尽塔低层","book_id":"book_shadow_step","requires_book":false,"atk_bonus":3,"dex_bonus":2,"crit_bonus":1}'),
  ('night_blade', '夜刃诀', 'assassin', 8, 0, 0, '{"kind":"passive","type":"刺客攻击","desc":"被动增加攻击和敏捷。","source":"比奇矿洞、尸王、无尽塔 5 层后","book_id":"book_night_blade","requires_book":true,"atk_bonus":5,"dex_bonus":1}'),
  ('ghost_body', '鬼步', 'assassin', 18, 0, 0, '{"kind":"passive","type":"刺客身法","desc":"被动增加敏捷、物理防御和魔法防御。","source":"毒蛇山谷、沃玛神庙","book_id":"book_ghost_body","requires_book":true,"dex_bonus":4,"def_bonus":1,"mdef_bonus":1}'),
  ('blood_mark', '血印', 'assassin', 28, 0, 0, '{"kind":"passive","type":"刺客高阶","desc":"被动增加攻击、生命和暴击。","source":"盟重祖玛神庙、世界首领、苍月岛高阶怪物","book_id":"book_blood_mark","requires_book":true,"atk_bonus":6,"hp_bonus":20,"crit_bonus":1}'),

  ('focus_breath', '凝神诀', 'all', 3, 0, 0, '{"kind":"passive","type":"通用基础","desc":"被动增加攻魔、防御、生命和魔法。","source":"比奇城区任务、比奇野外、无尽塔低层","book_id":"book_focus_breath","requires_book":false,"atk_bonus":1,"mag_bonus":1,"def_bonus":1,"mdef_bonus":1,"hp_bonus":10,"mp_bonus":8}'),
  ('field_medicine', '战场调息', 'all', 12, 0, 0, '{"kind":"passive","type":"通用生存","desc":"被动增加生命和魔法上限。","source":"毒蛇山谷、矿洞深处、无尽塔 10 层后","book_id":"book_field_medicine","requires_book":true,"hp_bonus":35,"mp_bonus":15}'),
  ('treasure_sense', '寻宝眼', 'all', 20, 0, 0, '{"kind":"passive","type":"通用身法","desc":"被动增加敏捷和暴击。","source":"沃玛神庙、盟重石墓阵、无尽塔 20 层后","book_id":"book_treasure_sense","requires_book":true,"dex_bonus":3,"crit_bonus":1}'),
  ('iron_will', '百折心法', 'all', 30, 0, 0, '{"kind":"passive","type":"通用高阶","desc":"被动增加生命、物理防御和魔法防御。","source":"盟重祖玛神庙、苍月岛、世界首领","book_id":"book_iron_will","requires_book":true,"hp_bonus":50,"def_bonus":2,"mdef_bonus":2}')
on conflict (id) do update set
  name = excluded.name,
  class = excluded.class,
  min_level = excluded.min_level,
  mp_cost = excluded.mp_cost,
  cooldown_ms = excluded.cooldown_ms,
  config = excluded.config;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats) values
  ('book_basic_sword', '攻杀剑术技能书', 'book', null, 'rare', 700, false, '{"skill":"basic_sword"}'),
  ('book_warrior_body', '铁布衫技能书', 'book', null, 'rare', 1200, false, '{"skill":"warrior_body"}'),
  ('book_halfmoon_blade', '半月弯刀技能书', 'book', null, 'epic', 2400, false, '{"skill":"halfmoon_blade"}'),
  ('book_flame_blade', '烈火剑意技能书', 'book', null, 'legendary', 5200, false, '{"skill":"flame_blade"}'),
  ('book_fireball', '火球精通技能书', 'book', null, 'rare', 700, false, '{"skill":"fireball"}'),
  ('book_magic_shield', '魔法盾理技能书', 'book', null, 'rare', 1200, false, '{"skill":"magic_shield"}'),
  ('book_lightning_master', '雷电术精通技能书', 'book', null, 'epic', 2400, false, '{"skill":"lightning_master"}'),
  ('book_ice_roar', '冰咆哮心法技能书', 'book', null, 'legendary', 5200, false, '{"skill":"ice_roar"}'),
  ('book_healing_charm', '治愈心法技能书', 'book', null, 'rare', 700, false, '{"skill":"healing_charm"}'),
  ('book_spirit_talisman', '灵魂火符技能书', 'book', null, 'rare', 1200, false, '{"skill":"spirit_talisman"}'),
  ('book_summon_guard', '召唤契约技能书', 'book', null, 'epic', 2400, false, '{"skill":"summon_guard"}'),
  ('book_poison_lore', '施毒心经技能书', 'book', null, 'legendary', 5200, false, '{"skill":"poison_lore"}'),
  ('book_shadow_step', '影袭身法技能书', 'book', null, 'rare', 700, false, '{"skill":"shadow_step"}'),
  ('book_night_blade', '夜刃诀技能书', 'book', null, 'rare', 1200, false, '{"skill":"night_blade"}'),
  ('book_ghost_body', '鬼步技能书', 'book', null, 'epic', 2400, false, '{"skill":"ghost_body"}'),
  ('book_blood_mark', '血印技能书', 'book', null, 'legendary', 5200, false, '{"skill":"blood_mark"}'),
  ('book_focus_breath', '凝神诀技能书', 'book', null, 'rare', 650, false, '{"skill":"focus_breath"}'),
  ('book_field_medicine', '战场调息技能书', 'book', null, 'rare', 1300, false, '{"skill":"field_medicine"}'),
  ('book_treasure_sense', '寻宝眼技能书', 'book', null, 'epic', 2600, false, '{"skill":"treasure_sense"}'),
  ('book_iron_will', '百折心法技能书', 'book', null, 'legendary', 5600, false, '{"skill":"iron_will"}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats;
