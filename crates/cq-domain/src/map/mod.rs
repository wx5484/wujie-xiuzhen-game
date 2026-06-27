use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub exits: BTreeMap<String, String>,
    pub spawns: Vec<String>,
    pub safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub rooms: BTreeMap<String, Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub zones: BTreeMap<String, Zone>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub zone: String,
    pub room: String,
}

impl World {
    pub fn room(&self, position: &Position) -> Option<&Room> {
        self.zones.get(&position.zone)?.rooms.get(&position.room)
    }

    pub fn resolve_exit(&self, position: &Position, direction: &str) -> Option<Position> {
        let current = self.room(position)?;
        let dest = current.exits.get(direction)?;
        if let Some((zone, room)) = dest.split_once(':') {
            return Some(Position { zone: zone.into(), room: room.into() });
        }
        Some(Position { zone: position.zone.clone(), room: dest.clone() })
    }
}

pub fn start_position() -> Position {
    Position { zone: "fanchen".into(), room: "qingniu_city".into() }
}

pub fn death_return_position(position: &Position) -> Position {
    match position.zone.as_str() {
        "fanchen" => Position { zone: "fanchen".into(), room: "qingniu_city".into() },
        "xiuzhen" => Position { zone: "xiuzhen".into(), room: "tianshui_city".into() },
        "feisheng" => Position { zone: "feisheng".into(), room: "void_fortress".into() },
        "ancient_secret" => Position { zone: "ancient_secret".into(), room: "taichu_camp".into() },
        _ => start_position(),
    }
}

pub fn death_return_name(position: &Position) -> &'static str {
    match position.zone.as_str() {
        "fanchen" => "青牛城",
        "xiuzhen" => "天水古城",
        "feisheng" => "虚空要塞",
        "ancient_secret" => "太初远征营地",
        _ => "青牛城",
    }
}

pub fn starter_world() -> World {
    fn room(
        id: &str,
        name: &str,
        desc: &str,
        exits: &[(&str, &str)],
        spawns: &[&str],
        safe: bool,
    ) -> Room {
        Room {
            id: id.into(),
            name: name.into(),
            desc: desc.into(),
            exits: exits.iter().map(|(label, target)| ((*label).into(), (*target).into())).collect(),
            spawns: spawns.iter().map(|spawn| (*spawn).into()).collect(),
            safe,
        }
    }

    let mut zones = BTreeMap::new();

    let mut fanchen_rooms = BTreeMap::new();
    fanchen_rooms.insert("qingniu_city".into(), room("qingniu_city", "青牛城", "凡尘界核心大城安全区。", &[("迷雾竹林·外围", "bamboo_outer"), ("废弃灵矿·入口", "mine_entrance"), ("苍茫荒野·边缘", "wilderness_edge"), ("青牛平原", "qingniu_plain")], &[], true));
    fanchen_rooms.insert("bamboo_outer".into(), room("bamboo_outer", "Lv.1-3 竹林外围", "迷雾竹林外围。", &[("青牛城", "qingniu_city"), ("竹林岔道", "bamboo_fork")], &["mob_demon_vine"], false));
    fanchen_rooms.insert("bamboo_fork".into(), room("bamboo_fork", "Lv.4-6 竹林岔道", "密竹间雾气渐重。", &[("竹林外围", "bamboo_outer"), ("密林河道", "dense_river")], &["mob_iron_bamboo_rat"], false));
    fanchen_rooms.insert("dense_river".into(), room("dense_river", "Lv.7-9 密林河道", "赤目妖猪在河道边游荡。", &[("竹林岔道", "bamboo_fork"), ("竹林深处", "bamboo_depths")], &["mob_red_eye_boar"], false));
    fanchen_rooms.insert("bamboo_depths".into(), room("bamboo_depths", "Lv.10 竹林深处", "狂暴猪王守着竹林尽头。", &[("密林河道", "dense_river"), ("废弃矿道", "abandoned_mine")], &["boss_raging_boar_king"], false));
    fanchen_rooms.insert("mine_entrance".into(), room("mine_entrance", "Lv.11-13 矿洞入口", "废弃灵矿入口。", &[("青牛城", "qingniu_city"), ("废弃矿道", "abandoned_mine")], &["mob_ore_rat"], false));
    fanchen_rooms.insert("abandoned_mine".into(), room("abandoned_mine", "Lv.14-17 废弃矿道", "尸傀与矿工残魂徘徊。", &[("矿洞入口", "mine_entrance"), ("竹林深处", "bamboo_depths"), ("塌陷暗道", "collapsed_passage")], &["mob_wandering_corpse"], false));
    fanchen_rooms.insert("collapsed_passage".into(), room("collapsed_passage", "Lv.18-19 塌陷暗道", "塌陷暗道尘土未散。", &[("废弃矿道", "abandoned_mine"), ("矿长室", "foreman_room")], &["mob_spirit_corpse_miner"], false));
    fanchen_rooms.insert("foreman_room".into(), room("foreman_room", "Lv.20 矿长室", "尸傀监工守着矿长室。", &[("塌陷暗道", "collapsed_passage"), ("荒野边缘", "wilderness_edge")], &["boss_corpse_foreman"], false));
    fanchen_rooms.insert("wilderness_edge".into(), room("wilderness_edge", "Lv.21-28 荒野边缘", "苍茫荒野边缘。", &[("青牛城", "qingniu_city"), ("矿长室", "foreman_room"), ("兽骨高地", "bone_highland")], &["mob_wild_wolf"], false));
    fanchen_rooms.insert("bone_highland".into(), room("bone_highland", "Lv.29-35 兽骨高地", "血翼骨雕盘旋其上。", &[("荒野边缘", "wilderness_edge"), ("荒野深处", "wilderness_depths")], &["mob_bloodwing_vulture"], false));
    fanchen_rooms.insert("wilderness_depths".into(), room("wilderness_depths", "Lv.36-39 荒野深处", "风沙通往祭坛。", &[("兽骨高地", "bone_highland"), ("荒野祭坛", "wilderness_altar")], &["mob_rampage_lizard"], false));
    fanchen_rooms.insert("wilderness_altar".into(), room("wilderness_altar", "Lv.40 荒野祭坛", "镇界石魔镇守此地。", &[("荒野深处", "wilderness_depths"), ("青牛平原", "qingniu_plain")], &["boss_boundary_stonemaw"], false));
    fanchen_rooms.insert("qingniu_plain".into(), room("qingniu_plain", "青牛平原", "平原商道起点。", &[("青牛城", "qingniu_city"), ("荒野祭坛", "wilderness_altar"), ("天水古道", "tianshui_ancient_road")], &["mob_plain_gale_horse"], false));
    fanchen_rooms.insert("tianshui_ancient_road".into(), room("tianshui_ancient_road", "天水古道", "通往天水古城的商路。", &[("青牛平原", "qingniu_plain"), ("天水古城", "xiuzhen:tianshui_city")], &["mob_ancient_road_wraith"], false));
    zones.insert("fanchen".into(), Zone { id: "fanchen".into(), name: "第一界域：凡尘界".into(), rooms: fanchen_rooms });

    let mut xiuzhen_rooms = BTreeMap::new();
    xiuzhen_rooms.insert("tianshui_city".into(), room("tianshui_city", "天水古城", "修真界核心大城安全区。", &[("天水古道", "fanchen:tianshui_ancient_road"), ("天水书院", "tianshui_academy"), ("毒气沼泽", "poison_marsh"), ("残塔一层", "tower_floor_1")], &[], true));
    xiuzhen_rooms.insert("tianshui_academy".into(), room("tianshui_academy", "天水书院", "天水古城内的书院，可购买各职业初期技能。", &[("天水古城", "tianshui_city")], &[], true));
    xiuzhen_rooms.insert("poison_marsh".into(), room("poison_marsh", "Lv.41-55 毒气沼泽", "万妖谷入口。", &[("天水古城", "tianshui_city"), ("百花秘境", "flower_secret")], &["mob_poison_gator"], false));
    xiuzhen_rooms.insert("flower_secret".into(), room("flower_secret", "Lv.56-68 百花秘境", "花妖潜藏其中。", &[("毒气沼泽", "poison_marsh"), ("猿啼裂谷", "ape_ravine")], &["mob_phantom_flower"], false));
    xiuzhen_rooms.insert("ape_ravine".into(), room("ape_ravine", "Lv.69-79 猿啼裂谷", "六臂魔猿怒吼。", &[("百花秘境", "flower_secret"), ("万妖洞天", "wanyao_cave")], &["mob_six_arm_ape"], false));
    xiuzhen_rooms.insert("wanyao_cave".into(), room("wanyao_cave", "Lv.80 万妖洞天", "万妖谷主坐镇洞天。", &[("猿啼裂谷", "ape_ravine"), ("残塔二层", "tower_floor_2")], &["boss_wanyao_lord"], false));
    xiuzhen_rooms.insert("tower_floor_1".into(), room("tower_floor_1", "Lv.81-95 残塔一层", "镇魔残塔底层。", &[("天水古城", "tianshui_city"), ("残塔二层", "tower_floor_2")], &["mob_bound_cultist"], false));
    xiuzhen_rooms.insert("tower_floor_2".into(), room("tower_floor_2", "Lv.96-110 残塔二层", "无头镇墓将巡游。", &[("残塔一层", "tower_floor_1"), ("万妖洞天", "wanyao_cave"), ("残塔三层", "tower_floor_3")], &["mob_headless_warden"], false));
    xiuzhen_rooms.insert("tower_floor_3".into(), room("tower_floor_3", "Lv.111-119 残塔三层", "塔顶封印正在松动。", &[("残塔二层", "tower_floor_2"), ("塔顶封印", "tower_top_seal"), ("塔底暗流", "tower_undercurrent")], &["mob_iron_jailer"], false));
    xiuzhen_rooms.insert("tower_top_seal".into(), room("tower_top_seal", "Lv.120 塔顶封印", "怨魂聚合体撞击封印。", &[("残塔三层", "tower_floor_3"), ("塔底暗流", "tower_undercurrent")], &["boss_wraith_aggregate"], false));
    xiuzhen_rooms.insert("tower_undercurrent".into(), room("tower_undercurrent", "残塔塔底暗流", "暗流通往寒冷前线。", &[("残塔三层", "tower_floor_3"), ("破冰前哨站", "ice_outpost")], &[], false));
    xiuzhen_rooms.insert("ice_outpost".into(), room("ice_outpost", "破冰前哨站", "中转营地安全区。", &[("残塔塔底暗流", "tower_undercurrent"), ("冰封雪林", "ice_forest"), ("炼狱", "purgatory")], &[], true));
    xiuzhen_rooms.insert("purgatory".into(), room("purgatory", "炼狱", "破冰前哨站下方的苦修火窟，可打坐修炼等级经验，不产出装备、金币或材料。", &[("破冰前哨站", "ice_outpost")], &[], true));
    xiuzhen_rooms.insert("ice_forest".into(), room("ice_forest", "Lv.121-135 冰封雪林", "冰雪覆盖林地。", &[("破冰前哨站", "ice_outpost"), ("凛冬峡谷", "winter_canyon")], &["mob_snow_wanderer"], false));
    xiuzhen_rooms.insert("winter_canyon".into(), room("winter_canyon", "Lv.136-150 凛冬峡谷", "寒风切割峡谷。", &[("冰封雪林", "ice_forest"), ("龙骨冰原", "dragonbone_icefield")], &["mob_ice_horn_demon"], false));
    xiuzhen_rooms.insert("dragonbone_icefield".into(), room("dragonbone_icefield", "Lv.151-159 龙骨冰原", "极寒冰龙盘踞。", &[("凛冬峡谷", "winter_canyon"), ("冰原王座", "ice_throne")], &["mob_frost_dragon"], false));
    xiuzhen_rooms.insert("ice_throne".into(), room("ice_throne", "Lv.160 冰原王座", "冰原主宰镇守王座。", &[("龙骨冰原", "dragonbone_icefield"), ("破冰航线", "icebreaking_route")], &["boss_icefield_overlord"], false));
    xiuzhen_rooms.insert("icebreaking_route".into(), room("icebreaking_route", "破冰航线", "飞艇终点是虚空要塞。", &[("冰原王座", "ice_throne"), ("虚空要塞", "feisheng:void_fortress")], &[], false));
    zones.insert("xiuzhen".into(), Zone { id: "xiuzhen".into(), name: "第二界域：修真界".into(), rooms: xiuzhen_rooms });

    let mut feisheng_rooms = BTreeMap::new();
    feisheng_rooms.insert("void_fortress".into(), room("void_fortress", "虚空要塞", "飞升界核心大城安全区。", &[("破冰航线码头", "xiuzhen:icebreaking_route"), ("虚空市集", "void_market"), ("忘川河畔", "wangchuan_bank"), ("幻月迷阵", "moon_maze")], &[], true));
    feisheng_rooms.insert("void_market".into(), room("void_market", "虚空市集", "虚空要塞内的魔能市集，可购买各职业中期技能。", &[("虚空要塞", "void_fortress")], &[], true));
    feisheng_rooms.insert("wangchuan_bank".into(), room("wangchuan_bank", "Lv.161-175 忘川河畔", "忘川水鬼哀嚎。", &[("虚空要塞", "void_fortress"), ("迷魂渡口", "lost_ferry")], &["mob_wangchuan_water_ghost"], false));
    feisheng_rooms.insert("lost_ferry".into(), room("lost_ferry", "Lv.176-185 迷魂渡口", "骷髅艄公守渡口。", &[("忘川河畔", "wangchuan_bank"), ("幽冥水府", "nether_water_palace")], &["mob_soul_reaper"], false));
    feisheng_rooms.insert("nether_water_palace".into(), room("nether_water_palace", "Lv.186-199 幽冥水府", "冥河夜叉潜伏。", &[("迷魂渡口", "lost_ferry"), ("判官殿", "judge_hall")], &["mob_nether_yaksha"], false));
    feisheng_rooms.insert("judge_hall".into(), room("judge_hall", "Lv.200 判官殿", "阎罗判官坐镇。", &[("幽冥水府", "nether_water_palace"), ("银狐广场", "silver_fox_square")], &["boss_yanluo_judge"], false));
    feisheng_rooms.insert("moon_maze".into(), room("moon_maze", "Lv.201-215 幻月迷阵", "月光折返如镜。", &[("虚空要塞", "void_fortress"), ("银狐广场", "silver_fox_square")], &["mob_moon_fox_demon"], false));
    feisheng_rooms.insert("silver_fox_square".into(), room("silver_fox_square", "Lv.216-230 银狐广场", "银甲护卫列阵。", &[("幻月迷阵", "moon_maze"), ("判官殿", "judge_hall"), ("拜月神台", "moon_worship_platform")], &["mob_silver_armor_guard"], false));
    feisheng_rooms.insert("moon_worship_platform".into(), room("moon_worship_platform", "Lv.231-239 拜月神台", "拜月祭司守台。", &[("银狐广场", "silver_fox_square"), ("天狐内殿", "skyfox_inner_palace")], &["mob_moon_priest"], false));
    feisheng_rooms.insert("skyfox_inner_palace".into(), room("skyfox_inner_palace", "Lv.240 天狐内殿", "天狐老祖镇守。", &[("拜月神台", "moon_worship_platform"), ("混沌庇护所", "chaos_shelter"), ("深渊浅层", "abyss_shallow")], &["boss_skyfox_ancestor"], false));
    feisheng_rooms.insert("chaos_shelter".into(), room("chaos_shelter", "混沌庇护所", "混沌深渊中转安全区。", &[("天狐内殿遗迹", "skyfox_inner_palace"), ("深渊浅层", "abyss_shallow"), ("虚境", "void_realm")], &[], true));
    feisheng_rooms.insert("void_realm".into(), room("void_realm", "虚境", "混沌庇护所内的心识秘境，可指定已学习技能打坐研修，只增长技能经验。", &[("混沌庇护所", "chaos_shelter")], &[], true));
    feisheng_rooms.insert("abyss_shallow".into(), room("abyss_shallow", "Lv.241-260 深渊浅层", "深渊魔眼睁开。", &[("混沌庇护所", "chaos_shelter"), ("天狐内殿", "skyfox_inner_palace"), ("深渊魔窟", "abyss_cave")], &["mob_abyss_demon_eye"], false));
    feisheng_rooms.insert("abyss_cave".into(), room("abyss_cave", "Lv.261-280 深渊魔窟", "裂空兽游荡。", &[("深渊浅层", "abyss_shallow"), ("混沌风暴", "chaos_storm")], &["mob_blood_rift_beast"], false));
    feisheng_rooms.insert("chaos_storm".into(), room("chaos_storm", "Lv.281-299 混沌风暴", "风暴核心翻涌。", &[("深渊魔窟", "abyss_cave"), ("渊兽巢穴", "abyss_beast_lair")], &["mob_chaos_storm_spirit"], false));
    feisheng_rooms.insert("abyss_beast_lair".into(), room("abyss_beast_lair", "Lv.300 渊兽巢穴", "混沌渊兽王盘踞。", &[("混沌风暴", "chaos_storm"), ("登天雷池", "ascension_thunder_pool")], &["boss_chaos_abyss_beast"], false));
    feisheng_rooms.insert("ascension_thunder_pool".into(), room("ascension_thunder_pool", "登天雷池", "雷池通往太初远征营地。", &[("渊兽巢穴", "abyss_beast_lair"), ("太初远征营地", "ancient_secret:taichu_camp")], &[], false));
    zones.insert("feisheng".into(), Zone { id: "feisheng".into(), name: "第三界域：飞升界".into(), rooms: feisheng_rooms });

    let mut ancient_rooms = BTreeMap::new();
    ancient_rooms.insert("taichu_camp".into(), room("taichu_camp", "太初远征营地", "终极探索区域大型安全区。", &[("登天雷池", "feisheng:ascension_thunder_pool"), ("矿区外围", "mining_outer"), ("瘴气毒林", "miasma_forest"), ("破败山门", "ruined_gate")], &[], true));
    ancient_rooms.insert("mining_outer".into(), room("mining_outer", "Lv.301-320 矿区外围", "源石傀儡游荡。", &[("太初远征营地", "taichu_camp"), ("源石晶洞", "source_crystal_cave")], &["mob_source_golem"], false));
    ancient_rooms.insert("source_crystal_cave".into(), room("source_crystal_cave", "Lv.321-339 源石晶洞", "晶龙潜伏。", &[("矿区外围", "mining_outer"), ("源石王座", "source_throne")], &["mob_mutant_crystal_dragon"], false));
    ancient_rooms.insert("source_throne".into(), room("source_throne", "Lv.340 源石王座", "源石傀儡王守座。", &[("源石晶洞", "source_crystal_cave"), ("吞天蛇沼", "devouring_snake_marsh")], &["boss_source_golem_king"], false));
    ancient_rooms.insert("miasma_forest".into(), room("miasma_forest", "Lv.341-360 瘴气毒林", "洪荒大泽入口。", &[("太初远征营地", "taichu_camp"), ("吞天蛇沼", "devouring_snake_marsh")], &["mob_poison_dragon"], false));
    ancient_rooms.insert("devouring_snake_marsh".into(), room("devouring_snake_marsh", "Lv.361-379 吞天蛇沼", "巨蟒潜伏深沼。", &[("瘴气毒林", "miasma_forest"), ("源石王座", "source_throne"), ("巨猿领地", "giant_ape_domain")], &["mob_devouring_python"], false));
    ancient_rooms.insert("giant_ape_domain".into(), room("giant_ape_domain", "Lv.380 巨猿领地", "洪荒巨猿撼木。", &[("吞天蛇沼", "devouring_snake_marsh"), ("破败山门", "ruined_gate")], &["boss_primordial_giant_ape"], false));
    ancient_rooms.insert("ruined_gate".into(), room("ruined_gate", "Lv.381-400 破败山门", "造化仙门遗址入口。", &[("太初远征营地", "taichu_camp"), ("巨猿领地", "giant_ape_domain"), ("仙门天阶", "immortal_steps")], &["mob_fallen_heaven_soldier"], false));
    ancient_rooms.insert("immortal_steps".into(), room("immortal_steps", "Lv.401-419 仙门天阶", "仙门剑阵阻路。", &[("破败山门", "ruined_gate"), ("凌霄残殿", "lingxiao_ruins"), ("灰烬平原", "ash_plain")], &["mob_immortal_sword_array"], false));
    ancient_rooms.insert("lingxiao_ruins".into(), room("lingxiao_ruins", "Lv.420 凌霄残殿", "堕落谪仙守殿。", &[("仙门天阶", "immortal_steps"), ("浴火神巢", "fire_nest")], &["boss_fallen_exiled_immortal"], false));
    ancient_rooms.insert("ash_plain".into(), room("ash_plain", "Lv.421-440 灰烬平原", "火灵在灰烬中徘徊。", &[("仙门天阶", "immortal_steps"), ("浴火神巢", "fire_nest")], &["mob_ash_fire_spirit"], false));
    ancient_rooms.insert("fire_nest".into(), room("fire_nest", "Lv.441-459 浴火神巢", "焚天神雀盘旋。", &[("灰烬平原", "ash_plain"), ("凌霄残殿", "lingxiao_ruins"), ("涅槃核心", "nirvana_core")], &["mob_burning_sky_sparrow"], false));
    ancient_rooms.insert("nirvana_core".into(), room("nirvana_core", "Lv.460 涅槃核心", "神凰分身燃烧。", &[("浴火神巢", "fire_nest"), ("碎星带", "broken_stars")], &["boss_phoenix_avatar"], false));
    ancient_rooms.insert("broken_stars".into(), room("broken_stars", "Lv.461-480 碎星带", "星屑如刃。", &[("涅槃核心", "nirvana_core"), ("巨兽陨石", "beast_meteor")], &["mob_star_behemoth"], false));
    ancient_rooms.insert("beast_meteor".into(), room("beast_meteor", "Lv.481-498 巨兽陨石", "黑洞异魔潜伏。", &[("碎星带", "broken_stars"), ("天道祭坛", "heavenly_dao_altar")], &["mob_void_devourer"], false));
    ancient_rooms.insert("heavenly_dao_altar".into(), room("heavenly_dao_altar", "Lv.499-500 天道祭坛", "鸿蒙天道幻影镇守祭坛。", &[("巨兽陨石", "beast_meteor"), ("星际观测台", "stargazer_observatory")], &["boss_heavenly_dao_phantom"], false));
    ancient_rooms.insert("stargazer_observatory".into(), room("stargazer_observatory", "星际观测台", "无怪物观测台。", &[("天道祭坛", "heavenly_dao_altar")], &[], false));
    zones.insert("ancient_secret".into(), Zone { id: "ancient_secret".into(), name: "终极探索区域".into(), rooms: ancient_rooms });

    World { zones }
}
