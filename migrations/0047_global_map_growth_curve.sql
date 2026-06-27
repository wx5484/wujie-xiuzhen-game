-- 0.2.9: global level 1-200 map curve, Tier 6 areas, and mythic Dominator set.

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('skill_page', '技能书残页', 'material', null, 'rare', 80, true, '{"source":"tower"}', '{"bind_on_reward":true}'),

  ('dominator_blade', '主宰神刃', 'weapon', 'weapon', 'mythic', 1500000, false, '{"atk":1280,"atk_pct":14,"crit_pct":10,"heavy_hit_pct":12,"boss_damage_pct":10,"ignore_def_pct":8,"guaranteed_hit_pct":5,"life_steal_pct":4,"tier":12,"score":4200,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}'),
  ('dominator_armor', '主宰神甲', 'armor', 'chest', 'mythic', 1450000, false, '{"def":1280,"mdef":520,"def_pct":12,"damage_reduce_pct":6,"paralyze_resist_pct":6,"petrify_resist_pct":6,"hp_pct":8,"mp_pct":6,"tier":12,"score":4100,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}'),
  ('dominator_helm', '主宰神盔', 'armor', 'head', 'mythic', 1200000, false, '{"def":760,"mdef":280,"def_pct":8,"damage_reduce_pct":4,"paralyze_resist_pct":5,"petrify_resist_pct":5,"hp_pct":5,"mp_pct":4,"tier":12,"score":3180,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}'),
  ('dominator_boots', '主宰神靴', 'armor', 'feet', 'mythic', 1180000, false, '{"def":700,"mdef":260,"def_pct":7,"damage_reduce_pct":4,"paralyze_resist_pct":5,"petrify_resist_pct":5,"hp_pct":4,"mp_pct":4,"tier":12,"score":3020,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}'),
  ('dominator_belt', '主宰神带', 'armor', 'waist', 'mythic', 1160000, false, '{"def":720,"mdef":300,"def_pct":7,"damage_reduce_pct":4,"paralyze_resist_pct":5,"petrify_resist_pct":5,"hp_pct":6,"mp_pct":3,"tier":12,"score":3100,"affix_count":8,"special_mechanism":"绝对防御","special_mechanism_extra":"控制免疫","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}'),
  ('dominator_necklace', '主宰项链', 'accessory', 'neck', 'mythic', 1220000, false, '{"atk":560,"mag":560,"luck":220,"crit_pct":10,"crit_damage_pct":18,"skill_damage_pct":8,"boss_damage_pct":8,"mana_steal_pct":5,"tier":12,"score":3380,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}'),
  ('dominator_bracelet', '主宰手镯', 'accessory', 'bracelet_left', 'mythic', 1200000, false, '{"atk":520,"mag":420,"def":260,"mdef":180,"luck":210,"crit_pct":8,"heavy_hit_pct":8,"life_steal_pct":5,"tier":12,"score":3260,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}'),
  ('dominator_ring', '主宰戒指', 'accessory', 'ring_left', 'mythic', 1240000, false, '{"atk":620,"mag":470,"luck":230,"crit_pct":12,"crit_damage_pct":22,"heavy_hit_pct":8,"life_steal_pct":4,"mana_steal_pct":4,"tier":12,"score":3500,"affix_count":8,"special_mechanism":"主宰之域","special_mechanism_extra":"创世一击","set":"dominator"}', '{"set":"dominator","exclusive_source":"world_boss","quality_standard":"global_curve_2026"}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = item_templates.flags || excluded.flags;

insert into world_zones (id, name) values
  ('cangyue', '苍月岛'),
  ('fengmo', '封魔谷'),
  ('redmoon', '赤月峡谷'),
  ('molong', '魔龙岭'),
  ('huyue', '狐月神殿'),
  ('ice_city', '冰雪之城'),
  ('chaos_abyss', '混沌深渊')
on conflict (id) do update set name = excluded.name;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('mengzhong', 'town', '盟重土城', '土城是中期远征者集结地，通往石墓、蜈蚣洞、祖玛寺庙、苍月航线和世界首领祭坛。', '{"盟重荒漠":"desert_gate","石墓阵":"stone_tomb","蜈蚣洞":"centipede:entrance","祖玛寺庙":"zuma:entrance","苍月岛":"cangyue:safe_harbor","世界首领":"boss_field"}', '[]', true),
  ('mengzhong', 'boss_field', '盟重祭坛', '四小时一轮的世界首领挑战在这里开启，击杀必定获得主宰套装随机部件。', '{"盟重土城":"town","盟重荒漠":"desert_gate"}', '[]', true),

  ('cangyue', 'safe_harbor', '苍月岛安全区', '海港木栈道外潮声不歇，药师和船夫在这里接应远征者。', '{"盟重土城":"mengzhong:town","苍月岛海岸":"island","幻境牛魔殿":"bull_temple"}', '[]', true),
  ('cangyue', 'island', '苍月岛海岸', '海风里混着妖气，骨魔和牛魔的气息都从内陆传来。推荐 81-88 级探索。', '{"苍月安全区":"safe_harbor","骨魔洞":"bone_cave","幻境牛魔殿":"bull_temple"}', '["cangyue_warrior","sea_demon","bone_soldier"]', false),
  ('cangyue', 'bone_cave', '骨魔洞', '白骨铺满洞窟，骷髅统领和黄泉教主会检验玩家强化与续航。推荐 88-98 级探索。', '{"苍月岛海岸":"island","幻境牛魔殿":"bull_temple"}', '["bone_soldier","bone_general","nether_lord"]', false),
  ('cangyue', 'bull_temple', '幻境牛魔殿', '幻境牛魔王盘踞在古殿深处，击败后正式打开境界养成路线。推荐 98-110 级探索。', '{"苍月安全区":"safe_harbor","骨魔洞":"bone_cave","封魔谷":"fengmo:camp"}', '["bull_guard","bull_warrior","bull_king"]', false),

  ('fengmo', 'camp', '封魔营地', '远征者在山谷口搭起营火，适合从苍月岛补给后继续推进。', '{"幻境牛魔殿":"cangyue:bull_temple","封魔矿道":"mine_path"}', '[]', true),
  ('fengmo', 'mine_path', '封魔矿道', '残破矿道里魔气翻涌，虹魔蝙蝠和封魔卫兵不断涌出。推荐 86-94 级探索。', '{"封魔营地":"camp","霸者大厅":"overlord_hall"}', '["fengmo_bat","fengmo_guard","rainbow_spirit"]', false),
  ('fengmo', 'overlord_hall', '霸者大厅', '封魔旧殿中仍有战鼓回响，虹魔战士和虹魔猪卫巡守大厅。推荐 94-102 级探索。', '{"封魔矿道":"mine_path","封魔祭坛":"altar"}', '["rainbow_warrior","rainbow_boar","rainbow_guard"]', false),
  ('fengmo', 'altar', '封魔祭坛', '虹魔教主残影守住白日门前的最后关口，适合积累突破材料。推荐 102-110 级探索。', '{"霸者大厅":"overlord_hall","白日门":"redmoon:bairimen_gate"}', '["rainbow_guard","rainbow_priest","rainbow_lord"]', false),

  ('redmoon', 'bairimen_gate', '白日门', '白日门是赤月路线前的高阶安全据点，适合补给和整理仓库。', '{"封魔谷":"fengmo:altar","山谷密道":"secret_path"}', '[]', true),
  ('redmoon', 'secret_path', '山谷密道', '石壁潮湿狭窄，月魔蜘蛛沿着洞顶游走。推荐 104-110 级探索。', '{"白日门":"bairimen_gate","抉择之地":"choice_land"}', '["moon_spider","steel_spider","blood_giant"]', false),
  ('redmoon', 'choice_land', '抉择之地', '岔路尽头有赤月气息渗出，血巨人和钢牙蜘蛛守住道路。推荐 110-116 级探索。', '{"山谷密道":"secret_path","恶魔祭坛":"demon_altar"}', '["steel_spider","blood_giant","redmoon_priest"]', false),
  ('redmoon', 'demon_altar', '恶魔祭坛', '祭坛上残火不灭，双头金刚和赤月祭司巡守周围。推荐 116-120 级探索。', '{"抉择之地":"choice_land","赤月峡谷":"canyon"}', '["redmoon_priest","twin_head_guard"]', false),
  ('redmoon', 'canyon', '赤月峡谷', '赤月恶魔拥有 100 级以上 BOSS 强度，开始具备传奇装备极低概率掉落。推荐 120 级探索。', '{"恶魔祭坛":"demon_altar","魔龙岭":"molong:outer"}', '["redmoon_priest","twin_head_guard","redmoon_demon"]', false),

  ('molong', 'outer', '魔龙岭外谷', '魔龙城外的荒岭，魔龙守卫和魔龙刀兵在谷口集结。推荐 121-128 级探索。', '{"赤月峡谷":"redmoon:canyon","魔龙城":"city"}', '["molong_guard","molong_blade"]', false),
  ('molong', 'city', '魔龙城', '魔龙远征军建立的高阶安全区，后期整备与行会集结点。', '{"魔龙岭外谷":"outer","魔龙东郊":"east"}', '[]', true),
  ('molong', 'east', '魔龙东郊', '城外荒土遍布魔龙枪兵和魔龙破甲兵。推荐 128-138 级探索。', '{"魔龙城":"city","魔龙旧寨":"village"}', '["molong_lancer","molong_breaker"]', false),
  ('molong', 'village', '魔龙旧寨', '旧寨残垣里还残留魔龙军阵，魔龙统领在此集结残部。推荐 138-148 级探索。', '{"魔龙东郊":"east","魔龙沼泽":"swamp"}', '["molong_hunter","molong_general"]', false),
  ('molong', 'swamp', '魔龙沼泽', '黑水沼泽会吞没脚步，魔龙战将守着血域入口。推荐 148-156 级探索。', '{"魔龙旧寨":"village","魔龙血域":"bloodland"}', '["molong_general","molong_warrior"]', false),
  ('molong', 'bloodland', '魔龙血域', '魔龙血魔和魔龙教主盘踞在血域尽头，考验吸血、吸蓝和控制抗性。推荐 156-160 级探索。', '{"魔龙沼泽":"swamp","狐月神殿":"huyue:sanctuary"}', '["molong_warrior","molong_blood_demon","molong_lord"]', false),

  ('huyue', 'sanctuary', '狐月神殿营地', '狐月神殿前的临时营地，只为 160 级后的远征者提供短暂喘息。', '{"魔龙血域":"molong:bloodland","月狐古道":"moon_path"}', '[]', true),
  ('huyue', 'moon_path', '月狐古道', '银色残月照亮石阶，狐月妖灵拥有更高命中与压制能力。推荐 161-168 级探索。', '{"狐月营地":"sanctuary","神殿外庭":"outer_court"}', '["huyue_fox_spirit","huyue_moon_guard"]', false),
  ('huyue', 'outer_court', '神殿外庭', '月光在外庭凝成符阵，狐月祭司会持续消耗玩家续航。推荐 168-176 级探索。', '{"月狐古道":"moon_path","狐月祭坛":"altar"}', '["huyue_moon_guard","huyue_priest"]', false),
  ('huyue', 'altar', '狐月祭坛', '狐月天尊守住通往冰雪之城的门扉，高等级特殊属性开始成为硬门槛。推荐 176-182 级探索。', '{"神殿外庭":"outer_court","冰雪之城":"ice_city:camp"}', '["huyue_priest","huyue_god"]', false),

  ('ice_city', 'camp', '冰雪之城营地', '冰墙后的补给点寒风刺骨，远征者在这里准备进入终极深渊。', '{"狐月神殿":"huyue:altar","冰雪城门":"frost_gate"}', '[]', true),
  ('ice_city', 'frost_gate', '冰雪城门', '冰雪骑士和冰魂法师驻守城门，攻击节奏更快且防御更厚。推荐 182-190 级探索。', '{"冰雪营地":"camp","冰封王座":"throne"}', '["ice_knight","ice_wizard"]', false),
  ('ice_city', 'throne', '冰封王座', '冰霜巨人和冰雪女王统治王座，低减伤角色会被迅速击穿。推荐 190-196 级探索。', '{"冰雪城门":"frost_gate","混沌深渊":"chaos_abyss:entrance"}', '["ice_titan","ice_queen"]', false),

  ('chaos_abyss', 'entrance', '混沌入口', '深渊入口扭曲空间，混沌守望者开始具备接近满级的压迫数值。推荐 196-198 级探索。', '{"冰封王座":"ice_city:throne","混沌裂隙":"rift"}', '["chaos_watcher","chaos_devourer"]', false),
  ('chaos_abyss', 'rift', '混沌裂隙', '裂隙中法则错乱，普通小怪也拥有首领级血量和伤害。推荐 198-200 级探索。', '{"混沌入口":"entrance","深渊核心":"core"}', '["chaos_devourer","chaos_watcher"]', false),
  ('chaos_abyss', 'core', '深渊核心', '混沌霸主盘踞在满级前的最后关口，需要极限闪避、减伤和特殊属性支撑。推荐 200 级挑战。', '{"混沌裂隙":"rift"}', '["chaos_devourer","chaos_overlord"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set exits = exits || '{"白日门":"redmoon:bairimen_gate"}'::jsonb
where zone_id = 'bq_plains' and id = 'plains';

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('cangyue_warrior', '苍月妖兵', 82, 30000, 1050, 680, 22000, 4200, false, 360),
  ('sea_demon', '海魔妖', 84, 33000, 1120, 720, 25000, 4700, false, 380),
  ('bone_soldier', '骷髅战将', 86, 36000, 1180, 790, 28000, 5200, false, 400),
  ('bone_general', '骷髅统领', 92, 52000, 1380, 960, 38000, 6800, false, 440),
  ('nether_lord', '黄泉教主', 100, 240000, 2300, 1550, 120000, 22000, true, 2100),
  ('bull_guard', '牛魔护卫', 96, 60000, 1520, 1020, 46000, 7800, false, 460),
  ('bull_warrior', '牛魔战士', 100, 76000, 1700, 1180, 56000, 9200, false, 480),
  ('bull_king', '幻境牛魔王', 108, 420000, 2750, 1900, 180000, 30000, true, 2400),

  ('fengmo_bat', '虹魔蝙蝠', 88, 42000, 1260, 820, 32000, 5600, false, 380),
  ('fengmo_guard', '封魔卫兵', 90, 47000, 1360, 900, 36000, 6200, false, 400),
  ('rainbow_spirit', '虹魔妖灵', 94, 56000, 1500, 1020, 44000, 7300, false, 420),
  ('rainbow_warrior', '虹魔战士', 98, 68000, 1660, 1160, 52000, 8500, false, 440),
  ('rainbow_boar', '虹魔猪卫', 100, 76000, 1780, 1280, 60000, 9600, false, 460),
  ('rainbow_guard', '虹魔护法', 104, 92000, 1980, 1440, 76000, 11600, false, 480),
  ('rainbow_priest', '虹魔祭司', 106, 105000, 2160, 1580, 90000, 13200, false, 500),
  ('rainbow_lord', '虹魔教主', 110, 360000, 2850, 2050, 160000, 27000, true, 2400),

  ('moon_spider', '月魔蜘蛛', 104, 90000, 1900, 1280, 72000, 11600, false, 450),
  ('steel_spider', '钢牙蜘蛛', 110, 112000, 2150, 1480, 88000, 13600, false, 480),
  ('blood_giant', '血巨人', 114, 138000, 2450, 1700, 108000, 16000, false, 520),
  ('redmoon_priest', '赤月祭司', 118, 168000, 2800, 1980, 132000, 19000, false, 560),
  ('twin_head_guard', '双头金刚', 120, 520000, 3600, 2600, 240000, 36000, true, 2100),
  ('redmoon_demon', '赤月恶魔', 120, 760000, 4200, 3100, 320000, 52000, true, 2700),

  ('molong_guard', '魔龙守卫', 124, 180000, 3150, 2200, 130000, 21000, false, 520),
  ('molong_blade', '魔龙刀兵', 128, 220000, 3500, 2500, 160000, 25000, false, 560),
  ('molong_lancer', '魔龙枪兵', 132, 270000, 3900, 2800, 200000, 30000, false, 600),
  ('molong_breaker', '魔龙破甲兵', 138, 340000, 4500, 3300, 250000, 37000, false, 640),
  ('molong_hunter', '魔龙射手', 144, 420000, 5100, 3700, 320000, 46000, false, 680),
  ('molong_general', '魔龙统领', 150, 540000, 5900, 4300, 410000, 58000, false, 720),
  ('molong_warrior', '魔龙战将', 156, 680000, 6800, 5000, 520000, 72000, false, 760),
  ('molong_blood_demon', '魔龙血魔', 158, 1450000, 8500, 6500, 820000, 110000, true, 3000),
  ('molong_lord', '魔龙教主', 160, 2200000, 9800, 7600, 1100000, 150000, true, 3600),

  ('huyue_fox_spirit', '狐月妖灵', 162, 760000, 7600, 5400, 560000, 76000, false, 780),
  ('huyue_moon_guard', '狐月神卫', 168, 900000, 8300, 6100, 660000, 88000, false, 820),
  ('huyue_priest', '狐月祭司', 174, 1100000, 9300, 7000, 820000, 108000, false, 860),
  ('huyue_god', '狐月天尊', 180, 3200000, 12500, 9300, 1500000, 190000, true, 4200),
  ('ice_knight', '冰雪骑士', 184, 1280000, 10500, 7800, 980000, 125000, false, 900),
  ('ice_wizard', '冰魂法师', 188, 1180000, 11800, 7200, 1100000, 138000, false, 920),
  ('ice_titan', '冰霜巨人', 192, 1650000, 12800, 9300, 1300000, 158000, false, 960),
  ('ice_queen', '冰雪女王', 196, 5200000, 15500, 11200, 2100000, 240000, true, 4800),
  ('chaos_watcher', '混沌守望者', 196, 1850000, 13600, 9800, 1500000, 180000, false, 980),
  ('chaos_devourer', '混沌吞噬者', 198, 2200000, 14800, 10800, 1700000, 205000, false, 1000),
  ('chaos_overlord', '混沌霸主', 200, 7200000, 18500, 13500, 2800000, 320000, true, 5400)
on conflict (id) do update set
  name = excluded.name,
  level = excluded.level,
  max_hp = excluded.max_hp,
  atk = excluded.atk,
  def = excluded.def,
  exp = excluded.exp,
  gold = excluded.gold,
  boss = excluded.boss,
  respawn_seconds = excluded.respawn_seconds;
