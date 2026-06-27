-- 0.2.6: light-variant 1.76-inspired progression pass.
-- Keep legacy template ids stable so existing bags, consignments, and drops keep working.

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('sword_wood', '木剑', 'weapon', 'weapon', 'common', 120, false, '{"atk":3,"tier":1,"score":8}', '{}'),
  ('armor_cloth', '布衣', 'armor', 'chest', 'common', 160, false, '{"def":3,"hp":35,"tier":1,"score":10}', '{}'),
  ('ring_copper', '古铜戒指', 'accessory', 'ring_left', 'uncommon', 260, false, '{"atk":2,"luck":1,"tier":1,"score":12}', '{}'),
  ('blade_green_t1', '乌木剑', 'weapon', 'weapon', 'uncommon', 620, false, '{"atk":10,"luck":1,"tier":1,"score":20}', '{}'),
  ('armor_jade_t1', '轻型盔甲', 'armor', 'chest', 'uncommon', 680, false, '{"def":8,"hp":80,"mdef":2,"tier":1,"score":24}', '{}'),
  ('helm_bronze_t1', '青铜头盔', 'armor', 'head', 'uncommon', 520, false, '{"def":5,"hp":50,"tier":1,"score":14}', '{}'),
  ('boots_deerskin_t1', '鹿皮靴', 'armor', 'feet', 'uncommon', 480, false, '{"def":3,"luck":2,"tier":1,"score":12}', '{}'),
  ('bracelet_guard_t1', '铁手镯', 'accessory', 'bracelet_left', 'uncommon', 540, false, '{"atk":3,"def":2,"tier":1,"score":15}', '{}'),

  ('blade_darkiron_t2', '青铜斧', 'weapon', 'weapon', 'rare', 1400, false, '{"atk":22,"mag":3,"tier":2,"score":46}', '{}'),
  ('armor_cloud_t2', '中型盔甲', 'armor', 'chest', 'rare', 1500, false, '{"def":18,"hp":170,"mdef":4,"tier":2,"score":54}', '{}'),
  ('ring_sea_t2', '蓝色水晶戒指', 'accessory', 'ring_left', 'rare', 1700, false, '{"atk":10,"mag":7,"hp":90,"luck":4,"tier":2,"score":48}', '{}'),
  ('neck_jade_t2', '灯笼项链', 'accessory', 'neck', 'rare', 1600, false, '{"mag":10,"hp":90,"luck":3,"tier":2,"score":44}', '{}'),
  ('bracelet_tiger_t2', '大手镯', 'accessory', 'bracelet_left', 'rare', 1500, false, '{"atk":12,"luck":5,"tier":2,"score":45}', '{}'),
  ('blade_serpent_t2', '修罗', 'weapon', 'weapon', 'rare', 1900, false, '{"atk":26,"luck":4,"tier":2,"score":58}', '{}'),
  ('belt_snake_t2', '兽皮腰带', 'armor', 'waist', 'rare', 1320, false, '{"def":10,"mdef":5,"hp":120,"tier":2,"score":42}', '{}'),
  ('boots_cloud_t2', '布鞋', 'armor', 'feet', 'rare', 1260, false, '{"def":8,"luck":5,"hp":80,"tier":2,"score":38}', '{}'),

  ('blade_flame_t3', '井中月', 'weapon', 'weapon', 'epic', 3600, false, '{"atk":42,"mag":8,"luck":6,"tier":3,"score":96}', '{}'),
  ('armor_star_t3', '重盔甲', 'armor', 'chest', 'epic', 3800, false, '{"def":38,"hp":420,"mdef":10,"tier":3,"score":116}', '{}'),
  ('ring_thunder_t3', '龙之戒指', 'accessory', 'ring_left', 'epic', 4200, false, '{"atk":22,"mag":16,"hp":230,"luck":8,"crit_pct":1,"tier":3,"score":112}', '{}'),
  ('armor_woma_t3', '幽灵战衣', 'armor', 'chest', 'epic', 4300, false, '{"def":46,"hp":480,"mdef":12,"tier":3,"score":128}', '{}'),
  ('helm_woma_t3', '道士头盔', 'armor', 'head', 'epic', 3900, false, '{"def":34,"hp":300,"tier":3,"score":96}', '{}'),
  ('neck_woma_t3', '绿色项链', 'accessory', 'neck', 'epic', 4200, false, '{"atk":20,"mag":20,"luck":8,"tier":3,"score":108}', '{}'),
  ('bracelet_woma_t3', '死神手套', 'accessory', 'bracelet_right', 'epic', 4100, false, '{"atk":18,"def":8,"mdef":6,"tier":3,"score":102}', '{}'),
  ('boots_woma_t3', '道士靴', 'armor', 'feet', 'epic', 3600, false, '{"def":24,"luck":9,"hp":220,"tier":3,"score":88}', '{}'),

  ('blade_dragon_t4', '裁决之杖', 'weapon', 'weapon', 'legendary', 9000, false, '{"atk":82,"mag":14,"luck":12,"crit_pct":2,"tier":4,"score":210,"skill_flame_blade_bonus":1}', '{}'),
  ('armor_phoenix_t4', '战神盔甲', 'armor', 'chest', 'legendary', 9600, false, '{"def":82,"hp":880,"mdef":22,"tier":4,"score":248,"petrify_pct":1}', '{}'),
  ('ring_sun_t4', '力量戒指', 'accessory', 'ring_left', 'legendary', 10800, false, '{"atk":46,"mag":34,"hp":480,"luck":16,"crit_pct":2,"tier":4,"score":252}', '{}'),
  ('blade_purgatory_t4', '炼狱', 'weapon', 'weapon', 'epic', 11800, false, '{"atk":98,"luck":14,"heavy_hit_pct":2,"tier":4,"score":270}', '{}'),
  ('armor_dragon_t4', '龙鳞战甲', 'armor', 'chest', 'epic', 11600, false, '{"def":98,"mdef":28,"hp":1100,"tier":4,"score":292}', '{}'),
  ('ring_zuma_t4', '泰坦戒指', 'accessory', 'ring_right', 'epic', 9800, false, '{"atk":52,"mag":38,"hp":540,"luck":14,"crit_pct":2,"tier":4,"score":266}', '{}'),
  ('helm_zuma_t4', '黑铁头盔', 'armor', 'head', 'epic', 9200, false, '{"def":68,"hp":650,"mdef":12,"tier":4,"score":216}', '{}'),
  ('bracelet_zuma_t4', '骑士手镯', 'accessory', 'bracelet_left', 'epic', 9400, false, '{"atk":38,"mag":28,"luck":15,"tier":4,"score":224}', '{}'),

  ('blade_cangyue_t5', '怒斩', 'weapon', 'weapon', 'epic', 32000, false, '{"atk":150,"mag":28,"luck":24,"crit_pct":3,"heavy_hit_pct":3,"tier":5,"score":430}', '{}'),
  ('armor_bull_t5', '圣战宝甲', 'armor', 'chest', 'epic', 34000, false, '{"def":150,"mdef":58,"hp":1800,"tier":5,"score":470}', '{}'),
  ('neck_nether_t5', '圣战项链', 'accessory', 'neck', 'epic', 30000, false, '{"atk":74,"mag":74,"hp":920,"luck":26,"crit_pct":3,"tier":5,"score":430}', '{}'),
  ('boss_relic_blade', '屠龙', 'weapon', 'weapon', 'legendary', 30000, false, '{"atk":132,"mag":28,"luck":22,"heavy_hit_pct":4,"tier":5,"score":380,"skill_halfmoon_blade_bonus":1}', '{}'),
  ('boss_relic_armor', '天魔神甲', 'armor', 'chest', 'legendary', 30000, false, '{"def":132,"mdef":46,"hp":1580,"petrify_pct":1,"tier":5,"score":405}', '{}'),
  ('ring_bull_t5', '圣战戒指', 'accessory', 'ring_right', 'epic', 28000, false, '{"atk":64,"mag":48,"hp":760,"luck":25,"crit_pct":3,"tier":5,"score":390}', '{}'),
  ('bracelet_nether_t5', '圣战手镯', 'accessory', 'bracelet_right', 'epic', 26000, false, '{"atk":58,"def":30,"mdef":22,"luck":24,"tier":5,"score":360}', '{}'),
  ('boots_cangyue_t5', '圣战战靴', 'armor', 'feet', 'epic', 24000, false, '{"def":80,"hp":700,"luck":28,"tier":5,"score":320}', '{}'),

  ('blade_soul_t6', '开天', 'weapon', 'weapon', 'epic', 46000, false, '{"atk":190,"mag":40,"luck":30,"crit_pct":3,"heavy_hit_pct":3,"tier":6,"score":535}', '{}'),
  ('armor_soul_t6', '雷霆战甲', 'armor', 'chest', 'epic', 48000, false, '{"def":198,"mdef":72,"hp":2250,"tier":6,"score":595}', '{}'),
  ('helm_soul_t6', '雷霆头盔', 'armor', 'head', 'epic', 42000, false, '{"def":116,"mdef":30,"hp":980,"tier":6,"score":420}', '{}'),
  ('ring_soul_t6', '雷霆战戒', 'accessory', 'ring_left', 'legendary', 43000, false, '{"atk":84,"mag":68,"hp":920,"luck":32,"crit_pct":3,"mana_steal_pct":2,"tier":6,"score":480}', '{}'),
  ('neck_soul_t6', '雷霆项链', 'accessory', 'neck', 'epic', 43000, false, '{"atk":78,"mag":86,"luck":32,"tier":6,"score":480}', '{}'),
  ('bracelet_soul_t6', '雷霆护腕', 'accessory', 'bracelet_left', 'epic', 41000, false, '{"atk":78,"def":46,"mdef":34,"luck":34,"tier":6,"score":470}', '{}'),

  ('blade_starfall_t7', '镇天', 'weapon', 'weapon', 'legendary', 68000, false, '{"atk":250,"mag":54,"luck":42,"crit_pct":4,"heavy_hit_pct":4,"paralyze_pct":1,"tier":7,"score":730,"skill_ice_roar_bonus":1}', '{}'),
  ('armor_starfall_t7', '烈焰魔衣', 'armor', 'chest', 'epic', 70000, false, '{"def":260,"mdef":98,"hp":3100,"tier":7,"score":805}', '{}'),
  ('helm_starfall_t7', '烈焰头盔', 'armor', 'head', 'epic', 61000, false, '{"def":150,"mdef":42,"hp":1320,"luck":24,"tier":7,"score":570}', '{}'),
  ('ring_starfall_t7', '烈焰魔戒', 'accessory', 'ring_right', 'legendary', 62000, false, '{"atk":110,"mag":90,"hp":1200,"luck":44,"crit_pct":4,"mana_steal_pct":3,"tier":7,"score":650}', '{}'),
  ('neck_starfall_t7', '烈焰项链', 'accessory', 'neck', 'epic', 62000, false, '{"atk":102,"mag":116,"luck":44,"tier":7,"score":660}', '{}'),
  ('bracelet_starfall_t7', '烈焰护腕', 'accessory', 'bracelet_right', 'epic', 59000, false, '{"atk":105,"def":64,"mdef":48,"luck":48,"tier":7,"score":640}', '{}'),

  ('blade_void_t8', '玄天', 'weapon', 'weapon', 'legendary', 92000, false, '{"atk":325,"mag":72,"luck":58,"crit_pct":5,"heavy_hit_pct":5,"petrify_pct":1,"tier":8,"score":940,"skill_poison_lore_bonus":1}', '{}'),
  ('armor_void_t8', '光芒道袍', 'armor', 'chest', 'legendary', 96000, false, '{"def":340,"mdef":132,"hp":4200,"paralyze_pct":1,"tier":8,"score":1030}', '{}'),
  ('helm_void_t8', '光芒头盔', 'armor', 'head', 'epic', 85000, false, '{"def":198,"mdef":58,"hp":1800,"luck":34,"tier":8,"score":740}', '{}'),
  ('ring_void_t8', '光芒道戒', 'accessory', 'ring_left', 'legendary', 86000, false, '{"atk":146,"mag":124,"hp":1600,"luck":62,"crit_pct":5,"mana_steal_pct":4,"tier":8,"score":835}', '{}'),
  ('neck_void_t8', '光芒项链', 'accessory', 'neck', 'epic', 86000, false, '{"atk":138,"mag":154,"luck":62,"tier":8,"score":850}', '{}'),
  ('bracelet_void_t8', '光芒护腕', 'accessory', 'bracelet_left', 'epic', 83000, false, '{"atk":142,"def":88,"mdef":66,"luck":66,"heavy_hit_pct":3,"tier":8,"score":830}', '{}'),

  ('blade_god_t9', '王者之刃', 'weapon', 'weapon', 'legendary', 130000, false, '{"atk":430,"mag":96,"luck":82,"crit_pct":6,"heavy_hit_pct":7,"paralyze_pct":1,"tier":9,"score":1260,"skill_flame_blade_bonus":2}', '{}'),
  ('armor_god_t9', '王者战甲', 'armor', 'chest', 'legendary', 136000, false, '{"def":450,"mdef":178,"hp":5800,"paralyze_pct":1,"petrify_pct":1,"tier":9,"score":1380}', '{}'),
  ('helm_god_t9', '王者战盔', 'armor', 'head', 'epic', 118000, false, '{"def":260,"mdef":82,"hp":2500,"luck":48,"tier":9,"score":980}', '{}'),
  ('ring_god_t9', '王者战戒', 'accessory', 'ring_right', 'legendary', 120000, false, '{"atk":196,"mag":166,"hp":2200,"luck":86,"crit_pct":6,"mana_steal_pct":5,"tier":9,"score":1120}', '{}'),
  ('neck_god_t9', '王者项链', 'accessory', 'neck', 'legendary', 120000, false, '{"atk":188,"mag":210,"luck":86,"heavy_hit_pct":3,"tier":9,"score":1140}', '{}'),
  ('bracelet_god_t9', '王者护腕', 'accessory', 'bracelet_right', 'epic', 116000, false, '{"atk":192,"def":118,"mdef":90,"luck":92,"crit_pct":4,"heavy_hit_pct":5,"tier":9,"score":1120}', '{}'),

  ('blade_immortal_t10', '倚天剑', 'weapon', 'weapon', 'legendary', 190000, false, '{"atk":590,"mag":140,"luck":112,"crit_pct":8,"heavy_hit_pct":10,"paralyze_pct":2,"petrify_pct":1,"tier":10,"score":1730,"skill_flame_blade_bonus":3}', '{}'),
  ('armor_immortal_t10', '凤天魔甲', 'armor', 'chest', 'legendary', 200000, false, '{"def":620,"mdef":245,"hp":8200,"paralyze_pct":2,"petrify_pct":2,"tier":10,"score":1900}', '{}'),
  ('helm_immortal_t10', '星王战盔', 'armor', 'head', 'epic', 176000, false, '{"def":360,"mdef":116,"hp":3600,"luck":70,"tier":10,"score":1360}', '{}'),
  ('ring_immortal_t10', '星王战戒', 'accessory', 'ring_left', 'legendary', 180000, false, '{"atk":270,"mag":230,"hp":3150,"luck":118,"crit_pct":8,"mana_steal_pct":6,"tier":10,"score":1560}', '{}'),
  ('neck_immortal_t10', '星王项链', 'accessory', 'neck', 'legendary', 180000, false, '{"atk":260,"mag":295,"luck":118,"heavy_hit_pct":5,"tier":10,"score":1600}', '{}'),
  ('bracelet_immortal_t10', '星王护腕', 'accessory', 'bracelet_left', 'legendary', 172000, false, '{"atk":265,"def":165,"mdef":125,"luck":130,"crit_pct":6,"heavy_hit_pct":8,"paralyze_pct":2,"tier":10,"score":1580}', '{}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = item_templates.flags || excluded.flags;

update item_templates
set
  stats = stats - 'life_steal_pct',
  rarity = case
    when kind in ('weapon', 'armor', 'accessory') and rarity in ('supreme', 'ultimate') then 'legendary'
    else rarity
  end
where kind in ('weapon', 'armor', 'accessory');

insert into world_zones (id, name) values
  ('woma', '沃玛寺庙'),
  ('centipede', '蜈蚣洞'),
  ('zuma', '祖玛寺庙'),
  ('redmoon', '赤月峡谷'),
  ('molong', '魔龙岭')
on conflict (id) do update set name = excluded.name;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('bq_plains', 'plains', '比奇平原', '新手最熟悉的练级地带，通向城区、森林、废矿、毒蛇山谷和沃玛路线。推荐 1-10 级探索。', '{"比奇城区":"bq_town:gate","新手村":"newbie_village","森林":"forest","矿洞":"mine_entrance","毒蛇山谷":"snake_valley:valley_gate","沃玛寺庙":"woma:entrance"}', '["chicken","deer","scarecrow","hook_cat","orc_scout"]', false),
  ('bq_plains', 'forest', '比奇森林', '林地里钉耙猫、半兽人和森林雪人不断游荡。推荐 6-14 级探索。', '{"比奇平原":"plains","沃玛寺庙":"woma:entrance"}', '["hook_cat","forest_wolf","orc_scout","forest_yeti"]', false),
  ('bq_plains', 'mine_entrance', '废矿入口', '矿车倾倒在湿冷的洞口，僵尸和洞蛆的气息从深处涌出。推荐 10-18 级探索。', '{"比奇平原":"plains","矿区一层":"mine_depth"}', '["cave_bat","mine_zombie","zombie_crawler"]', false),
  ('bq_plains', 'mine_depth', '矿区一层', '黑暗矿道纵横交错，尸王残影守着更深的石门。推荐 16-24 级探索。', '{"废矿入口":"mine_entrance","尸王殿":"mine_palace"}', '["mine_zombie","zombie_soldier","corpse_warrior"]', false),
  ('bq_plains', 'mine_palace', '尸王殿', '尸王殿深处刷新尸王，适合完成早期装备和技能书积累。推荐 22-28 级探索。', '{"矿区一层":"mine_depth"}', '["zombie_soldier","corpse_warrior","corpse_king"]', false),

  ('woma', 'entrance', '沃玛森林', '沃玛寺庙外缘的密林，半兽勇士和沃玛守卫在这里游荡。推荐 18-24 级探索。', '{"比奇森林":"bq_plains:forest","沃玛寺庙一层":"first"}', '["orc_warrior","woma_guard","woma_bat"]', false),
  ('woma', 'first', '沃玛寺庙一层', '石柱间火光忽明忽暗，沃玛战士开始成群出现。推荐 24-30 级探索。', '{"沃玛森林":"entrance","沃玛寺庙二层":"second"}', '["woma_bat","woma_warrior","woma_flame"]', false),
  ('woma', 'second', '沃玛寺庙二层', '通往祭坛的长廊，沃玛长老和火焰沃玛守住入口。推荐 30-36 级探索。', '{"沃玛寺庙一层":"first","沃玛祭坛":"altar"}', '["woma_warrior","woma_flame","woma_elder"]', false),
  ('woma', 'altar', '沃玛祭坛', '沃玛教主会在祭坛深处现身，是早期第一道 Boss 门槛。推荐 36-42 级探索。', '{"沃玛寺庙二层":"second","盟重土城":"mengzhong:town"}', '["woma_elder","woma_lord"]', false),

  ('mengzhong', 'desert_gate', '盟重荒漠', '黄沙遮天，通往石墓、蜈蚣洞、祖玛寺庙和苍月航线。推荐 30-36 级探索。', '{"毒蛇山谷":"snake_valley:valley_depth","盟重土城":"town","石墓阵":"stone_tomb","蜈蚣洞":"centipede:entrance","祖玛寺庙":"zuma:entrance","世界首领":"boss_field"}', '["desert_wolf","armored_beetle","desert_bandit"]', false),
  ('centipede', 'entrance', '死亡山谷', '蜈蚣洞入口潮湿阴冷，钳虫和蜈蚣成群出没。推荐 34-42 级探索。', '{"盟重荒漠":"mengzhong:desert_gate","地牢一层东":"east"}', '["centipede_bug","evil_centipede","dark_warrior"]', false),
  ('centipede', 'east', '地牢一层东', '阴暗地牢深处会出现跳跳蜂和巨型蠕虫。推荐 42-50 级探索。', '{"死亡山谷":"entrance","黑暗地带":"dark"}', '["evil_centipede","jumping_bee","giant_worm"]', false),
  ('centipede', 'dark', '黑暗地带', '触龙神的巢穴外层，普通怪已经要求二到三阶装备。推荐 50-58 级探索。', '{"地牢一层东":"east","触龙神巢穴":"dragon_nest"}', '["giant_worm","dark_warrior","evil_centipede"]', false),
  ('centipede', 'dragon_nest', '触龙神巢穴', '触龙神潜伏在岩层深处，适合检验强化 +5 左右的装备强度。推荐 58-62 级探索。', '{"黑暗地带":"dark","封魔谷":"fengmo:camp"}', '["giant_worm","evil_centipede","touch_dragon"]', false),

  ('zuma', 'entrance', '祖玛寺庙入口', '祖玛雕像在昏暗殿堂中苏醒。推荐 42-50 级探索。', '{"盟重荒漠":"mengzhong:desert_gate","祖玛寺庙三层":"third"}', '["zuma_statue","zuma_archer","zuma_guard"]', false),
  ('zuma', 'third', '祖玛寺庙三层', '祖玛卫士巡逻密集，开始要求稳定补给和技能等级。推荐 50-58 级探索。', '{"祖玛寺庙入口":"entrance","祖玛阁":"maze"}', '["zuma_guard","zuma_archer","zuma_statue"]', false),
  ('zuma', 'maze', '祖玛阁', '祖玛阁道路错乱，精英守卫强度接近强化 +5 装备门槛。推荐 58-66 级探索。', '{"祖玛寺庙三层":"third","祖玛教主之家":"lord_room"}', '["zuma_guard","zuma_statue","zuma_elite"]', false),
  ('zuma', 'lord_room', '祖玛教主之家', '祖玛教主之家是中期装备、技能和界限突破的第一次综合考验。推荐 66-72 级探索。', '{"祖玛阁":"maze","封魔谷":"fengmo:camp"}', '["zuma_elite","zuma_lord"]', false),

  ('redmoon', 'bairimen_gate', '白日门', '白日门是赤月路线前的高阶安全据点，适合补给和整理仓库。', '{"比奇平原":"bq_plains:plains","山谷密道":"secret_path"}', '[]', true),
  ('redmoon', 'secret_path', '山谷密道', '石壁潮湿狭窄，月魔蜘蛛沿着洞顶游走。推荐 72-82 级探索。', '{"白日门":"bairimen_gate","抉择之地":"choice_land"}', '["moon_spider","steel_spider","blood_giant"]', false),
  ('redmoon', 'choice_land', '抉择之地', '岔路尽头有赤月气息渗出，血巨人和钢牙蜘蛛守住道路。推荐 82-92 级探索。', '{"山谷密道":"secret_path","恶魔祭坛":"demon_altar"}', '["steel_spider","blood_giant","redmoon_priest"]', false),
  ('redmoon', 'demon_altar', '恶魔祭坛', '祭坛上残火不灭，双头金刚和赤月祭司巡守周围。推荐 92-100 级探索。', '{"抉择之地":"choice_land","赤月峡谷":"canyon"}', '["redmoon_priest","twin_head_guard"]', false),
  ('redmoon', 'canyon', '赤月峡谷', '深谷中赤月恶魔的威压直逼神魂，中期以后必须搭配装备和界限突破。推荐 100-110 级探索。', '{"恶魔祭坛":"demon_altar","魔龙岭":"molong:outer"}', '["redmoon_priest","twin_head_guard","redmoon_demon"]', false),

  ('molong', 'outer', '魔龙岭外谷', '魔龙城外的荒岭，魔龙守卫和魔龙刀兵在谷口集结。推荐 110-120 级探索。', '{"赤月峡谷":"redmoon:canyon","魔龙城":"city"}', '["molong_guard","molong_blade"]', false),
  ('molong', 'city', '魔龙城', '魔龙远征军建立的高阶安全区，后期整备与行会集结点。', '{"魔龙岭外谷":"outer","魔龙东郊":"east"}', '[]', true),
  ('molong', 'east', '魔龙东郊', '城外荒土遍布魔龙枪兵和魔龙破甲兵。推荐 120-132 级探索。', '{"魔龙城":"city","魔龙旧寨":"village"}', '["molong_lancer","molong_breaker"]', false),
  ('molong', 'village', '魔龙旧寨', '旧寨残垣里还残留魔龙军阵，魔龙统领在此集结残部。推荐 132-142 级探索。', '{"魔龙东郊":"east","魔龙沼泽":"swamp"}', '["molong_hunter","molong_general"]', false),
  ('molong', 'swamp', '魔龙沼泽', '黑水沼泽会吞没脚步，魔龙战将守着血域入口。推荐 142-150 级探索。', '{"魔龙旧寨":"village","魔龙血域":"bloodland"}', '["molong_general","molong_warrior"]', false),
  ('molong', 'bloodland', '魔龙血域', '血域尽头魔龙血魔和魔龙教主盘踞，后期 Boss 按强化 +8 和界限 +5 预期设计。推荐 150 级后探索。', '{"魔龙沼泽":"swamp"}', '["molong_warrior","molong_blood_demon","molong_lord"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set exits = exits || '{"白日门":"redmoon:bairimen_gate"}'::jsonb
where zone_id = 'bq_plains' and id = 'plains';

update world_rooms
set exits = exits || '{"魔龙岭":"molong:outer"}'::jsonb
where zone_id = 'cangyue' and id = 'molong_city';

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('chicken', '鸡', 1, 28, 3, 0, 5, 4, false, 45),
  ('deer', '鹿', 2, 38, 4, 1, 8, 6, false, 60),
  ('sheep', '羊', 3, 52, 5, 1, 10, 7, false, 60),
  ('scarecrow', '稻草人', 3, 62, 6, 1, 12, 8, false, 45),
  ('hook_cat', '钉耙猫', 5, 92, 10, 3, 22, 14, false, 80),
  ('orc_scout', '半兽人', 6, 118, 13, 4, 30, 18, false, 90),
  ('forest_wolf', '森林雪人', 8, 160, 18, 6, 42, 24, false, 100),
  ('forest_yeti', '森林雪人王', 12, 360, 32, 14, 120, 70, false, 160),
  ('poison_bee', '毒蜘蛛', 10, 205, 24, 8, 60, 35, false, 120),
  ('cave_bat', '洞蛆', 12, 260, 28, 10, 82, 46, false, 120),
  ('mine_zombie', '僵尸', 16, 410, 42, 18, 130, 70, false, 150),
  ('zombie_crawler', '爬行僵尸', 18, 520, 50, 22, 170, 85, false, 160),
  ('zombie_soldier', '电僵尸', 22, 720, 68, 30, 260, 120, false, 180),
  ('corpse_warrior', '尸王护卫', 24, 880, 76, 36, 330, 150, false, 190),
  ('corpse_king', '尸王', 28, 5200, 145, 78, 1800, 650, true, 900),
  ('snake', '毒蛇', 15, 340, 36, 14, 115, 62, false, 120),
  ('red_snake', '红蛇', 18, 510, 48, 20, 150, 76, false, 140),
  ('valley_bandit', '山谷流寇', 22, 760, 66, 30, 240, 115, false, 150),
  ('serpent_guard', '虎蛇', 26, 980, 82, 38, 340, 150, false, 170),
  ('serpent_king', '虎蛇王', 30, 6200, 160, 88, 2200, 760, true, 900),
  ('orc_warrior', '半兽勇士', 22, 820, 74, 34, 280, 135, false, 160),
  ('woma_bat', '暗黑战士', 24, 900, 82, 40, 330, 150, false, 170),
  ('woma_guard', '沃玛卫士', 28, 1280, 105, 56, 500, 210, false, 180),
  ('woma_warrior', '沃玛战士', 32, 1650, 130, 68, 680, 280, false, 210),
  ('woma_flame', '火焰沃玛', 34, 1900, 148, 72, 780, 320, false, 220),
  ('woma_elder', '沃玛长老', 36, 2350, 170, 88, 980, 380, false, 240),
  ('woma_lord', '沃玛教主', 42, 18000, 330, 185, 6800, 1800, true, 1500),
  ('desert_wolf', '沙漠土狼', 32, 1500, 130, 62, 520, 220, false, 180),
  ('armored_beetle', '盔甲虫', 34, 1750, 145, 78, 620, 250, false, 190),
  ('desert_bandit', '盟重流寇', 36, 2100, 168, 86, 760, 300, false, 210),
  ('red_boar', '红野猪', 38, 2600, 190, 100, 900, 360, false, 210),
  ('black_boar', '黑野猪', 40, 3100, 215, 118, 1100, 430, false, 220),
  ('white_boar', '白野猪', 46, 26000, 440, 250, 9000, 2600, true, 1500),
  ('centipede_bug', '钳虫', 38, 2800, 205, 112, 960, 380, false, 220),
  ('evil_centipede', '蜈蚣', 42, 3600, 245, 138, 1350, 520, false, 250),
  ('jumping_bee', '跳跳蜂', 46, 4300, 285, 150, 1700, 640, false, 260),
  ('giant_worm', '巨型蠕虫', 52, 6200, 360, 210, 2600, 920, false, 300),
  ('dark_warrior', '黑色恶蛆', 56, 7200, 420, 240, 3400, 1100, false, 320),
  ('touch_dragon', '触龙神', 62, 52000, 680, 430, 22000, 5200, true, 1800),
  ('zuma_guard', '祖玛卫士', 52, 6800, 390, 235, 3100, 1000, false, 300),
  ('zuma_archer', '祖玛弓箭手', 54, 6000, 430, 210, 3300, 1080, false, 300),
  ('zuma_statue', '祖玛雕像', 56, 7600, 455, 270, 3900, 1250, false, 320),
  ('zuma_elite', '极品祖玛卫士', 64, 13500, 610, 390, 7600, 2200, false, 420),
  ('zuma_lord', '祖玛教主', 72, 105000, 920, 620, 52000, 9800, true, 2400),
  ('cangyue_warrior', '苍月妖兵', 44, 3900, 260, 130, 1600, 620, false, 250),
  ('sea_demon', '海魔妖', 46, 4300, 290, 142, 1900, 720, false, 260),
  ('bone_soldier', '骷髅战将', 48, 5000, 325, 168, 2300, 840, false, 280),
  ('bone_general', '骷髅统领', 54, 7600, 430, 245, 4200, 1350, false, 320),
  ('nether_lord', '黄泉教主', 64, 72000, 780, 500, 34000, 7600, true, 2100),
  ('bull_guard', '牛魔护卫', 58, 8800, 480, 280, 5200, 1600, false, 340),
  ('bull_warrior', '牛魔战士', 62, 11000, 560, 330, 6600, 1900, false, 360),
  ('bull_king', '牛魔王', 72, 98000, 930, 650, 56000, 10500, true, 2400),
  ('fengmo_bat', '虹魔蝙蝠', 64, 11800, 560, 330, 6200, 1700, false, 300),
  ('fengmo_guard', '封魔卫兵', 66, 13200, 610, 370, 7600, 2000, false, 330),
  ('rainbow_spirit', '虹魔妖灵', 68, 15000, 680, 420, 9200, 2300, false, 350),
  ('rainbow_warrior', '虹魔战士', 70, 17200, 740, 470, 11000, 2600, false, 370),
  ('rainbow_boar', '虹魔猪卫', 72, 20500, 820, 520, 13500, 3100, false, 390),
  ('rainbow_guard', '虹魔护法', 74, 24200, 900, 590, 16000, 3600, false, 420),
  ('rainbow_priest', '虹魔祭司', 76, 28200, 990, 660, 19000, 4200, false, 450),
  ('rainbow_lord', '虹魔教主', 82, 125000, 1250, 850, 68000, 12500, true, 2100),
  ('moon_spider', '月魔蜘蛛', 82, 31000, 1050, 720, 22000, 4300, false, 420),
  ('steel_spider', '钢牙蜘蛛', 88, 39000, 1180, 820, 28000, 5200, false, 450),
  ('blood_giant', '血巨人', 92, 48000, 1320, 930, 34000, 6200, false, 480),
  ('redmoon_priest', '赤月祭司', 98, 62000, 1520, 1080, 44000, 7600, false, 520),
  ('twin_head_guard', '双头金刚', 106, 210000, 2050, 1450, 98000, 17000, true, 1800),
  ('redmoon_demon', '赤月恶魔', 112, 360000, 2450, 1720, 165000, 28000, true, 2400),
  ('molong_guard', '魔龙守卫', 112, 76000, 1800, 1180, 52000, 9000, false, 480),
  ('molong_blade', '魔龙刀兵', 118, 94000, 2050, 1340, 62000, 10500, false, 520),
  ('molong_lancer', '魔龙枪兵', 124, 116000, 2280, 1500, 76000, 12400, false, 560),
  ('molong_breaker', '魔龙破甲兵', 132, 150000, 2600, 1720, 98000, 15600, false, 600),
  ('molong_hunter', '魔龙射手', 138, 180000, 2900, 1900, 120000, 18500, false, 640),
  ('molong_general', '魔龙统领', 144, 225000, 3250, 2180, 150000, 23000, false, 680),
  ('molong_warrior', '魔龙战将', 150, 285000, 3650, 2450, 190000, 29000, false, 720),
  ('molong_blood_demon', '魔龙血魔', 156, 620000, 4700, 3150, 300000, 46000, true, 2400),
  ('molong_lord', '魔龙教主', 168, 980000, 5400, 3650, 460000, 70000, true, 3000)
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

update mob_templates
set
  exp = greatest(1, (exp * 85 / 100)),
  gold = greatest(0, (gold * 90 / 100))
where level >= 35 and boss = false;

update mob_templates
set
  exp = greatest(1, (exp * 75 / 100)),
  gold = greatest(0, (gold * 85 / 100))
where level >= 50 and boss = true;
