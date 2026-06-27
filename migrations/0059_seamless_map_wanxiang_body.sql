-- 0.2.9: seamless world topology update and Wanxiang body-forging system.

alter table if exists vip_potion_settings
  add column if not exists auto_extract_essence_enabled boolean not null default false;

alter table if exists vip_potion_settings
  add column if not exists auto_extract_essence_max_tier integer not null default 0;

alter table if exists vip_potion_settings
  drop constraint if exists vip_potion_settings_auto_extract_essence_max_tier_check;

alter table if exists vip_potion_settings
  add constraint vip_potion_settings_auto_extract_essence_max_tier_check
  check (auto_extract_essence_max_tier between 0 and 17);

alter table if exists vip_potion_settings
  drop constraint if exists vip_potion_settings_auto_extract_exclusive_check;

alter table if exists vip_potion_settings
  add constraint vip_potion_settings_auto_extract_exclusive_check
  check (not (auto_decompose_enabled and auto_extract_essence_enabled));

create table if not exists character_wanxiang_bodies (
  character_id bigint primary key references characters(id) on delete cascade,
  level integer not null default 1 check (level between 1 and 1000),
  essence bigint not null default 0 check (essence >= 0),
  updated_at timestamptz not null default now()
);

insert into character_wanxiang_bodies (character_id)
select id from characters
on conflict (character_id) do nothing;

delete from world_rooms;
delete from world_zones;

insert into world_zones (id, name, config) values
  ('fanchen', '第一界域：凡尘界', '{"realm_index":1,"safe_hub":"qingniu_city"}'::jsonb),
  ('xiuzhen', '第二界域：修真界', '{"realm_index":2,"safe_hub":"tianshui_city","relay_hub":"ice_outpost"}'::jsonb),
  ('feisheng', '第三界域：飞升界', '{"realm_index":3,"safe_hub":"void_fortress","relay_hub":"chaos_shelter"}'::jsonb),
  ('ancient_secret', '终极探索区域', '{"realm_index":4,"safe_hub":"taichu_camp","danger":"extreme"}'::jsonb)
on conflict (id) do update set
  name = excluded.name,
  config = excluded.config;

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  ('fanchen', 'qingniu_city', '青牛城', '核心大城安全区，新手初始地，也是凡尘界四通八达的枢纽。', '{"迷雾竹林·外围":"bamboo_outer","废弃灵矿·入口":"mine_entrance","苍茫荒野·边缘":"wilderness_edge","青牛平原":"qingniu_plain"}', '[]', true),
  ('fanchen', 'bamboo_outer', 'Lv.1-3 竹林外围', '迷雾竹林外围，藤蔓与毒蛇盘踞在潮湿竹影里。', '{"青牛城":"qingniu_city","竹林岔道":"bamboo_fork"}', '["mob_demon_vine","mob_iron_bamboo_rat","mob_green_poison_snake","mob_bamboo_insect_spirit"]', false),
  ('fanchen', 'bamboo_fork', 'Lv.4-6 竹林岔道', '岔道雾气渐重，林妖与潜蛛在密竹间窥伺。', '{"竹林外围":"bamboo_outer","密林河道":"dense_river"}', '["mob_iron_bamboo_rat","mob_green_poison_snake","mob_poison_spider","mob_mist_forest_demon"]', false),
  ('fanchen', 'dense_river', 'Lv.7-9 密林河道', '密林河道边妖猪饮血，食人花妖扎根浅滩。', '{"竹林岔道":"bamboo_fork","竹林深处":"bamboo_depths"}', '["mob_red_eye_boar","mob_poison_spider","mob_cannibal_flower","mob_mist_forest_demon"]', false),
  ('fanchen', 'bamboo_depths', 'Lv.10 竹林深处', '狂暴猪王守住竹林尽头，打通后可直达废弃矿道。', '{"密林河道":"dense_river","废弃矿道":"abandoned_mine"}', '["mob_red_eye_boar","mob_cannibal_flower","boss_raging_boar_king"]', false),
  ('fanchen', 'mine_entrance', 'Lv.11-13 矿洞入口', '废弃灵矿入口，噬矿鼠与毒蝠啃噬残余灵石。', '{"青牛城":"qingniu_city","废弃矿道":"abandoned_mine"}', '["mob_ore_rat","mob_mutant_poison_bat","mob_pickaxe_skeleton","mob_wraith_flame"]', false),
  ('fanchen', 'abandoned_mine', 'Lv.14-17 废弃矿道', '尸傀与矿工残魂在废矿深处徘徊。', '{"矿洞入口":"mine_entrance","竹林深处":"bamboo_depths","塌陷暗道":"collapsed_passage"}', '["mob_wandering_corpse","mob_spirit_corpse_miner","mob_cave_rock_spider","mob_blind_scavenger_snake"]', false),
  ('fanchen', 'collapsed_passage', 'Lv.18-19 塌陷暗道', '塌陷暗道尘土未散，怨灵火种在石缝里跳动。', '{"废弃矿道":"abandoned_mine","矿长室":"foreman_room"}', '["mob_pickaxe_skeleton","mob_spirit_corpse_miner","mob_cave_rock_spider","mob_wraith_flame"]', false),
  ('fanchen', 'foreman_room', 'Lv.20 矿长室', '尸傀监工守着矿长室，击败后可直接前往荒野边缘。', '{"塌陷暗道":"collapsed_passage","荒野边缘":"wilderness_edge"}', '["mob_wandering_corpse","boss_corpse_foreman"]', false),
  ('fanchen', 'wilderness_edge', 'Lv.21-28 荒野边缘', '苍茫荒野边缘，野狼、沙蝎和荒草精怪在烈风中游荡。', '{"青牛城":"qingniu_city","矿长室":"foreman_room","兽骨高地":"bone_highland"}', '["mob_wild_wolf","mob_giant_sand_scorpion","mob_wasteland_bison","mob_blood_hyena"]', false),
  ('fanchen', 'bone_highland', 'Lv.29-35 兽骨高地', '兽骨高地上骨雕盘旋，巨蜥与枯木精怪伏击旅人。', '{"荒野边缘":"wilderness_edge","荒野深处":"wilderness_depths"}', '["mob_bloodwing_vulture","mob_rampage_lizard","mob_deadwood_spirit","mob_blood_hyena"]', false),
  ('fanchen', 'wilderness_depths', 'Lv.36-39 荒野深处', '荒野深处风沙蔽日，通往镇界石魔所在祭坛。', '{"兽骨高地":"bone_highland","荒野祭坛":"wilderness_altar"}', '["mob_wild_wolf","mob_wasteland_bison","mob_rampage_lizard","mob_deadwood_spirit"]', false),
  ('fanchen', 'wilderness_altar', 'Lv.40 荒野祭坛', '镇界石魔镇守祭坛遗迹，祭坛遗迹直通青牛平原。', '{"荒野深处":"wilderness_depths","青牛平原":"qingniu_plain"}', '["mob_giant_sand_scorpion","boss_boundary_stonemaw"]', false),
  ('fanchen', 'qingniu_plain', '青牛平原', '平原商道起点，连接青牛城与天水古道。', '{"青牛城":"qingniu_city","荒野祭坛":"wilderness_altar","天水古道":"tianshui_ancient_road"}', '["mob_plain_gale_horse","mob_demon_wandering_swordsman","mob_road_bandit"]', false),
  ('fanchen', 'tianshui_ancient_road', '天水古道', '古道怨灵与铜甲巨虫守着通往天水古城的商路。', '{"青牛平原":"qingniu_plain","天水古城":"xiuzhen:tianshui_city"}', '["mob_ancient_road_wraith","mob_bronze_horn_beetle","mob_wild_grass_spirit"]', false),

  ('xiuzhen', 'tianshui_city', '天水古城', '核心大城安全区，水乡古都，也是无尽塔的所在地。', '{"天水古道":"fanchen:tianshui_ancient_road","毒气沼泽":"poison_marsh","残塔一层":"tower_floor_1"}', '[]', true),
  ('xiuzhen', 'poison_marsh', 'Lv.41-55 毒气沼泽', '万妖谷入口，毒沼巨鳄与剧毒蟾蜍潜伏在腐水中。', '{"天水古城":"tianshui_city","百花秘境":"flower_secret"}', '["mob_poison_gator","mob_blood_mosquito","mob_mud_slime","mob_poison_toad"]', false),
  ('xiuzhen', 'flower_secret', 'Lv.56-68 百花秘境', '百花秘境香气迷离，花妖与妖藤在瘴雾中伸展。', '{"毒气沼泽":"poison_marsh","猿啼裂谷":"ape_ravine"}', '["mob_phantom_flower","mob_entangling_vine","mob_ghostface_spider","mob_poison_toad"]', false),
  ('xiuzhen', 'ape_ravine', 'Lv.69-79 猿啼裂谷', '六臂魔猿吼声回荡，铁甲犀牛踏碎峡谷石壁。', '{"百花秘境":"flower_secret","万妖洞天":"wanyao_cave"}', '["mob_six_arm_ape","mob_iron_rhino","mob_ghostface_spider","mob_mud_slime"]', false),
  ('xiuzhen', 'wanyao_cave', 'Lv.80 万妖洞天', '万妖谷主坐镇洞天，后山小径连通残塔二层。', '{"猿啼裂谷":"ape_ravine","残塔二层":"tower_floor_2"}', '["mob_six_arm_ape","mob_blood_mosquito","boss_wanyao_lord"]', false),
  ('xiuzhen', 'tower_floor_1', 'Lv.81-95 残塔一层', '镇魔残塔底层，缚灵咒徒与守塔石像环伺。', '{"天水古城":"tianshui_city","残塔二层":"tower_floor_2"}', '["mob_bound_cultist","mob_broken_blade_skeleton","mob_tower_stone_statue"]', false),
  ('xiuzhen', 'tower_floor_2', 'Lv.96-110 残塔二层', '无头镇墓将巡游残塔二层，洞天后山小径从此汇入。', '{"残塔一层":"tower_floor_1","万妖洞天":"wanyao_cave","残塔三层":"tower_floor_3"}', '["mob_headless_warden","mob_burning_blood_corpse","mob_painful_spirit"]', false),
  ('xiuzhen', 'tower_floor_3', 'Lv.111-119 残塔三层', '铁血典狱长把守残塔三层，塔顶封印正在松动。', '{"残塔二层":"tower_floor_2","塔顶封印":"tower_top_seal","塔底暗流":"tower_undercurrent"}', '["mob_iron_jailer","mob_small_nether_dragon","mob_tower_stone_statue"]', false),
  ('xiuzhen', 'tower_top_seal', 'Lv.120 塔顶封印', '怨魂聚合体撞击封印，塔底暗流通向寒冷前线。', '{"残塔三层":"tower_floor_3","塔底暗流":"tower_undercurrent"}', '["mob_painful_spirit","boss_wraith_aggregate"]', false),
  ('xiuzhen', 'tower_undercurrent', '残塔塔底暗流', '残塔塔底暗流寒意逼人，可通往破冰前哨站。', '{"残塔三层":"tower_floor_3","塔顶封印":"tower_top_seal","破冰前哨站":"ice_outpost"}', '[]', false),
  ('xiuzhen', 'ice_outpost', '破冰前哨站', '安全区中转营地，连接残塔塔底暗流与冰封雪林。', '{"残塔塔底暗流":"tower_undercurrent","冰封雪林":"ice_forest"}', '[]', true),
  ('xiuzhen', 'ice_forest', 'Lv.121-135 冰封雪林', '风雪游女与雪人勇士在冰封雪林中出没。', '{"破冰前哨站":"ice_outpost","凛冬峡谷":"winter_canyon"}', '["mob_snow_wanderer","mob_snowman_warrior","mob_ice_crystal_maiden"]', false),
  ('xiuzhen', 'winter_canyon', 'Lv.136-150 凛冬峡谷', '冰甲角魔与极地魔熊盘踞凛冬峡谷。', '{"冰封雪林":"ice_forest","龙骨冰原":"dragonbone_icefield"}', '["mob_ice_horn_demon","mob_polar_demon_bear","mob_frost_spirit","mob_icebreaking_walrus"]', false),
  ('xiuzhen', 'dragonbone_icefield', 'Lv.151-159 龙骨冰原', '极寒冰龙盘踞龙骨之间，王座就在冰原尽头。', '{"凛冬峡谷":"winter_canyon","冰原王座":"ice_throne"}', '["mob_frost_dragon","mob_frost_spirit","mob_icebreaking_walrus"]', false),
  ('xiuzhen', 'ice_throne', 'Lv.160 冰原王座', '冰原主宰镇守王座，王座背后的破冰航线可直达虚空要塞。', '{"龙骨冰原":"dragonbone_icefield","破冰航线":"icebreaking_route"}', '["mob_frost_dragon","boss_icefield_overlord"]', false),
  ('xiuzhen', 'icebreaking_route', '破冰航线', '王座背后的破冰航线，飞艇终点是虚空要塞。', '{"冰原王座":"ice_throne","虚空要塞":"feisheng:void_fortress"}', '[]', false),

  ('feisheng', 'void_fortress', '虚空要塞', '核心大城安全区，漂浮在天际的魔能都市，也是世界首领挑战入口。', '{"破冰航线码头":"xiuzhen:icebreaking_route","忘川河畔":"wangchuan_bank","幻月迷阵":"moon_maze"}', '[]', true),
  ('feisheng', 'wangchuan_bank', 'Lv.161-175 忘川河畔', '忘川河畔水鬼哀嚎，幽冥犬在雾里奔行。', '{"虚空要塞":"void_fortress","迷魂渡口":"lost_ferry"}', '["mob_wangchuan_water_ghost","mob_nether_patroller","mob_nether_dog"]', false),
  ('feisheng', 'lost_ferry', 'Lv.176-185 迷魂渡口', '骷髅艄公与勾魂使者守着迷魂渡口。', '{"忘川河畔":"wangchuan_bank","幽冥水府":"nether_water_palace"}', '["mob_soul_reaper","mob_bone_boatman","mob_grudge_wraith"]', false),
  ('feisheng', 'nether_water_palace', 'Lv.186-199 幽冥水府', '冥河夜叉与腐骨灵花在幽冥水府中滋生。', '{"迷魂渡口":"lost_ferry","判官殿":"judge_hall"}', '["mob_nether_yaksha","mob_rotten_bone_flower","mob_nether_patroller"]', false),
  ('feisheng', 'judge_hall', 'Lv.200 判官殿', '阎罗判官翻阅生死簿，水底漩涡连通银狐广场。', '{"幽冥水府":"nether_water_palace","银狐广场":"silver_fox_square"}', '["mob_soul_reaper","boss_yanluo_judge"]', false),
  ('feisheng', 'moon_maze', 'Lv.201-215 幻月迷阵', '幻月迷阵折返月光，狐妖与月光之灵在其中游走。', '{"虚空要塞":"void_fortress","银狐广场":"silver_fox_square"}', '["mob_moon_fox_demon","mob_nine_tail_phantom","mob_moonlight_spirit"]', false),
  ('feisheng', 'silver_fox_square', 'Lv.216-230 银狐广场', '银狐广场中护卫列阵，水府漩涡从广场边缘涌出。', '{"幻月迷阵":"moon_maze","判官殿":"judge_hall","拜月神台":"moon_worship_platform"}', '["mob_silver_armor_guard","mob_fox_swordsman","mob_spirit_fox_fairy"]', false),
  ('feisheng', 'moon_worship_platform', 'Lv.231-239 拜月神台', '拜月祭司与魅惑花魁守着神台。', '{"银狐广场":"silver_fox_square","天狐内殿":"skyfox_inner_palace"}', '["mob_moon_priest","mob_charming_courtesan","mob_nine_tail_phantom"]', false),
  ('feisheng', 'skyfox_inner_palace', 'Lv.240 天狐内殿', '天狐老祖坐镇内殿，内殿空间裂缝直连深渊浅层。', '{"拜月神台":"moon_worship_platform","混沌庇护所":"chaos_shelter","深渊浅层":"abyss_shallow"}', '["mob_spirit_fox_fairy","boss_skyfox_ancestor"]', false),
  ('feisheng', 'chaos_shelter', '混沌庇护所', '安全区中转营地，连接天狐内殿遗迹与深渊浅层。', '{"天狐内殿遗迹":"skyfox_inner_palace","深渊浅层":"abyss_shallow"}', '[]', true),
  ('feisheng', 'abyss_shallow', 'Lv.241-260 深渊浅层', '深渊浅层魔眼睁开，虚空爬行者贴地潜行。', '{"混沌庇护所":"chaos_shelter","天狐内殿":"skyfox_inner_palace","深渊魔窟":"abyss_cave"}', '["mob_abyss_demon_eye","mob_void_crawler","mob_chaos_leech"]', false),
  ('feisheng', 'abyss_cave', 'Lv.261-280 深渊魔窟', '嗜血裂空兽与渊狱魔锤在深渊魔窟中游荡。', '{"深渊浅层":"abyss_shallow","混沌风暴":"chaos_storm"}', '["mob_blood_rift_beast","mob_abyss_hammer","mob_nightmare_devourer"]', false),
  ('feisheng', 'chaos_storm', 'Lv.281-299 混沌风暴', '混沌风暴灵与黯灭星魔在风暴核心翻涌。', '{"深渊魔窟":"abyss_cave","渊兽巢穴":"abyss_beast_lair"}', '["mob_chaos_storm_spirit","mob_dark_star_demon","mob_nightmare_devourer"]', false),
  ('feisheng', 'abyss_beast_lair', 'Lv.300 渊兽巢穴', '混沌渊兽王盘踞巢穴，巢穴后方登天雷池直达太初远征营地。', '{"混沌风暴":"chaos_storm","登天雷池":"ascension_thunder_pool"}', '["mob_blood_rift_beast","boss_chaos_abyss_beast"]', false),
  ('feisheng', 'ascension_thunder_pool', '登天雷池', '穿过渊兽巢穴后方的登天雷池，即可抵达太初远征营地。', '{"渊兽巢穴":"abyss_beast_lair","太初远征营地":"ancient_secret:taichu_camp"}', '[]', false),

  ('ancient_secret', 'taichu_camp', '太初远征营地', '大型安全区，直连古矿区、大泽区、仙门遗址，形成终极环形探索区域。', '{"登天雷池":"feisheng:ascension_thunder_pool","矿区外围":"mining_outer","瘴气毒林":"miasma_forest","破败山门":"ruined_gate"}', '[]', true),
  ('ancient_secret', 'mining_outer', 'Lv.301-320 矿区外围', '太初古矿外围，源石傀儡与寻灵鼠王游荡。', '{"太初远征营地":"taichu_camp","源石晶洞":"source_crystal_cave"}', '["mob_source_golem","mob_crystallized_mine_demon","mob_spirit_seeking_rat_king"]', false),
  ('ancient_secret', 'source_crystal_cave', 'Lv.321-339 源石晶洞', '源石晶洞深处异化晶龙和钻地沙虫潜伏。', '{"矿区外围":"mining_outer","源石王座":"source_throne"}', '["mob_primordial_rock_spirit","mob_companion_spirit_beast","mob_mutant_crystal_dragon","mob_drilling_sandworm"]', false),
  ('ancient_secret', 'source_throne', 'Lv.340 源石王座', '源石傀儡王守着古矿王座，挖穿石壁可横向连通吞天蛇沼。', '{"源石晶洞":"source_crystal_cave","吞天蛇沼":"devouring_snake_marsh"}', '["mob_ancient_mine_guard","boss_source_golem_king"]', false),
  ('ancient_secret', 'miasma_forest', 'Lv.341-360 瘴气毒林', '洪荒大泽入口，剧毒飞龙与腐烂古树在瘴气里盘旋。', '{"太初远征营地":"taichu_camp","吞天蛇沼":"devouring_snake_marsh"}', '["mob_poison_dragon","mob_three_eye_toad","mob_rotten_ancient_tree"]', false),
  ('ancient_secret', 'devouring_snake_marsh', 'Lv.361-379 吞天蛇沼', '吞天巨蟒与吸血巨蛭潜伏在深沼，古矿石壁从此贯通。', '{"瘴气毒林":"miasma_forest","源石王座":"source_throne","巨猿领地":"giant_ape_domain"}', '["mob_devouring_python","mob_primordial_crocodile","mob_poison_mudling","mob_vampire_giant_leech"]', false),
  ('ancient_secret', 'giant_ape_domain', 'Lv.380 巨猿领地', '洪荒巨猿撼动大泽古木，巨木桥连通破败山门。', '{"吞天蛇沼":"devouring_snake_marsh","破败山门":"ruined_gate"}', '["mob_swamp_floater","boss_primordial_giant_ape"]', false),
  ('ancient_secret', 'ruined_gate', 'Lv.381-400 破败山门', '造化仙门遗址入口，堕落天兵与符文石像仍在巡守。', '{"太初远征营地":"taichu_camp","巨猿领地":"giant_ape_domain","仙门天阶":"immortal_steps"}', '["mob_fallen_heaven_soldier","mob_rune_stone_statue","mob_ruined_sword_spirit"]', false),
  ('ancient_secret', 'immortal_steps', 'Lv.401-419 仙门天阶', '仙门天阶直抵云端，仙门剑阵与幻音仙女阻住前路。', '{"破败山门":"ruined_gate","凌霄残殿":"lingxiao_ruins","灰烬平原":"ash_plain"}', '["mob_unconscious_sword_cultivator","mob_mountain_guard_beast","mob_possessed_cultivator","mob_immortal_sword_array","mob_illusion_sound_fairy"]', false),
  ('ancient_secret', 'lingxiao_ruins', 'Lv.420 凌霄残殿', '堕落谪仙守着残殿传送门，传送门连通浴火神巢。', '{"仙门天阶":"immortal_steps","浴火神巢":"fire_nest"}', '["mob_ruined_sword_spirit","boss_fallen_exiled_immortal"]', false),
  ('ancient_secret', 'ash_plain', 'Lv.421-440 灰烬平原', '灰烬平原热浪翻涌，灰烬火灵与余烬法师在火线徘徊。', '{"仙门天阶":"immortal_steps","浴火神巢":"fire_nest"}', '["mob_ash_fire_spirit","mob_volcanic_stone_demon","mob_ember_mage"]', false),
  ('ancient_secret', 'fire_nest', 'Lv.441-459 浴火神巢', '浴火神巢深处焚天神雀盘旋，神凰分身即将苏醒。', '{"灰烬平原":"ash_plain","凌霄残殿":"lingxiao_ruins","涅槃核心":"nirvana_core"}', '["mob_burning_sky_sparrow","mob_lava_giant","mob_flame_two_head_hound","mob_baby_phoenix","mob_flame_demon_commander"]', false),
  ('ancient_secret', 'nirvana_core', 'Lv.460 涅槃核心', '不死神凰分身在涅槃核心中燃烧，星火可送你飞升至碎星带。', '{"浴火神巢":"fire_nest","碎星带":"broken_stars"}', '["mob_flame_demon_commander","boss_phoenix_avatar"]', false),
  ('ancient_secret', 'broken_stars', 'Lv.461-480 碎星带', '碎星带星屑如刃，星空巨兽和星云鳐鱼游弋其间。', '{"涅槃核心":"nirvana_core","巨兽陨石":"beast_meteor"}', '["mob_star_behemoth","mob_nebula_ray","mob_meteor_star_crab","mob_chaos_star_spirit"]', false),
  ('ancient_secret', 'beast_meteor', 'Lv.481-498 巨兽陨石', '巨兽陨石周围黑洞异魔潜伏，维度观察者凝视旅人。', '{"碎星带":"broken_stars","天道祭坛":"heavenly_dao_altar"}', '["mob_void_devourer","mob_blackhole_demon","mob_dimension_observer"]', false),
  ('ancient_secret', 'heavenly_dao_altar', 'Lv.499-500 天道祭坛', '鸿蒙天道幻影镇守祭坛，祭坛连接无怪物的星际观测台。', '{"巨兽陨石":"beast_meteor","星际观测台":"stargazer_observatory"}', '["mob_hongmeng_daoling","mob_chaos_star_spirit","boss_heavenly_dao_phantom"]', false),
  ('ancient_secret', 'stargazer_observatory', '星际观测台', '无怪物观测台，第一次抵达此地象征你已走到大世界边界。', '{"天道祭坛":"heavenly_dao_altar"}', '[]', false)
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

with mob_defs(id, name, level, boss, respawn_seconds) as (
  values
    ('mob_demon_vine','魔化藤蔓',2,false,45),('mob_iron_bamboo_rat','铁甲竹鼠',4,false,45),('mob_green_poison_snake','碧磷毒蛇',5,false,45),('mob_red_eye_boar','赤目妖猪',7,false,60),('mob_poison_spider','剧毒潜蛛',8,false,60),('mob_cannibal_flower','食人花妖',9,false,60),('mob_mist_forest_demon','迷雾林妖',9,false,60),('mob_bamboo_insect_spirit','竹节虫精',6,false,45),('boss_raging_boar_king','狂暴猪王',10,true,900),
    ('mob_ore_rat','噬矿鼠',12,false,75),('mob_wandering_corpse','游荡尸傀',14,false,75),('mob_mutant_poison_bat','变异毒蝠',15,false,75),('mob_pickaxe_skeleton','铁镐骷髅',16,false,90),('mob_spirit_corpse_miner','灵尸矿工',18,false,90),('mob_cave_rock_spider','地穴岩蛛',18,false,90),('mob_blind_scavenger_snake','食腐盲蛇',19,false,90),('mob_wraith_flame','怨灵火种',19,false,90),('boss_corpse_foreman','尸傀监工',20,true,1200),
    ('mob_wild_wolf','荒原野狼',23,false,110),('mob_bloodwing_vulture','血翼骨雕',29,false,120),('mob_giant_sand_scorpion','巨型沙蝎',31,false,120),('mob_wasteland_bison','荒原野牛',33,false,130),('mob_rampage_lizard','狂暴巨蜥',35,false,130),('mob_deadwood_spirit','枯木精怪',37,false,140),('mob_blood_hyena','嗜血鬣狗',39,false,140),('boss_boundary_stonemaw','镇界石魔',40,true,1800),
    ('mob_plain_gale_horse','平原疾风马',30,false,120),('mob_demon_wandering_swordsman','魔化流浪剑客',34,false,130),('mob_road_bandit','劫道山贼',36,false,130),('mob_ancient_road_wraith','古道怨灵',38,false,140),('mob_bronze_horn_beetle','铜甲巨角虫',40,false,140),('mob_wild_grass_spirit','荒草野精',40,false,140),
    ('mob_poison_gator','毒沼巨鳄',48,false,180),('mob_phantom_flower','幻影花妖',60,false,200),('mob_six_arm_ape','六臂魔猿',74,false,220),('mob_blood_mosquito','嗜血魔蚊',52,false,180),('mob_iron_rhino','铁甲犀牛',68,false,210),('mob_entangling_vine','缠人妖藤',62,false,200),('mob_ghostface_spider','鬼面魔蛛',72,false,220),('mob_mud_slime','泥沼史莱姆',58,false,190),('mob_poison_toad','剧毒蟾蜍',66,false,210),('boss_wanyao_lord','万妖谷主',80,true,2400),
    ('mob_bound_cultist','缚灵咒徒',88,false,260),('mob_headless_warden','无头镇墓将',102,false,300),('mob_iron_jailer','铁血典狱长',116,false,340),('mob_burning_blood_corpse','燃血尸魔',100,false,300),('mob_small_nether_dragon','幽冥骨龙',112,false,330),('mob_painful_spirit','痛苦妖灵',118,false,340),('mob_broken_blade_skeleton','断刃骷髅',92,false,270),('mob_tower_stone_statue','守塔石像',108,false,320),('boss_wraith_aggregate','怨魂聚合体',120,true,3000),
    ('mob_snow_wanderer','风雪游女',130,false,380),('mob_ice_horn_demon','冰甲角魔',142,false,420),('mob_frost_dragon','极寒冰龙',156,false,460),('mob_snowman_warrior','雪人勇士',126,false,360),('mob_ice_crystal_maiden','冰晶雪女',134,false,390),('mob_polar_demon_bear','极地魔熊',148,false,430),('mob_frost_spirit','霜冻灵体',152,false,440),('mob_icebreaking_walrus','破冰海象',150,false,430),('boss_icefield_overlord','冰原主宰',160,true,3600),
    ('mob_wangchuan_water_ghost','忘川水鬼',168,false,500),('mob_nether_patroller','幽冥巡游者',174,false,520),('mob_soul_reaper','勾魂使者',182,false,540),('mob_nether_yaksha','冥河夜叉',192,false,580),('mob_bone_boatman','骷髅艄公',180,false,540),('mob_grudge_wraith','怨念游魂',188,false,560),('mob_rotten_bone_flower','腐骨灵花',196,false,580),('mob_nether_dog','幽冥犬',176,false,520),('boss_yanluo_judge','阎罗判官',200,true,4200),
    ('mob_moon_fox_demon','幻月狐妖',208,false,620),('mob_silver_armor_guard','银甲护卫',222,false,660),('mob_moon_priest','拜月祭司',234,false,700),('mob_charming_courtesan','魅惑花魁',230,false,690),('mob_fox_swordsman','狐族剑客',224,false,670),('mob_nine_tail_phantom','九尾幻影',238,false,710),('mob_spirit_fox_fairy','灵狐仙子',236,false,700),('mob_moonlight_spirit','月光之灵',214,false,640),('boss_skyfox_ancestor','天狐老祖',240,true,4800),
    ('mob_abyss_demon_eye','渊生魔眼',250,false,740),('mob_chaos_storm_spirit','混沌风暴灵',286,false,840),('mob_blood_rift_beast','嗜血裂空兽',270,false,800),('mob_void_crawler','虚空爬行者',258,false,760),('mob_abyss_hammer','渊狱魔锤',278,false,820),('mob_nightmare_devourer','梦魇吞噬者',290,false,860),('mob_dark_star_demon','黯灭星魔',298,false,880),('mob_chaos_leech','混沌巨蛭',264,false,780),('boss_chaos_abyss_beast','混沌渊兽王',300,true,5400),
    ('mob_source_golem','源石傀儡',310,false,920),('mob_crystallized_mine_demon','晶化矿妖',322,false,960),('mob_primordial_rock_spirit','太初岩精',330,false,980),('mob_companion_spirit_beast','伴生灵兽',326,false,970),('mob_spirit_seeking_rat_king','寻灵鼠王',318,false,940),('mob_mutant_crystal_dragon','异化晶龙',336,false,1000),('mob_drilling_sandworm','钻地沙虫',334,false,990),('mob_ancient_mine_guard','古矿守卫',338,false,1000),('boss_source_golem_king','源石傀儡王',340,true,6000),
    ('mob_poison_dragon','剧毒飞龙',350,false,1040),('mob_devouring_python','吞天巨蟒',366,false,1100),('mob_primordial_crocodile','洪荒巨鳄',358,false,1060),('mob_three_eye_toad','三眼魔蟾',354,false,1050),('mob_poison_mudling','剧毒泥怪',362,false,1080),('mob_vampire_giant_leech','吸血巨蛭',370,false,1120),('mob_rotten_ancient_tree','腐烂古树',356,false,1060),('mob_swamp_floater','沼泽浮游',374,false,1140),('boss_primordial_giant_ape','洪荒巨猿',380,true,6600),
    ('mob_fallen_heaven_soldier','堕落天兵',390,false,1160),('mob_unconscious_sword_cultivator','无意识剑修',402,false,1200),('mob_mountain_guard_beast','护山仙兽',406,false,1220),('mob_possessed_cultivator','走火入魔者',410,false,1240),('mob_immortal_sword_array','仙门剑阵',414,false,1260),('mob_rune_stone_statue','符文石像',396,false,1180),('mob_ruined_sword_spirit','破灭剑灵',418,false,1280),('mob_illusion_sound_fairy','幻音仙女',412,false,1250),('boss_fallen_exiled_immortal','堕落谪仙',420,true,7200),
    ('mob_ash_fire_spirit','灰烬火灵',430,false,1320),('mob_burning_sky_sparrow','焚天神雀',446,false,1380),('mob_lava_giant','熔岩巨人',450,false,1400),('mob_flame_two_head_hound','烈焰双头犬',444,false,1370),('mob_volcanic_stone_demon','火山石魔',438,false,1350),('mob_baby_phoenix','不死鸟幼体',454,false,1420),('mob_flame_demon_commander','炎魔统领',458,false,1440),('mob_ember_mage','余烬法师',436,false,1340),('boss_phoenix_avatar','不死神凰分身',460,true,7800),
    ('mob_star_behemoth','星空巨兽',470,false,1500),('mob_void_devourer','虚空吞噬者',486,false,1560),('mob_hongmeng_daoling','鸿蒙道灵',499,false,1620),('mob_meteor_star_crab','陨石星蟹',482,false,1540),('mob_nebula_ray','星云鳐鱼',476,false,1520),('mob_blackhole_demon','黑洞异魔',492,false,1580),('mob_dimension_observer','维度观察者',496,false,1600),('mob_chaos_star_spirit','混沌星灵',500,false,1640),('boss_heavenly_dao_phantom','鸿蒙天道幻影',500,true,8400)
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
  case when boss then normal_hp * 20 else normal_hp end as max_hp,
  case when boss then normal_atk * 2 else normal_atk end as atk,
  normal_def as def,
  greatest(10, round(level::double precision * level::double precision * 35)::bigint) as exp,
  greatest(5, round(level::double precision * level::double precision * 9)::bigint) as gold,
  boss,
  respawn_seconds,
  jsonb_build_object('drop_model', 'global_dynamic_tier_1_17')
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

update character_state
set zone = 'fanchen', room = 'qingniu_city'
where not exists (
  select 1
  from world_rooms wr
  where wr.zone_id = character_state.zone
    and wr.id = character_state.room
);

update bot_profiles
set
  zone = case
    when level <= 40 then 'fanchen'
    when level <= 160 then 'xiuzhen'
    when level <= 300 then 'feisheng'
    else 'ancient_secret'
  end,
  room = case
    when level between 1 and 3 then 'bamboo_outer'
    when level between 4 and 6 then 'bamboo_fork'
    when level between 7 and 10 then 'dense_river'
    when level between 11 and 17 then 'mine_entrance'
    when level between 18 and 20 then 'collapsed_passage'
    when level between 21 and 28 then 'wilderness_edge'
    when level between 29 and 35 then 'bone_highland'
    when level between 36 and 40 then 'wilderness_depths'
    when level between 41 and 55 then 'poison_marsh'
    when level between 56 and 68 then 'flower_secret'
    when level between 69 and 80 then 'ape_ravine'
    when level between 81 and 95 then 'tower_floor_1'
    when level between 96 and 110 then 'tower_floor_2'
    when level between 111 and 120 then 'tower_floor_3'
    when level between 121 and 135 then 'ice_forest'
    when level between 136 and 150 then 'winter_canyon'
    when level between 151 and 160 then 'dragonbone_icefield'
    when level between 161 and 185 then 'wangchuan_bank'
    when level between 186 and 200 then 'nether_water_palace'
    when level between 201 and 230 then 'moon_maze'
    when level between 231 and 240 then 'moon_worship_platform'
    when level between 241 and 260 then 'abyss_shallow'
    when level between 261 and 280 then 'abyss_cave'
    when level between 281 and 300 then 'chaos_storm'
    when level between 301 and 340 then 'mining_outer'
    when level between 341 and 380 then 'miasma_forest'
    when level between 381 and 420 then 'ruined_gate'
    when level between 421 and 460 then 'ash_plain'
    when level between 461 and 498 then 'broken_stars'
    else 'heavenly_dao_altar'
  end
where not exists (
  select 1
  from world_rooms wr
  where wr.zone_id = bot_profiles.zone
    and wr.id = bot_profiles.room
);
