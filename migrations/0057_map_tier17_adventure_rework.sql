-- 0.2.9: level 1-500 topology, Tier 1-17 equipment curve, and adventure-ready world.

alter table if exists vip_potion_settings
  drop constraint if exists vip_potion_settings_auto_decompose_max_tier_check;

alter table if exists vip_potion_settings
  add constraint vip_potion_settings_auto_decompose_max_tier_check
  check (auto_decompose_max_tier between 0 and 17);

delete from world_rooms;
delete from world_zones;

insert into world_zones (id, name, config) values
  ('fanchen', '第一界域：凡尘界', '{"realm_index":1,"safe_hub":"qingniu_town"}'::jsonb),
  ('xiuzhen', '第二界域：修真界', '{"realm_index":2,"safe_hub":"tianshui_city"}'::jsonb),
  ('feisheng', '第三界域：飞升界', '{"realm_index":3,"safe_hub":"void_fortress"}'::jsonb),
  ('ancient_secret', '第四界域：上古秘境', '{"realm_index":4,"safe_hub":null,"danger":"no_safe_zone"}'::jsonb)
on conflict (id) do update set
  name = excluded.name,
  config = excluded.config;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('fanchen', 'qingniu_town', '青牛镇', '第一界域主城安全区。镇外通往迷雾竹林。', '{"竹林外围":"bamboo_outer"}', '[]', true),
  ('fanchen', 'bamboo_outer', 'Lv.1-3 竹林外围', '迷雾竹林外缘，魔化藤蔓缠绕旧路。', '{"青牛镇":"qingniu_town","竹林岔道":"bamboo_fork"}', '["mob_demon_vine"]', false),
  ('fanchen', 'bamboo_fork', 'Lv.4-6 竹林岔道', '铁甲竹鼠在岔道间钻行，前方开始分出主干与首领支线。', '{"竹林外围":"bamboo_outer","密林河道":"forest_river","竹林深处":"bamboo_depths"}', '["mob_iron_bamboo_rat"]', false),
  ('fanchen', 'forest_river', 'Lv.7-9 密林河道', '河道边赤目妖猪横冲直撞，是前期挂机主干道。', '{"竹林岔道":"bamboo_fork","矿洞入口":"mine_entrance"}', '["mob_red_eye_boar"]', false),
  ('fanchen', 'bamboo_depths', 'Lv.10 竹林深处', '死胡同首领点。狂暴猪王守在竹影最深处，击败后需原路折返。', '{"竹林岔道":"bamboo_fork"}', '["boss_raging_boar_king"]', false),
  ('fanchen', 'mine_entrance', 'Lv.11-15 矿洞入口', '废弃灵矿入口，噬矿鼠啃食灵石残渣。', '{"密林河道":"forest_river","废弃矿道":"abandoned_mine"}', '["mob_ore_rat"]', false),
  ('fanchen', 'abandoned_mine', 'Lv.16-19 废弃矿道', '游荡尸傀拖着矿镐徘徊，矿道在此分叉。', '{"矿洞入口":"mine_entrance","塌陷暗道":"collapsed_passage","矿长室":"foreman_room"}', '["mob_wandering_corpse"]', false),
  ('fanchen', 'collapsed_passage', '塌陷暗道', '无怪的探索过渡节点，裂缝外透出荒野的风。', '{"废弃矿道":"abandoned_mine","荒野边缘":"wilderness_edge"}', '[]', false),
  ('fanchen', 'foreman_room', 'Lv.20 矿长室', '尸傀监工盘踞在矿长室，守着灵矿残余。', '{"废弃矿道":"abandoned_mine"}', '["boss_corpse_foreman"]', false),
  ('fanchen', 'wilderness_edge', 'Lv.21-28 荒野边缘', '苍茫荒野入口，荒原野狼结群游荡。', '{"塌陷暗道":"collapsed_passage","兽骨高地":"bone_highland"}', '["mob_wild_wolf"]', false),
  ('fanchen', 'bone_highland', 'Lv.29-35 兽骨高地', '兽骨堆成高地，血翼骨雕在空中盘旋。', '{"荒野边缘":"wilderness_edge","荒野深处":"wilderness_depths"}', '["mob_bloodwing_vulture"]', false),
  ('fanchen', 'wilderness_depths', 'Lv.36-39 荒野深处', '荒野狼王带着狼群守住古道遗迹前的最后一段路。', '{"兽骨高地":"bone_highland","古道遗迹":"ancient_road_ruins","荒野祭坛":"wilderness_altar"}', '["mob_wilderness_wolf_king"]', false),
  ('fanchen', 'ancient_road_ruins', '古道遗迹', '界域通道，残碑指向下一主城天水古城。', '{"荒野深处":"wilderness_depths","天水古城":"xiuzhen:tianshui_city"}', '[]', false),
  ('fanchen', 'wilderness_altar', 'Lv.40 荒野祭坛', '镇界石魔镇守凡尘界出口，是第一界域 BOSS。', '{"荒野深处":"wilderness_depths"}', '["boss_boundary_stonemaw"]', false),

  ('xiuzhen', 'tianshui_city', '天水古城', '第二界域主城安全区。城外毒气沼泽通往万妖谷。', '{"青牛镇":"fanchen:qingniu_town","毒气沼泽":"poison_marsh"}', '[]', true),
  ('xiuzhen', 'poison_marsh', 'Lv.41-55 毒气沼泽', '毒沼巨鳄潜伏于腐水之下。', '{"天水古城":"tianshui_city","百花秘境":"flower_secret"}', '["mob_poison_gator"]', false),
  ('xiuzhen', 'flower_secret', 'Lv.56-68 百花秘境', '幻影花妖在花瘴中显形又消失。', '{"毒气沼泽":"poison_marsh","猿啼裂谷":"ape_ravine"}', '["mob_phantom_flower"]', false),
  ('xiuzhen', 'ape_ravine', 'Lv.69-79 猿啼裂谷', '六臂魔猿的吼声在裂谷中回荡。', '{"百花秘境":"flower_secret","幽暗小径":"dark_path","万妖洞天":"wanyao_cave"}', '["mob_six_arm_ape"]', false),
  ('xiuzhen', 'dark_path', '幽暗小径', '万妖谷主干过渡道，幽光通往镇魔残塔。', '{"猿啼裂谷":"ape_ravine","残塔一层":"tower_floor_1"}', '[]', false),
  ('xiuzhen', 'wanyao_cave', 'Lv.80 万妖洞天', '万妖谷主坐镇洞天，是万妖谷关底首领。', '{"猿啼裂谷":"ape_ravine"}', '["boss_wanyao_lord"]', false),
  ('xiuzhen', 'tower_floor_1', 'Lv.81-95 残塔一层', '缚灵咒徒在残塔底层诵念断裂咒文。', '{"幽暗小径":"dark_path","残塔二层":"tower_floor_2"}', '["mob_bound_cultist"]', false),
  ('xiuzhen', 'tower_floor_2', 'Lv.96-110 残塔二层', '无头镇墓将拖着重甲巡行。', '{"残塔一层":"tower_floor_1","残塔三层":"tower_floor_3"}', '["mob_headless_warden"]', false),
  ('xiuzhen', 'tower_floor_3', 'Lv.111-119 残塔三层', '铁血典狱长守着通往塔底暗流的铁闸。', '{"残塔二层":"tower_floor_2","塔底暗流":"tower_undercurrent","塔顶封印":"tower_top_seal"}', '["mob_iron_jailer"]', false),
  ('xiuzhen', 'tower_undercurrent', '塔底暗流', '残塔主干过渡道，暗流尽头寒气涌动。', '{"残塔三层":"tower_floor_3","冰封雪林":"ice_forest"}', '[]', false),
  ('xiuzhen', 'tower_top_seal', 'Lv.120 塔顶封印', '怨魂聚合体撞击封印，是镇魔残塔关底首领。', '{"残塔三层":"tower_floor_3"}', '["boss_wraith_aggregate"]', false),
  ('xiuzhen', 'ice_forest', 'Lv.121-135 冰封雪林', '风雪游女隐入白林，寒意侵骨。', '{"塔底暗流":"tower_undercurrent","凛冬峡谷":"winter_canyon"}', '["mob_snow_wanderer"]', false),
  ('xiuzhen', 'winter_canyon', 'Lv.136-150 凛冬峡谷', '冰甲角魔踏碎峡谷冰桥。', '{"冰封雪林":"ice_forest","龙骨冰原":"dragonbone_icefield"}', '["mob_ice_horn_demon"]', false),
  ('xiuzhen', 'dragonbone_icefield', 'Lv.151-159 龙骨冰原', '极寒冰龙盘踞龙骨之间，冰原在此分出航线与王座。', '{"凛冬峡谷":"winter_canyon","破冰航线":"icebreaking_route","冰原王座":"ice_throne"}', '["mob_frost_dragon"]', false),
  ('xiuzhen', 'icebreaking_route', '破冰航线', '界域通道，破冰船驶向虚空要塞。', '{"龙骨冰原":"dragonbone_icefield","虚空要塞":"feisheng:void_fortress"}', '[]', false),
  ('xiuzhen', 'ice_throne', 'Lv.160 冰原王座', '冰原主宰端坐寒霜王座，是第二界域 BOSS。', '{"龙骨冰原":"dragonbone_icefield"}', '["boss_icefield_overlord"]', false),

  ('feisheng', 'void_fortress', '虚空要塞', '第三界域主城安全区。要塞外即忘川河畔。', '{"天水古城":"xiuzhen:tianshui_city","忘川河畔":"wangchuan_bank"}', '[]', true),
  ('feisheng', 'wangchuan_bank', 'Lv.161-175 忘川河畔', '忘川雾气漫过河岸，幽魂在水面低语。', '{"虚空要塞":"void_fortress","迷魂渡口":"lost_ferry"}', '["mob_wangchuan_wraith"]', false),
  ('feisheng', 'lost_ferry', 'Lv.176-185 迷魂渡口', '迷魂渡口船灯摇曳，渡灵鬼卒拦路。', '{"忘川河畔":"wangchuan_bank","幽冥水府":"nether_water_palace"}', '["mob_lost_ferryman"]', false),
  ('feisheng', 'nether_water_palace', 'Lv.186-199 幽冥水府', '幽冥水府深处传来判官殿钟声。', '{"迷魂渡口":"lost_ferry","妖气裂隙":"demon_rift","判官殿":"judge_hall"}', '["mob_nether_aquaguard"]', false),
  ('feisheng', 'demon_rift', '妖气裂隙', '主干过渡裂隙，妖气把空间撕向狐月神殿。', '{"幽冥水府":"nether_water_palace","幻月迷阵":"moon_maze"}', '[]', false),
  ('feisheng', 'judge_hall', 'Lv.200 判官殿', '阎罗判官翻阅生死簿，是九幽黄泉关底首领。', '{"幽冥水府":"nether_water_palace"}', '["boss_yanluo_judge"]', false),
  ('feisheng', 'moon_maze', 'Lv.201-215 幻月迷阵', '幻月迷阵中月光折返，银狐幻影环伺。', '{"妖气裂隙":"demon_rift","银狐广场":"silver_fox_square"}', '["mob_moon_illusion"]', false),
  ('feisheng', 'silver_fox_square', 'Lv.216-230 银狐广场', '银狐广场空旷明亮，天狐侍卫严阵以待。', '{"幻月迷阵":"moon_maze","拜月神台":"moon_worship_platform"}', '["mob_silver_fox_guard"]', false),
  ('feisheng', 'moon_worship_platform', 'Lv.231-239 拜月神台', '拜月神台连接空间裂痕与天狐内殿。', '{"银狐广场":"silver_fox_square","空间裂痕":"space_rift","天狐内殿":"skyfox_inner_palace"}', '["mob_moon_priestess"]', false),
  ('feisheng', 'space_rift', '空间裂痕', '主干过渡裂痕，深渊气息从另一端渗出。', '{"拜月神台":"moon_worship_platform","深渊浅层":"abyss_shallow"}', '[]', false),
  ('feisheng', 'skyfox_inner_palace', 'Lv.240 天狐内殿', '天狐老祖坐镇内殿，是狐月神殿关底首领。', '{"拜月神台":"moon_worship_platform"}', '["boss_skyfox_ancestor"]', false),
  ('feisheng', 'abyss_shallow', 'Lv.241-260 深渊浅层', '混沌深渊浅层扭曲重力，深渊游魂不断凝形。', '{"空间裂痕":"space_rift","深渊魔窟":"abyss_cave"}', '["mob_abyss_wanderer"]', false),
  ('feisheng', 'abyss_cave', 'Lv.261-280 深渊魔窟', '深渊魔窟里魔气翻滚，魔窟守卫吞噬灵光。', '{"深渊浅层":"abyss_shallow","混沌风暴":"chaos_storm"}', '["mob_abyss_guard"]', false),
  ('feisheng', 'chaos_storm', 'Lv.281-299 混沌风暴', '混沌风暴撕裂视线，通往登天雷池与渊兽巢穴。', '{"深渊魔窟":"abyss_cave","登天雷池":"ascension_thunder_pool","渊兽巢穴":"abyss_beast_lair"}', '["mob_chaos_stormfiend"]', false),
  ('feisheng', 'ascension_thunder_pool', '登天雷池', '飞升通道。雷池尽头是上古秘境太初矿区外围。', '{"混沌风暴":"chaos_storm","太初矿区外围":"ancient_secret:mining_outer"}', '[]', false),
  ('feisheng', 'abyss_beast_lair', 'Lv.300 渊兽巢穴', '混沌渊兽王盘踞巢穴，是第三界域 BOSS。', '{"混沌风暴":"chaos_storm"}', '["boss_chaos_abyss_beast"]', false),

  ('ancient_secret', 'mining_outer', 'Lv.301-320 太初矿区外围', '上古秘境无安全区，太初古矿外围源石裸露。', '{"登天雷池":"feisheng:ascension_thunder_pool","源石晶洞":"source_crystal_cave"}', '["mob_source_miner"]', false),
  ('ancient_secret', 'source_crystal_cave', 'Lv.321-339 源石晶洞', '源石晶洞折射古老灵光，晶洞深处有王座支线。', '{"太初矿区外围":"mining_outer","干涸大泽":"dry_marsh","源石王座":"source_throne"}', '["mob_source_crystal_beast"]', false),
  ('ancient_secret', 'dry_marsh', '干涸大泽', '过渡节点。干裂河床通往洪荒毒林。', '{"源石晶洞":"source_crystal_cave","洪荒毒林":"miasma_forest"}', '[]', false),
  ('ancient_secret', 'source_throne', 'Lv.340 源石王座', '源石傀儡王自晶簇中站起。', '{"源石晶洞":"source_crystal_cave"}', '["boss_source_golem_king"]', false),
  ('ancient_secret', 'miasma_forest', 'Lv.341-360 瘴气毒林', '洪荒大泽边缘，瘴气毒林里毒藤翻卷。', '{"干涸大泽":"dry_marsh","吞天蛇沼":"devouring_snake_marsh"}', '["mob_miasma_treant"]', false),
  ('ancient_secret', 'devouring_snake_marsh', 'Lv.361-379 吞天蛇沼', '吞天蛇沼深不见底，古蛇盘踞泥潭。', '{"瘴气毒林":"miasma_forest","远古栈道":"ancient_plank_path","巨猿领地":"giant_ape_domain"}', '["mob_devouring_serpent"]', false),
  ('ancient_secret', 'ancient_plank_path', '远古栈道', '过渡节点。腐朽栈道通向造化仙门。', '{"吞天蛇沼":"devouring_snake_marsh","破败山门":"ruined_gate"}', '[]', false),
  ('ancient_secret', 'giant_ape_domain', 'Lv.380 巨猿领地', '洪荒巨猿撼动大泽古木。', '{"吞天蛇沼":"devouring_snake_marsh"}', '["boss_primordial_giant_ape"]', false),
  ('ancient_secret', 'ruined_gate', 'Lv.381-400 破败山门', '造化仙门山门破败，残阵仍在运转。', '{"远古栈道":"ancient_plank_path","仙门天阶":"immortal_steps"}', '["mob_ruined_gatekeeper"]', false),
  ('ancient_secret', 'immortal_steps', 'Lv.401-419 仙门天阶', '仙门天阶直入云端，天阶尽头分向残殿与火山口。', '{"破败山门":"ruined_gate","陨落火山口":"fallen_volcano","凌霄残殿":"lingxiao_ruins"}', '["mob_heavenstep_sentinel"]', false),
  ('ancient_secret', 'fallen_volcano', '陨落火山口', '过渡节点。焦黑火山口外是涅槃火域灰烬平原。', '{"仙门天阶":"immortal_steps","灰烬平原":"ash_plain"}', '[]', false),
  ('ancient_secret', 'lingxiao_ruins', 'Lv.420 凌霄残殿', '堕落谪仙守着凌霄残殿。', '{"仙门天阶":"immortal_steps"}', '["boss_fallen_exiled_immortal"]', false),
  ('ancient_secret', 'ash_plain', 'Lv.421-440 灰烬平原', '灰烬平原热浪翻涌，火域异兽在灰中复燃。', '{"陨落火山口":"fallen_volcano","浴火神巢":"fire_nest"}', '["mob_ash_firebeast"]', false),
  ('ancient_secret', 'fire_nest', 'Lv.441-459 浴火神巢', '浴火神巢火羽纷落，神凰分身的气息压迫四周。', '{"灰烬平原":"ash_plain","星空古路":"star_road","涅槃核心":"nirvana_core"}', '["mob_phoenix_nest_guard"]', false),
  ('ancient_secret', 'star_road', '星空古路', '过渡节点。星光古路延伸至鸿蒙星海。', '{"浴火神巢":"fire_nest","碎星带":"broken_stars"}', '[]', false),
  ('ancient_secret', 'nirvana_core', 'Lv.460 涅槃核心', '不死神凰分身在涅槃核心中燃烧。', '{"浴火神巢":"fire_nest"}', '["boss_phoenix_avatar"]', false),
  ('ancient_secret', 'broken_stars', 'Lv.461-480 碎星带', '鸿蒙星海外环，碎星带中星屑如刃。', '{"星空古路":"star_road","巨兽陨石":"beast_meteor"}', '["mob_star_shardling"]', false),
  ('ancient_secret', 'beast_meteor', 'Lv.481-498 巨兽陨石', '巨兽陨石漂浮在星海深处，陨兽仍有余威。', '{"碎星带":"broken_stars","天道祭坛":"heavenly_dao_altar","终极虚空":"final_void"}', '["mob_meteor_behemoth"]', false),
  ('ancient_secret', 'heavenly_dao_altar', 'Lv.499 天道祭坛', '鸿蒙天道幻影镇守终极虚空前的门槛。', '{"巨兽陨石":"beast_meteor"}', '["boss_heavenly_dao_phantom"]', false),
  ('ancient_secret', 'final_void', '终极虚空', '终极世界 BOSS 区域。万古渊魔只掉落主宰套装，刷新时间由系统设定。', '{"巨兽陨石":"beast_meteor"}', '["world_boss_eternal_abyss_demon"]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

with mob_defs(id, name, level, boss, respawn_seconds) as (
  values
    ('mob_demon_vine', '魔化藤蔓', 3, false, 45),
    ('mob_iron_bamboo_rat', '铁甲竹鼠', 6, false, 60),
    ('mob_red_eye_boar', '赤目妖猪', 9, false, 75),
    ('boss_raging_boar_king', '狂暴猪王', 10, true, 900),
    ('mob_ore_rat', '噬矿鼠', 15, false, 100),
    ('mob_wandering_corpse', '游荡尸傀', 19, false, 120),
    ('boss_corpse_foreman', '尸傀监工', 20, true, 1200),
    ('mob_wild_wolf', '荒原野狼', 28, false, 140),
    ('mob_bloodwing_vulture', '血翼骨雕', 35, false, 160),
    ('mob_wilderness_wolf_king', '荒野狼王', 39, false, 180),
    ('boss_boundary_stonemaw', '镇界石魔', 40, true, 1800),
    ('mob_poison_gator', '毒沼巨鳄', 55, false, 220),
    ('mob_phantom_flower', '幻影花妖', 68, false, 260),
    ('mob_six_arm_ape', '六臂魔猿', 79, false, 300),
    ('boss_wanyao_lord', '万妖谷主', 80, true, 2400),
    ('mob_bound_cultist', '缚灵咒徒', 95, false, 340),
    ('mob_headless_warden', '无头镇墓将', 110, false, 380),
    ('mob_iron_jailer', '铁血典狱长', 119, false, 420),
    ('boss_wraith_aggregate', '怨魂聚合体', 120, true, 3000),
    ('mob_snow_wanderer', '风雪游女', 135, false, 460),
    ('mob_ice_horn_demon', '冰甲角魔', 150, false, 500),
    ('mob_frost_dragon', '极寒冰龙', 159, false, 540),
    ('boss_icefield_overlord', '冰原主宰', 160, true, 3600),
    ('mob_wangchuan_wraith', '忘川幽魂', 175, false, 580),
    ('mob_lost_ferryman', '迷魂渡者', 185, false, 620),
    ('mob_nether_aquaguard', '幽冥水府守卫', 199, false, 660),
    ('boss_yanluo_judge', '阎罗判官', 200, true, 4200),
    ('mob_moon_illusion', '幻月狐影', 215, false, 700),
    ('mob_silver_fox_guard', '银狐卫', 230, false, 740),
    ('mob_moon_priestess', '拜月祭女', 239, false, 780),
    ('boss_skyfox_ancestor', '天狐老祖', 240, true, 4800),
    ('mob_abyss_wanderer', '深渊游魂', 260, false, 820),
    ('mob_abyss_guard', '深渊魔卫', 280, false, 860),
    ('mob_chaos_stormfiend', '混沌风暴魔', 299, false, 900),
    ('boss_chaos_abyss_beast', '混沌渊兽王', 300, true, 5400),
    ('mob_source_miner', '太初矿灵', 320, false, 940),
    ('mob_source_crystal_beast', '源石晶兽', 339, false, 980),
    ('boss_source_golem_king', '源石傀儡王', 340, true, 6000),
    ('mob_miasma_treant', '瘴气毒树', 360, false, 1020),
    ('mob_devouring_serpent', '吞天古蛇', 379, false, 1060),
    ('boss_primordial_giant_ape', '洪荒巨猿', 380, true, 6600),
    ('mob_ruined_gatekeeper', '破败门灵', 400, false, 1100),
    ('mob_heavenstep_sentinel', '仙门天卫', 419, false, 1140),
    ('boss_fallen_exiled_immortal', '堕落谪仙', 420, true, 7200),
    ('mob_ash_firebeast', '灰烬火兽', 440, false, 1180),
    ('mob_phoenix_nest_guard', '浴火巢卫', 459, false, 1220),
    ('boss_phoenix_avatar', '不死神凰分身', 460, true, 7800),
    ('mob_star_shardling', '碎星灵', 480, false, 1260),
    ('mob_meteor_behemoth', '巨兽陨灵', 498, false, 1300),
    ('boss_heavenly_dao_phantom', '鸿蒙天道幻影', 499, true, 8400),
    ('world_boss_eternal_abyss_demon', '万古渊魔', 600, true, 14400)
),
scaled as (
  select
    id,
    name,
    level,
    boss,
    respawn_seconds,
    round(200 * power(level::double precision, 2.4))::bigint as normal_hp,
    round(20 * power(level::double precision, 2.4))::bigint as normal_atk,
    round(10 * power(level::double precision, 2.3))::bigint as normal_def
  from mob_defs
)
insert into mob_templates (id, name, level, max_hp, atk, def, exp, gold, boss, respawn_seconds, drops) 
select
  id,
  name,
  level,
  case
    when id = 'world_boss_eternal_abyss_demon' then normal_hp * 100
    when boss then normal_hp * 20
    else normal_hp
  end as max_hp,
  case
    when id = 'world_boss_eternal_abyss_demon' then normal_atk * 4
    when boss then normal_atk * 2
    else normal_atk
  end as atk,
  normal_def as def,
  greatest(10, round(level::double precision * level::double precision * 35)::bigint) as exp,
  greatest(5, round(level::double precision * level::double precision * 9)::bigint) as gold,
  boss,
  respawn_seconds,
  jsonb_build_object('drop_model', 'global_dynamic_tier_1_17', 'max_drop_tier', least(17, greatest(1, ceil(level::double precision / 30.0)::int)))
from scaled
on conflict (id) do update set
  name = excluded.name,
  level = excluded.level,
  max_hp = excluded.max_hp,
  atk = excluded.atk,
  def = excluded.def,
  exp = excluded.exp,
  gold = excluded.gold,
  boss = excluded.boss,
  respawn_seconds = excluded.respawn_seconds,
  drops = excluded.drops;

with tier_defs(tier, series, set_id, rarity) as (
  values
    (1, '凡尘系列', null, 'common'),
    (2, '玄铁系列', null, 'uncommon'),
    (3, '青云套装', 'qingyun', 'rare'),
    (4, '地煞系列', null, 'rare'),
    (5, '天罡系列', null, 'epic'),
    (6, '纯阳套装', 'pureyang', 'epic'),
    (7, '星陨系列', null, 'epic'),
    (8, '幽冥系列', null, 'legendary'),
    (9, '九霄套装', 'jiuxiao', 'legendary'),
    (10, '神渊系列', null, 'legendary'),
    (11, '天狐系列', null, 'legendary'),
    (12, '混沌套装', 'chaos', 'mythic'),
    (13, '太初系列', null, 'mythic'),
    (14, '洪荒系列', null, 'mythic'),
    (15, '造化套装', 'zaohua', 'supreme'),
    (16, '涅槃系列', null, 'supreme'),
    (17, '鸿蒙系列', null, 'ultimate')
),
slot_defs(slot_id, kind, slot, label, slot_class) as (
  values
    ('weapon', 'weapon', 'weapon', '战刃', 'weapon'),
    ('chest', 'armor', 'chest', '战甲', 'armor'),
    ('head', 'armor', 'head', '头盔', 'armor'),
    ('waist', 'armor', 'waist', '腰带', 'armor'),
    ('feet', 'armor', 'feet', '战靴', 'armor'),
    ('neck', 'accessory', 'neck', '项链', 'accessory'),
    ('bracelet', 'accessory', 'bracelet_left', '手镯', 'accessory'),
    ('ring', 'accessory', 'ring_left', '戒指', 'accessory')
),
generated as (
  select
    format('t%s_%s', lpad(tier::text, 2, '0'), slot_id) as id,
    case
      when tier = 1 and slot_id = 'weapon' then '破旧铁剑'
      when tier = 1 and slot_id = 'chest' then '粗布麻衣'
      when tier = 2 and slot_id = 'weapon' then '精炼长刃'
      when tier = 2 and slot_id = 'chest' then '玄铁重铠'
      when tier = 16 and slot_id = 'chest' then '涅槃火甲'
      else replace(series, '系列', '') || label
    end as name,
    kind,
    slot,
    rarity,
    (tier::bigint * tier::bigint * 10000 + case slot_class when 'weapon' then 9000 when 'accessory' then 7000 else 6000 end)::bigint as price,
    jsonb_strip_nulls(jsonb_build_object(
      'tier', tier,
      'series', series,
      'score', (power(tier::double precision, 3.0) * case slot_class when 'weapon' then 130 else 100 end)::bigint,
      'affix_count', case when tier >= 6 then 3 + least(5, tier / 3) else 1 + tier / 3 end,
      'atk', case when slot_class in ('weapon', 'accessory') then round(50 * power(tier::double precision, 2.8))::bigint end,
      'mag', case when slot_class in ('weapon', 'accessory') then round(50 * power(tier::double precision, 2.8))::bigint end,
      'hp', case when slot_class = 'armor' then round(250 * power(tier::double precision, 2.8))::bigint end,
      'def', case when slot_class = 'armor' then round(20 * power(tier::double precision, 2.5))::bigint end,
      'mdef', case when slot_class = 'armor' then round(20 * power(tier::double precision, 2.5))::bigint end,
      'life_steal_pct', case when tier = 4 then 1 end,
      'crit_pct', case when tier = 5 then 2 end,
      'ignore_def_pct', case when tier = 8 then least(floor((tier - 5) * 1.5), 100)::bigint end,
      'damage_deepen_pct', case when tier = 10 then 5 end,
      'guaranteed_hit_pct', case when tier = 11 then least(floor((tier - 5) * 1.5), 100)::bigint end,
      'boss_damage_pct', case when tier = 14 then 10 end,
      'origin_revive_cd_seconds', case when tier = 16 and slot_id = 'chest' then 3600 end,
      'max_percent_affix_cap', case when tier >= 6 then floor((tier - 5) * 1.5)::bigint end,
      'set', set_id
    )) as stats,
    jsonb_strip_nulls(jsonb_build_object(
      'set', set_id,
      'drop_tier', tier,
      'slot_class', slot_class,
      'equipment_standard', 'tier_1_17_dynamic_2026'
    )) as flags
  from tier_defs
  cross join slot_defs
)
insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags)
select id, name, kind, slot, rarity, price, false, stats, flags
from generated
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;

with dominator_slots(slot_id, kind, slot, name, slot_class) as (
  values
    ('weapon', 'weapon', 'weapon', '主宰神刃', 'weapon'),
    ('chest', 'armor', 'chest', '主宰神甲', 'armor'),
    ('head', 'armor', 'head', '主宰神盔', 'armor'),
    ('neck', 'accessory', 'neck', '主宰项链', 'accessory'),
    ('bracelet', 'accessory', 'bracelet_left', '主宰手镯', 'accessory'),
    ('ring', 'accessory', 'ring_left', '主宰戒指', 'accessory'),
    ('waist', 'armor', 'waist', '主宰神带', 'armor'),
    ('feet', 'armor', 'feet', '主宰神靴', 'armor')
),
generated as (
  select
    'dominator_' || case slot_id when 'weapon' then 'blade' when 'chest' then 'armor' when 'head' then 'helm' when 'bracelet' then 'bracelet' when 'ring' then 'ring' when 'waist' then 'belt' when 'feet' then 'boots' else slot_id end as id,
    name,
    kind,
    slot,
    'ultimate' as rarity,
    50000000::bigint as price,
    jsonb_strip_nulls(jsonb_build_object(
      'tier', 17,
      'series', '主宰套装',
      'score', case slot_class when 'weapon' then 300000 else 220000 end,
      'atk', case when slot_class in ('weapon', 'accessory') then 250000 end,
      'mag', case when slot_class in ('weapon', 'accessory') then 250000 end,
      'hp', case when slot_class = 'armor' then 1200000 end,
      'def', case when slot_class = 'armor' then 120000 end,
      'mdef', case when slot_class = 'armor' then 120000 end,
      'crit_pct', case when slot_class in ('weapon', 'accessory') then 18 end,
      'crit_damage_pct', case when slot_class in ('weapon', 'accessory') then 50 end,
      'life_steal_pct', case when slot_class in ('weapon', 'accessory') then 5 end,
      'mana_steal_pct', case when slot_class in ('weapon', 'accessory') then 5 end,
      'max_percent_affix_cap', 18,
      'set', 'dominator',
      'special_mechanism', '主宰之域'
    )) as stats,
    jsonb_build_object(
      'set', 'dominator',
      'drop_tier', 17,
      'slot_class', slot_class,
      'exclusive_source', 'world_boss_eternal_abyss_demon',
      'equipment_standard', 'dominator_2026'
    ) as flags
  from dominator_slots
)
insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags)
select id, name, kind, slot, rarity, price, false, stats, flags
from generated
on conflict (id) do update set
  name = excluded.name,
  kind = excluded.kind,
  slot = excluded.slot,
  rarity = excluded.rarity,
  price = excluded.price,
  stackable = excluded.stackable,
  stats = excluded.stats,
  flags = excluded.flags;
