create table if not exists guild_task_claims (
  guild_id bigint not null references guilds(id) on delete cascade,
  character_id bigint not null references characters(id) on delete cascade,
  task_kind text not null check (task_kind in ('patrol', 'supply', 'boss_drill')),
  period_key text not null default current_date::text,
  contribution bigint not null default 0,
  funds_added bigint not null default 0,
  completed_at timestamptz not null default now(),
  primary key (guild_id, character_id, task_kind, period_key)
);

create index if not exists guild_task_claims_period_idx
  on guild_task_claims (guild_id, period_key, task_kind);

update guilds
set
  notice = case
    when name = '比奇守卫军' then '稳定型行会：适合新手做每日巡猎、补给建设和材料积累。'
    when name = '沙城远征队' then '进攻型行会：适合中后期玩家参与首领演武、沙巴克和高阶地图推进。'
    else notice
  end,
  funds = case
    when name = '比奇守卫军' then greatest(funds, 18000)
    when name = '沙城远征队' then greatest(funds, 62000)
    else funds
  end,
  level = case
    when name = '比奇守卫军' then greatest(level, 2)
    when name = '沙城远征队' then greatest(level, 4)
    else level
  end;

insert into quest_templates
  (id, category, name, description, min_level, sort_order, objectives, rewards)
values
  (
    'main_reach_level_20',
    'main',
    '山谷试炼',
    '角色等级达到 20 级，完成毒蛇山谷到沃玛森林的过渡。',
    10,
    120,
    '{"kind":"level","required":20}',
    '{"gold":520,"items":[{"template_id":"potion_large","quantity":2,"bind":true},{"template_id":"treasure_shard","quantity":1,"bind":true}]}'
  ),
  (
    'main_reach_level_35',
    'main',
    '盟重立足',
    '角色等级达到 35 级，开始依赖装备、技能和界限突破共同成长。',
    20,
    130,
    '{"kind":"level","required":35}',
    '{"gold":1200,"items":[{"template_id":"pet_food","quantity":2,"bind":true},{"template_id":"treasure_shard","quantity":2,"bind":true},{"template_id":"cultivation_pill","quantity":1,"bind":true}]}'
  ),
  (
    'main_reach_level_50',
    'main',
    '苍月远征',
    '角色等级达到 50 级，准备从苍月岛衔接封魔谷。',
    35,
    140,
    '{"kind":"level","required":50}',
    '{"gold":2600,"items":[{"template_id":"pet_food_pack","quantity":1,"bind":true},{"template_id":"treasure_shard_pack","quantity":1,"bind":true},{"template_id":"cultivation_pill","quantity":2,"bind":true}]}'
  ),
  (
    'main_reach_level_70',
    'main',
    '封魔破局',
    '角色等级达到 70 级，封魔谷毕业后可补强技能、宠物、法宝和修炼再进白日门。',
    50,
    150,
    '{"kind":"level","required":70}',
    '{"gold":4200,"items":[{"template_id":"pet_food_pack","quantity":1,"bind":true},{"template_id":"treasure_shard_pack","quantity":1,"bind":true},{"template_id":"cultivation_pill_pack","quantity":1,"bind":true}]}'
  ),
  (
    'main_reach_level_100',
    'main',
    '赤月尽头',
    '角色等级达到 100 级，进入魔龙城前需要高阶装备与系统成长配合。',
    70,
    160,
    '{"kind":"level","required":100}',
    '{"gold":7200,"items":[{"template_id":"guardian_gem","quantity":1,"bind":true},{"template_id":"battle_gem","quantity":1,"bind":true},{"template_id":"cultivation_pill_pack","quantity":1,"bind":true}]}'
  ),
  (
    'daily_guild_task',
    'daily',
    '每日行会事务',
    '今日完成任意 1 个行会任务，获得稳定的界限突破材料。',
    1,
    325,
    '{"kind":"guild_task","required":1}',
    '{"gold":180,"items":[{"template_id":"pet_food","quantity":1,"bind":true},{"template_id":"treasure_shard","quantity":1,"bind":true},{"template_id":"cultivation_pill","quantity":1,"bind":true}]}'
  ),
  (
    'daily_guild_hunt',
    'daily',
    '行会巡猎',
    '今日击败 35 只怪物，为行会任务和个人成长积累补给。',
    12,
    335,
    '{"kind":"kill_any","required":35}',
    '{"gold":260,"items":[{"template_id":"potion_large","quantity":1,"bind":true},{"template_id":"guild_merit_token","quantity":1,"bind":true}]}'
  )
on conflict (id) do update set
  category = excluded.category,
  name = excluded.name,
  description = excluded.description,
  min_level = excluded.min_level,
  sort_order = excluded.sort_order,
  objectives = excluded.objectives,
  rewards = excluded.rewards,
  enabled = true;

update quest_templates
set
  description = '今日累计击败 20 只怪物，提供基础药品和少量宠物材料。',
  rewards = '{"gold":160,"items":[{"template_id":"potion_small","quantity":2,"bind":true},{"template_id":"potion_mana","quantity":2,"bind":true},{"template_id":"pet_food","quantity":1,"bind":true}]}'
where id = 'daily_hunt_20';

update quest_templates
set
  description = '今日完成 1 次行会捐献或行会补给任务，稳定获得少量法宝材料。',
  rewards = '{"gold":100,"items":[{"template_id":"treasure_shard","quantity":1,"bind":true},{"template_id":"guild_merit_token","quantity":1,"bind":true}]}'
where id = 'daily_guild_donate';

update quest_templates
set
  description = '今日击败 30 只怪物，为宠物、法宝和修炼各补 1 份绑定材料。',
  rewards = '{"gold":220,"items":[{"template_id":"pet_food","quantity":1,"bind":true},{"template_id":"treasure_shard","quantity":1,"bind":true},{"template_id":"cultivation_pill","quantity":1,"bind":true}]}'
where id = 'daily_growth_supply';

update quest_templates
set rewards = '{"gold":220,"items":[{"template_id":"potion_small","quantity":3,"bind":true},{"template_id":"potion_mana","quantity":2,"bind":true}]}'
where id = 'main_reach_level_5';

update quest_templates
set rewards = '{"gold":360,"items":[{"template_id":"potion_large","quantity":1,"bind":true},{"template_id":"pet_food","quantity":1,"bind":true}]}'
where id = 'main_reach_level_10';

insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds) values
  ('chicken', '鸡', 1, 22, 2, 0, 8, 5, false, 45),
  ('deer', '鹿', 2, 30, 3, 1, 12, 8, false, 60),
  ('sheep', '羊', 3, 42, 4, 1, 18, 12, false, 60),
  ('scarecrow', '稻草人', 2, 34, 4, 0, 14, 8, false, 45),
  ('hook_cat', '钉耙猫', 4, 66, 8, 2, 32, 20, false, 80),
  ('forest_wolf', '森林狼', 6, 102, 12, 4, 48, 30, false, 90),
  ('poison_bee', '毒蜂', 7, 86, 14, 3, 55, 34, false, 80),
  ('cave_bat', '洞穴蝙蝠', 8, 118, 16, 5, 70, 42, false, 90),
  ('mine_zombie', '矿洞僵尸', 10, 172, 21, 8, 100, 60, false, 120),
  ('corpse_warrior', '尸王护卫', 14, 300, 32, 13, 180, 95, false, 150),
  ('corpse_king', '尸王', 18, 1150, 52, 21, 520, 260, true, 600),
  ('snake', '毒蛇', 11, 198, 23, 9, 115, 68, false, 100),
  ('red_snake', '红蛇', 14, 295, 31, 12, 165, 88, false, 120),
  ('valley_bandit', '山谷流寇', 16, 380, 39, 16, 220, 120, false, 130),
  ('serpent_guard', '蛇王卫士', 19, 520, 50, 21, 310, 160, false, 150),
  ('serpent_king', '山谷蛇王', 22, 1650, 76, 32, 900, 420, true, 900),
  ('woma_guard', '沃玛卫士', 18, 620, 54, 25, 360, 180, false, 150),
  ('woma_warrior', '沃玛战将', 21, 790, 66, 31, 470, 230, false, 180),
  ('woma_elder', '沃玛长老', 24, 1000, 80, 38, 620, 300, false, 210),
  ('woma_lord', '沃玛教主', 28, 3000, 118, 58, 1800, 850, true, 1200),
  ('desert_wolf', '沙漠土狼', 21, 720, 66, 27, 430, 210, false, 140),
  ('armored_beetle', '盔甲虫', 23, 850, 74, 36, 520, 250, false, 160),
  ('desert_bandit', '盟重流寇', 25, 980, 86, 38, 650, 320, false, 180),
  ('red_boar', '红野猪', 27, 1120, 96, 44, 760, 370, false, 180),
  ('black_boar', '黑野猪', 29, 1300, 106, 50, 900, 430, false, 190),
  ('white_boar', '白野猪', 32, 3900, 152, 78, 2600, 1200, true, 1200),
  ('zuma_guard', '祖玛卫士', 32, 1760, 134, 66, 1250, 600, false, 220),
  ('zuma_archer', '祖玛弓箭手', 34, 1580, 152, 58, 1350, 660, false, 220),
  ('zuma_statue', '祖玛雕像', 36, 2300, 168, 82, 1650, 760, false, 240),
  ('zuma_lord', '祖玛教主', 38, 21000, 255, 122, 18000, 8000, true, 1800),
  ('cangyue_warrior', '苍月妖兵', 34, 1950, 145, 70, 1500, 720, false, 220),
  ('sea_demon', '海魔妖', 36, 2180, 160, 76, 1750, 800, false, 240),
  ('bone_soldier', '骨魔士兵', 38, 2520, 180, 86, 2100, 920, false, 260),
  ('bone_general', '骨魔将', 42, 3900, 222, 108, 3200, 1350, false, 300),
  ('nether_lord', '黄泉教主', 45, 26000, 310, 152, 24000, 10000, true, 1800),
  ('bull_guard', '牛魔护卫', 42, 4100, 226, 114, 3400, 1450, false, 300),
  ('bull_warrior', '牛魔战士', 45, 4900, 258, 128, 4300, 1700, false, 320),
  ('bull_king', '牛魔王', 50, 36000, 372, 190, 36000, 15000, true, 2400),
  ('fengmo_bat', '虹魔蝙蝠', 52, 5200, 300, 150, 5600, 1850, false, 260),
  ('fengmo_guard', '封魔卫兵', 55, 6200, 340, 178, 6800, 2200, false, 300),
  ('rainbow_spirit', '虹魔妖灵', 58, 7400, 390, 205, 8200, 2550, false, 320),
  ('rainbow_warrior', '虹魔战士', 61, 8800, 450, 238, 10800, 3000, false, 340),
  ('rainbow_boar', '虹魔猪卫', 64, 10400, 510, 278, 13500, 3400, false, 360),
  ('rainbow_guard', '虹魔护法', 67, 12400, 585, 325, 17000, 4200, false, 390),
  ('rainbow_priest', '虹魔祭司', 70, 14800, 675, 380, 22000, 5200, false, 420),
  ('rainbow_lord', '虹魔教主', 72, 76000, 900, 560, 82000, 16000, true, 1500),
  ('moon_spider', '月魔蜘蛛', 70, 14000, 640, 350, 18000, 3600, false, 300),
  ('blood_giant', '血巨人', 78, 18500, 760, 440, 26000, 5000, false, 360),
  ('redmoon_priest', '赤月祭司', 86, 24500, 910, 540, 36000, 7000, false, 420),
  ('twin_head_guard', '双头金刚', 94, 105000, 1160, 680, 90000, 16000, true, 1200),
  ('redmoon_demon', '赤月恶魔', 100, 190000, 1380, 820, 180000, 32000, true, 1800),
  ('molong_guard', '魔龙守卫', 100, 30000, 1080, 620, 52000, 9500, false, 420),
  ('molong_lancer', '魔龙枪兵', 110, 38000, 1300, 740, 70000, 12500, false, 480),
  ('molong_hunter', '魔龙猎手', 120, 48000, 1530, 860, 92000, 16000, false, 540),
  ('molong_general', '魔龙统领', 132, 62000, 1800, 1010, 130000, 22000, false, 600),
  ('molong_warrior', '魔龙战将', 142, 78000, 2050, 1160, 170000, 28000, false, 660),
  ('molong_blood_demon', '魔龙血魔', 150, 230000, 2780, 1500, 240000, 42000, true, 1800),
  ('molong_lord', '魔龙教主', 150, 360000, 3000, 1650, 360000, 65000, true, 2400)
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

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('blade_soul_t6', '镇魂天刃', 'weapon', 'weapon', 'supreme', 46000, false, '{"atk":190,"mag":40,"luck":30,"crit_pct":3,"heavy_hit_pct":3,"tier":6,"score":535}', '{}'),
  ('armor_soul_t6', '镇魂战甲', 'armor', 'chest', 'supreme', 48000, false, '{"def":198,"mdef":72,"hp":2250,"tier":6,"score":595}', '{}'),
  ('ring_soul_t6', '镇魂戒', 'accessory', 'ring_left', 'supreme', 43000, false, '{"atk":84,"mag":68,"hp":920,"luck":32,"crit_pct":3,"tier":6,"score":480}', '{}'),
  ('neck_soul_t6', '镇魂项链', 'accessory', 'neck', 'supreme', 43000, false, '{"atk":78,"mag":86,"luck":32,"life_steal_pct":1,"tier":6,"score":480}', '{}'),
  ('blade_starfall_t7', '星陨斩龙刃', 'weapon', 'weapon', 'ultimate', 68000, false, '{"atk":250,"mag":54,"luck":42,"crit_pct":4,"heavy_hit_pct":4,"tier":7,"score":730}', '{}'),
  ('armor_starfall_t7', '星陨龙甲', 'armor', 'chest', 'ultimate', 70000, false, '{"def":260,"mdef":98,"hp":3100,"tier":7,"score":805}', '{}'),
  ('ring_starfall_t7', '星陨戒', 'accessory', 'ring_right', 'ultimate', 62000, false, '{"atk":110,"mag":90,"hp":1200,"luck":44,"crit_pct":4,"tier":7,"score":650}', '{}'),
  ('neck_starfall_t7', '星陨链', 'accessory', 'neck', 'ultimate', 62000, false, '{"atk":102,"mag":116,"luck":44,"life_steal_pct":2,"tier":7,"score":660}', '{}'),
  ('blade_void_t8', '虚空裂魂刃', 'weapon', 'weapon', 'ultimate', 92000, false, '{"atk":325,"mag":72,"luck":58,"crit_pct":5,"heavy_hit_pct":5,"life_steal_pct":2,"tier":8,"score":940}', '{}'),
  ('armor_void_t8', '虚空玄甲', 'armor', 'chest', 'ultimate', 96000, false, '{"def":340,"mdef":132,"hp":4200,"paralyze_pct":1,"tier":8,"score":1030}', '{}'),
  ('ring_void_t8', '虚空神戒', 'accessory', 'ring_left', 'ultimate', 86000, false, '{"atk":146,"mag":124,"hp":1600,"luck":62,"crit_pct":5,"tier":8,"score":835}', '{}'),
  ('neck_void_t8', '虚空项链', 'accessory', 'neck', 'ultimate', 86000, false, '{"atk":138,"mag":154,"luck":62,"life_steal_pct":3,"tier":8,"score":850}', '{}'),
  ('blade_god_t9', '神阶开天刃', 'weapon', 'weapon', 'ultimate', 130000, false, '{"atk":430,"mag":96,"luck":82,"crit_pct":6,"heavy_hit_pct":7,"life_steal_pct":3,"paralyze_pct":1,"tier":9,"score":1260}', '{}'),
  ('armor_god_t9', '神阶不灭甲', 'armor', 'chest', 'ultimate', 136000, false, '{"def":450,"mdef":178,"hp":5800,"paralyze_pct":1,"tier":9,"score":1380}', '{}'),
  ('ring_god_t9', '神阶天命戒', 'accessory', 'ring_right', 'ultimate', 120000, false, '{"atk":196,"mag":166,"hp":2200,"luck":86,"crit_pct":6,"life_steal_pct":2,"tier":9,"score":1120}', '{}'),
  ('neck_god_t9', '神阶镇界链', 'accessory', 'neck', 'ultimate', 120000, false, '{"atk":188,"mag":210,"luck":86,"life_steal_pct":4,"heavy_hit_pct":3,"tier":9,"score":1140}', '{}'),
  ('blade_immortal_t10', '仙阶太初剑', 'weapon', 'weapon', 'ultimate', 190000, false, '{"atk":590,"mag":140,"luck":112,"crit_pct":8,"heavy_hit_pct":10,"life_steal_pct":4,"paralyze_pct":2,"tier":10,"score":1730}', '{}'),
  ('armor_immortal_t10', '仙阶万劫甲', 'armor', 'chest', 'ultimate', 200000, false, '{"def":620,"mdef":245,"hp":8200,"paralyze_pct":2,"tier":10,"score":1900}', '{}'),
  ('ring_immortal_t10', '仙阶长生戒', 'accessory', 'ring_left', 'ultimate', 180000, false, '{"atk":270,"mag":230,"hp":3150,"luck":118,"crit_pct":8,"life_steal_pct":4,"tier":10,"score":1560}', '{}'),
  ('neck_immortal_t10', '仙阶归墟链', 'accessory', 'neck', 'ultimate', 180000, false, '{"atk":260,"mag":295,"luck":118,"life_steal_pct":5,"heavy_hit_pct":5,"tier":10,"score":1600}', '{}')
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = item_templates.flags || excluded.flags;

update skills
set config = config || '{"desc":"被动增加攻击、防御，前期稳定提升近战强度；后期成长主要依赖装备和界限突破。","atk_bonus":3,"def_bonus":1}'::jsonb
where id = 'basic_sword';

update skills
set config = config || '{"desc":"被动增加魔法攻击和魔法上限，保证法师前期伤害但不替代装备。","mag_bonus":4,"mp_bonus":14}'::jsonb
where id = 'fireball';

update skills
set config = config || '{"desc":"被动增加生命、魔法防御和少量魔法攻击，提升道士前期容错。","hp_bonus":28,"mdef_bonus":2,"mag_bonus":1}'::jsonb
where id = 'healing_charm';

update skills
set config = config || '{"desc":"被动增加攻击、幸运和暴击，刺客前期爆发更高但仍需要补防御。","atk_bonus":3,"luck_bonus":2,"crit_pct_bonus":1}'::jsonb
where id = 'shadow_step';
