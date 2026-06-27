insert into world_zones (id, name) values
  ('bq_town', '比奇城区'),
  ('bq_plains', '比奇地区'),
  ('snake_valley', '毒蛇山谷'),
  ('mengzhong', '盟重区域'),
  ('cangyue', '苍月岛')
on conflict (id) do update set name = excluded.name;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('bq_town', 'gate', '比奇城区', '城墙内灯火通明，药店、仓库和集市都在这里。城外就是比奇平原。', '{"比奇平原":"bq_plains:plains","药店集市":"market","无尽塔":"tower"}', '[]', true),
  ('bq_town', 'market', '药店集市', '药师和铁匠守着摊位，新人常在这里补给后出城。', '{"比奇城区":"gate"}', '[]', true),
  ('bq_town', 'tower', '无尽塔入口', '石塔高耸入云，只有身在比奇地区的勇士可以发起无尽塔挑战。', '{"比奇城区":"gate","比奇平原":"bq_plains:plains"}', '[]', true),

  ('bq_plains', 'plains', '比奇平原', '新手最熟悉的练级地带，通向城区、新手村、森林、矿洞和毒蛇山谷。推荐 0-10 级探索。', '{"比奇城区":"bq_town:gate","新手村":"newbie_village","森林":"forest","矿洞":"mine_entrance","毒蛇山谷":"snake_valley:valley_gate"}', '["chicken","deer","scarecrow","hook_cat"]', false),
  ('bq_plains', 'newbie_village', '新手村', '木栅栏外鸡鹿成群，半兽人斥候偶尔摸到村口。推荐 0-6 级探索。', '{"比奇平原":"plains","森林":"forest"}', '["chicken","deer","scarecrow"]', false),
  ('bq_plains', 'forest', '比奇森林', '林地里钉耙猫、森林狼和毒蜂不断游荡。推荐 4-10 级探索。', '{"比奇平原":"plains","新手村":"newbie_village","沃玛森林":"woma_woods"}', '["hook_cat","forest_wolf","poison_bee"]', false),
  ('bq_plains', 'mine_entrance', '废矿入口', '矿车倾倒在湿冷的洞口，尸王和矿洞僵尸的气息从深处涌出。推荐 8-16 级探索。', '{"比奇平原":"plains","矿洞深处":"mine_depth"}', '["cave_bat","mine_zombie","corpse_warrior"]', false),
  ('bq_plains', 'mine_depth', '矿洞深处', '黑暗矿道纵横交错，尸王残影守着更深的石门。推荐 12-18 级探索。', '{"矿洞入口":"mine_entrance"}', '["mine_zombie","corpse_warrior","corpse_king"]', false),
  ('bq_plains', 'woma_woods', '沃玛森林', '古老神庙的外缘，沃玛卫士在林间巡逻。推荐 16-24 级探索。', '{"比奇森林":"forest","沃玛神庙":"woma_temple"}', '["woma_guard","woma_warrior"]', false),
  ('bq_plains', 'woma_temple', '沃玛神庙', '神庙里回荡着低沉鼓声，沃玛教主的影子偶尔掠过祭坛。推荐 20-28 级探索。', '{"沃玛森林":"woma_woods"}', '["woma_warrior","woma_elder","woma_lord"]', false),

  ('snake_valley', 'valley_gate', '毒蛇山谷入口', '山路狭窄潮湿，毒蛇和红蛇潜伏在石缝间。推荐 10-15 级探索。', '{"比奇平原":"bq_plains:plains","山谷深处":"valley_depth"}', '["snake","red_snake","valley_bandit"]', false),
  ('snake_valley', 'valley_depth', '毒蛇山谷深处', '蛇群盘踞在破碎祭坛旁，蛇王常在月色下现身。推荐 15-20 级探索。', '{"山谷入口":"valley_gate","盟重区域":"mengzhong:desert_gate"}', '["red_snake","valley_bandit","serpent_guard"]', false),

  ('mengzhong', 'desert_gate', '盟重荒漠', '黄沙遮天，沙漠土狼、盔甲虫和流寇在商道附近出没。推荐 20-25 级探索。', '{"毒蛇山谷":"snake_valley:valley_depth","盟重土城":"town","石墓":"stone_tomb","祖玛神庙":"zuma_temple","世界首领":"boss_field"}', '["desert_wolf","armored_beetle","desert_bandit"]', false),
  ('mengzhong', 'town', '盟重土城', '土城是远征者集结地，通往石墓、祖玛神庙和苍月岛航线。', '{"盟重荒漠":"desert_gate","苍月岛":"cangyue:island"}', '[]', true),
  ('mengzhong', 'stone_tomb', '石墓阵', '石墓阵炎热逼仄，红野猪和黑野猪在石道里横冲直撞。推荐 25-32 级探索。', '{"盟重荒漠":"desert_gate"}', '["red_boar","black_boar","white_boar"]', false),
  ('mengzhong', 'zuma_temple', '祖玛神庙', '祖玛雕像和卫士在昏暗殿堂中苏醒。推荐 30-38 级探索。', '{"盟重荒漠":"desert_gate"}', '["zuma_guard","zuma_archer","zuma_statue"]', false),
  ('mengzhong', 'boss_field', '盟重祭坛', '荒漠中心的远古祭坛，世界首领会在这里接受挑战。', '{"盟重荒漠":"desert_gate","盟重土城":"town"}', '["zuma_lord"]', false),

  ('cangyue', 'island', '苍月岛海岸', '海风里混着妖气，只有来到苍月岛后才可探索秘境。推荐 32-38 级探索。', '{"盟重土城":"mengzhong:town","骨魔洞":"bone_cave","牛魔寺庙":"bull_temple"}', '["cangyue_warrior","sea_demon","bone_soldier"]', false),
  ('cangyue', 'bone_cave', '骨魔洞', '白骨铺满洞窟，骨魔将和黄泉教主的气息盘旋不散。推荐 38-45 级探索。', '{"苍月岛":"island"}', '["bone_soldier","bone_general","nether_lord"]', false),
  ('cangyue', 'bull_temple', '牛魔寺庙', '牛魔族守着古老秘境入口，妖火照亮石阶。推荐 42-50 级探索。', '{"苍月岛":"island"}', '["bull_guard","bull_warrior","bull_king"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('scarecrow', '稻草人', 2, 38, 4, 0, 14, 8, false, 45),
  ('hook_cat', '钉耙猫', 4, 75, 9, 2, 32, 20, false, 80),
  ('forest_wolf', '森林狼', 6, 110, 13, 4, 48, 30, false, 90),
  ('poison_bee', '毒蜂', 7, 92, 16, 3, 55, 34, false, 80),
  ('cave_bat', '洞穴蝙蝠', 8, 128, 18, 4, 70, 42, false, 90),
  ('mine_zombie', '矿洞僵尸', 10, 190, 23, 7, 100, 60, false, 120),
  ('corpse_warrior', '尸王护卫', 14, 310, 35, 12, 180, 95, false, 150),
  ('corpse_king', '尸王', 18, 980, 55, 18, 520, 260, true, 600),
  ('snake', '毒蛇', 11, 210, 25, 8, 115, 68, false, 100),
  ('red_snake', '红蛇', 14, 300, 34, 10, 165, 88, false, 120),
  ('valley_bandit', '山谷流寇', 16, 380, 42, 14, 220, 120, false, 130),
  ('serpent_guard', '蛇王卫士', 19, 520, 55, 18, 310, 160, false, 150),
  ('serpent_king', '山谷蛇王', 22, 1500, 82, 28, 900, 420, true, 900),
  ('woma_guard', '沃玛卫士', 18, 620, 58, 22, 360, 180, false, 150),
  ('woma_warrior', '沃玛战将', 21, 780, 70, 28, 470, 230, false, 180),
  ('woma_elder', '沃玛长老', 24, 980, 86, 34, 620, 300, false, 210),
  ('woma_lord', '沃玛教主', 28, 2600, 128, 52, 1800, 850, true, 1200),
  ('desert_wolf', '沙漠土狼', 21, 720, 70, 24, 430, 210, false, 140),
  ('armored_beetle', '盔甲虫', 23, 840, 78, 32, 520, 250, false, 160),
  ('desert_bandit', '盟重流寇', 25, 980, 90, 34, 650, 320, false, 180),
  ('red_boar', '红野猪', 27, 1120, 102, 38, 760, 370, false, 180),
  ('black_boar', '黑野猪', 29, 1280, 115, 44, 900, 430, false, 190),
  ('white_boar', '白野猪', 32, 3600, 165, 70, 2600, 1200, true, 1200),
  ('zuma_guard', '祖玛卫士', 32, 1700, 145, 58, 1250, 600, false, 220),
  ('zuma_archer', '祖玛弓箭手', 34, 1500, 170, 50, 1350, 660, false, 220),
  ('zuma_statue', '祖玛雕像', 36, 2200, 185, 76, 1650, 760, false, 240),
  ('zuma_lord', '祖玛教主', 38, 18000, 280, 110, 18000, 8000, true, 1800),
  ('cangyue_warrior', '苍月妖兵', 34, 1900, 160, 62, 1500, 720, false, 220),
  ('sea_demon', '海魔妖', 36, 2100, 178, 68, 1750, 800, false, 240),
  ('bone_soldier', '骨魔士兵', 38, 2400, 198, 76, 2100, 920, false, 260),
  ('bone_general', '骨魔将', 42, 3600, 245, 96, 3200, 1350, false, 300),
  ('nether_lord', '黄泉教主', 45, 22000, 340, 140, 24000, 10000, true, 1800),
  ('bull_guard', '牛魔护卫', 42, 3800, 250, 105, 3400, 1450, false, 300),
  ('bull_warrior', '牛魔战士', 45, 4600, 285, 120, 4300, 1700, false, 320),
  ('bull_king', '牛魔王', 50, 30000, 420, 180, 36000, 15000, true, 2400)
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

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats) values
  ('helm_bronze_t1', '青铜头盔', 'armor', 'head', 'uncommon', 520, false, '{"def":5,"hp":45,"tier":1,"score":14}'),
  ('boots_deerskin_t1', '鹿皮靴', 'armor', 'feet', 'uncommon', 480, false, '{"def":3,"dex":3,"tier":1,"score":12}'),
  ('neck_jade_t2', '青玉项链', 'accessory', 'neck', 'rare', 1600, false, '{"mag":8,"hp":80,"tier":2,"score":40}'),
  ('bracelet_tiger_t2', '虎齿手镯', 'accessory', 'bracelet_left', 'rare', 1500, false, '{"atk":10,"dex":5,"tier":2,"score":42}'),
  ('blade_serpent_t2', '蛇纹剑', 'weapon', 'weapon', 'rare', 1900, false, '{"atk":22,"dex":4,"tier":2,"score":54}'),
  ('armor_woma_t3', '沃玛战甲', 'armor', 'chest', 'epic', 4300, false, '{"def":42,"hp":420,"mdef":10,"tier":3,"score":118}'),
  ('helm_woma_t3', '沃玛头盔', 'armor', 'head', 'epic', 3900, false, '{"def":30,"hp":260,"tier":3,"score":90}'),
  ('neck_woma_t3', '沃玛项链', 'accessory', 'neck', 'epic', 4200, false, '{"atk":18,"mag":18,"tier":3,"score":102}'),
  ('blade_purgatory_t4', '炼狱战刃', 'weapon', 'weapon', 'legendary', 11800, false, '{"atk":92,"dex":12,"tier":4,"score":260}'),
  ('armor_dragon_t4', '龙鳞战甲', 'armor', 'chest', 'legendary', 11600, false, '{"def":92,"mdef":24,"hp":980,"tier":4,"score":278}'),
  ('ring_zuma_t4', '祖玛神戒', 'accessory', 'ring_right', 'legendary', 9800, false, '{"atk":48,"mag":34,"hp":520,"tier":4,"score":250}'),
  ('blade_cangyue_t5', '苍月破军刃', 'weapon', 'weapon', 'supreme', 32000, false, '{"atk":150,"dex":28,"spirit":12,"tier":5,"score":430}'),
  ('armor_bull_t5', '牛魔王战甲', 'armor', 'chest', 'supreme', 34000, false, '{"def":150,"mdef":58,"hp":1800,"tier":5,"score":470}'),
  ('neck_nether_t5', '黄泉镇魂链', 'accessory', 'neck', 'supreme', 30000, false, '{"atk":72,"mag":72,"hp":900,"tier":5,"score":420}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats;
