-- 0.2.10: Wanxiang unlock gate, profession skill rebuild, and meditation relocation.

insert into world_rooms (zone_id, id, name, description, exits, spawns, safe) values
  (
    'xiuzhen',
    'purgatory',
    '炼狱',
    '破冰前哨站下方的苦修火窟，可打坐修炼等级经验，不产出装备、金币或材料。',
    '{"破冰前哨站":"ice_outpost"}',
    '[]',
    true
  ),
  (
    'feisheng',
    'void_realm',
    '虚境',
    '混沌庇护所内的心识秘境，可指定已学习技能打坐研修，只增长技能经验。',
    '{"混沌庇护所":"chaos_shelter"}',
    '[]',
    true
  )
on conflict (zone_id, id) do update set
  name = excluded.name,
  description = excluded.description,
  exits = excluded.exits,
  spawns = excluded.spawns,
  safe = excluded.safe;

update world_rooms
set exits = exits || '{"炼狱":"purgatory"}'::jsonb
where zone_id = 'xiuzhen' and id = 'ice_outpost';

update world_rooms
set exits = exits || '{"虚境":"void_realm"}'::jsonb
where zone_id = 'feisheng' and id = 'chaos_shelter';

delete from character_skills cs
using skills s
where cs.skill_id = s.id
  and s.class in ('warrior', 'mage', 'taoist', 'assassin', 'all')
  and s.id not in ('talent_battle_instinct', 'talent_immovable_king');

delete from skills
where class in ('warrior', 'mage', 'taoist', 'assassin', 'all')
  and id not in ('talent_battle_instinct', 'talent_immovable_king');

insert into skills (id, name, class, min_level, mp_cost, cooldown_ms, config, max_level) values
  (
    'skill_sword_qingfeng',
    '清风剑诀',
    'warrior',
    10,
    8,
    1100,
    '{"kind":"physical","type":"剑修主动","desc":"入门剑诀，将一缕真元附着于剑身，剑出如风，延绵不绝。","effect":"单体物理伤害，附带微量命中提升。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":100000,"power":1.15,"auto_priority":20,"guaranteed_hit_pct_per_100":6}'::jsonb,
    100
  ),
  (
    'skill_sword_armor_break',
    '破甲剑气',
    'warrior',
    20,
    0,
    0,
    '{"kind":"passive","type":"剑修被动","desc":"剑意初成，出招时自带锐利罡气，可轻易切开妖兽坚甲。","effect":"提升物理穿透，满级最高不超过目标防御的30%。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":150000,"ignore_def_pct_per_100":30}'::jsonb,
    100
  ),
  (
    'skill_sword_sky_draw',
    '斩天拔剑术',
    'warrior',
    30,
    18,
    1800,
    '{"kind":"physical","type":"剑修主动","desc":"养剑气于丹田，拔剑瞬间爆发出斩断天际的锋芒。","effect":"单次高倍率物理伤害，该次攻击暴击率额外提升20%。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":250000,"power":1.75,"auto_priority":45,"active_crit_pct":20}'::jsonb,
    100
  ),
  (
    'skill_sword_pure_yang',
    '纯阳剑体',
    'warrior',
    40,
    0,
    0,
    '{"kind":"passive","type":"剑修被动","desc":"引九天纯阳之气淬炼五脏六腑，肉身堪比法宝。","effect":"按百分比提升自身生命上限与物理攻击力。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":400000,"hp_pct_per_100":18,"atk_pct_per_100":15,"hp_bonus":40,"atk_bonus":4}'::jsonb,
    100
  ),
  (
    'skill_sword_shadow_dance',
    '幻影剑舞',
    'warrior',
    60,
    28,
    2200,
    '{"kind":"physical","type":"剑修主动","desc":"将身法催动至极限，化作数道残影对同一目标进行疯狂切割。","effect":"对单体目标造成3-4段连续物理伤害，当前战斗折算为高倍率连击。","source":"虚空要塞·虚空市集购买","acquire":"void_market","requires_book":false,"gold_cost":750000,"power":2.25,"auto_priority":65,"hit_count_min":3,"hit_count_max":4}'::jsonb,
    100
  ),
  (
    'skill_sword_blood_intent',
    '嗜血剑意',
    'warrior',
    100,
    0,
    0,
    '{"kind":"passive","type":"剑修被动","desc":"杀戮证道，以敌之精血反哺己身。","effect":"攻击附带吸血效果，随技能等级从3%成长到10%。","source":"虚空要塞·虚空市集购买","acquire":"void_market","requires_book":false,"gold_cost":1200000,"life_steal_base_pct":3,"life_steal_pct_per_100":7}'::jsonb,
    100
  ),
  (
    'skill_sword_wanfa',
    '一剑破万法',
    'warrior',
    120,
    0,
    0,
    '{"kind":"passive","type":"剑修被动","desc":"剑修之极，万法不侵，能以绝强剑气斩断敌人的术法反噬。","effect":"提升魔法防御与少量最终伤害减免。","source":"塔顶封印·怨魂聚合体掉落","requires_book":true,"book_id":"book_sword_wanfa","drop_rate":"medium","mdef_bonus":60,"damage_reduce_pct_per_100":8}'::jsonb,
    100
  ),
  (
    'skill_sword_dayan',
    '大衍神剑',
    'warrior',
    150,
    42,
    2800,
    '{"kind":"physical","type":"剑修主动","desc":"凝聚大衍之数，锁定目标神魂，使其避无可避。","effect":"极高命中率物理攻击，附带目标当前生命值5%的物理伤害，BOSS有上限。","source":"塔顶封印·怨魂聚合体掉落","requires_book":true,"book_id":"book_sword_dayan","drop_rate":"medium","power":2.55,"auto_priority":82,"guaranteed_hit_pct_per_100":20,"target_current_hp_pct":5,"boss_extra_damage_cap_atk":5}'::jsonb,
    100
  ),
  (
    'skill_sword_clear_heart',
    '剑心通明',
    'warrior',
    180,
    0,
    0,
    '{"kind":"passive","type":"剑修被动","desc":"心若冰清，天塌不惊，攻击时能精准寻找敌人的致命弱点。","effect":"提升基础暴击率与暴击伤害倍率。","source":"判官殿·阎罗判官掉落","requires_book":true,"book_id":"book_sword_clear_heart","drop_rate":"low","crit_pct_bonus":1,"crit_damage_pct_bonus":1}'::jsonb,
    100
  ),
  (
    'skill_sword_zhuxian',
    '诛仙灭世斩',
    'warrior',
    200,
    90,
    5200,
    '{"kind":"physical","type":"剑修终极技","desc":"献祭海量真元，凝聚上古诛仙虚影，斩出足以抹杀界域的灭世一剑。","effect":"超高倍率物理伤害，目标血量低于15%时伤害倍率额外提升50%。","source":"判官殿·阎罗判官掉落","requires_book":true,"book_id":"book_sword_zhuxian","drop_rate":"low","power":3.35,"auto_priority":110,"execute_threshold_pct":15,"execute_bonus_pct":50}'::jsonb,
    100
  ),

  (
    'skill_spell_zixiao',
    '紫霄引雷诀',
    'mage',
    10,
    12,
    1200,
    '{"kind":"magical","type":"法修主动","desc":"口颂真言，引九天之上的一道紫霄神雷劈击敌人天灵盖。","effect":"单体魔法伤害。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":100000,"power":1.25,"auto_priority":20}'::jsonb,
    100
  ),
  (
    'skill_spell_gather_spirit',
    '聚灵秘法',
    'mage',
    20,
    0,
    0,
    '{"kind":"passive","type":"法修被动","desc":"强行掠夺天地灵气扩充自身丹田，以支撑庞大的术法消耗。","effect":"提升最大真元上限及战斗续航。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":150000,"mp_pct_per_100":25,"mp_bonus":60,"mana_steal_base_pct":1,"mana_steal_pct_per_100":4}'::jsonb,
    100
  ),
  (
    'skill_spell_samadhi',
    '三昧真火',
    'mage',
    30,
    26,
    1700,
    '{"kind":"magical","type":"法修主动","desc":"喷吐出能焚尽万物的三昧真火，温度极高，无物不融。","effect":"单体魔法伤害，附带3回合灼烧，每回合造成施法者魔法攻击力20%的伤害。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":250000,"power":1.55,"auto_priority":42,"burn_turns":3,"burn_mag_pct":20}'::jsonb,
    100
  ),
  (
    'skill_spell_glass_body',
    '八卦琉璃身',
    'mage',
    40,
    0,
    0,
    '{"kind":"passive","type":"法修被动","desc":"真元在体表自动流转，化作八卦琉璃之盾抵御外力。","effect":"提升真元、生命、魔法防御与少量最终减伤。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":400000,"mp_pct_per_100":15,"hp_bonus":45,"mdef_bonus":8,"damage_reduce_pct_per_100":6}'::jsonb,
    100
  ),
  (
    'skill_spell_ice_domain',
    '绝对冰域',
    'mage',
    60,
    38,
    2300,
    '{"kind":"magical","type":"法修主动","desc":"瞬间抽干周围空间的温度，冻结敌人的气血运行。","effect":"单体魔法伤害，降低目标下一次攻击的命中率和速度。","source":"虚空要塞·虚空市集购买","acquire":"void_market","requires_book":false,"gold_cost":750000,"power":1.85,"auto_priority":64,"slow_next_attack_pct":20}'::jsonb,
    100
  ),
  (
    'skill_spell_five_roots',
    '五行灵根',
    'mage',
    100,
    0,
    0,
    '{"kind":"passive","type":"法修被动","desc":"洗筋伐髓，成就完美五行灵根，对天地元素的亲和力达到极致。","effect":"按固定百分比提升总魔法攻击力。","source":"虚空要塞·虚空市集购买","acquire":"void_market","requires_book":false,"gold_cost":1200000,"mag_pct_per_100":28,"mag_bonus":8}'::jsonb,
    100
  ),
  (
    'skill_spell_xuan_light',
    '寂灭玄光',
    'mage',
    120,
    52,
    2900,
    '{"kind":"magical","type":"法修主动","desc":"将五行之力极度压缩于指尖，射出一道贯穿虚空的毁灭玄光。","effect":"无视目标部分魔法防御的穿透魔法伤害。","source":"塔顶封印·怨魂聚合体掉落","requires_book":true,"book_id":"book_spell_xuan_light","drop_rate":"medium","power":2.35,"auto_priority":84,"ignore_mdef_pct":25}'::jsonb,
    100
  ),
  (
    'skill_spell_resonance',
    '法神共鸣',
    'mage',
    150,
    0,
    0,
    '{"kind":"passive","type":"法修被动","desc":"每一次施法，都能与天地大道产生共鸣，越战越勇。","effect":"施放主动技能时有小概率不消耗真元再触发一次同技能。当前战斗折算为技能伤害加深。","source":"塔顶封印·怨魂聚合体掉落","requires_book":true,"book_id":"book_spell_resonance","drop_rate":"medium","damage_deepen_pct_per_100":10,"free_recast_base_pct":5,"free_recast_pct_per_100":5}'::jsonb,
    100
  ),
  (
    'skill_spell_chaos_orb',
    '混沌法球',
    'mage',
    180,
    72,
    4200,
    '{"kind":"magical","type":"法修主动","desc":"抽取自身海量真元凝聚成一颗混沌黑洞，能吞噬一切生机。","effect":"消耗当前剩余真元10%，按比例转化为额外魔法伤害。","source":"判官殿·阎罗判官掉落","requires_book":true,"book_id":"book_spell_chaos_orb","drop_rate":"low","power":3.05,"auto_priority":100,"current_mp_cost_pct":10,"mp_to_damage_pct":60}'::jsonb,
    100
  ),
  (
    'skill_spell_thunder_array',
    '九天雷劫大阵',
    'mage',
    200,
    120,
    5600,
    '{"kind":"magical","type":"法修终极技","desc":"引动天道雷劫，代替天罚抹杀眼前的逆天之物。","effect":"消耗巨额真元，对目标造成连续5-9段递增魔法伤害，是总倍率最高的单体爆发技能。","source":"判官殿·阎罗判官掉落","requires_book":true,"book_id":"book_spell_thunder_array","drop_rate":"low","power":4.6,"auto_priority":115,"hit_count_min":5,"hit_count_max":9}'::jsonb,
    100
  ),

  (
    'skill_soul_shehun',
    '太上摄魂咒',
    'taoist',
    10,
    10,
    1200,
    '{"kind":"magical","type":"魂修主动","desc":"凌空画符，将幽冥之气凝聚成灵符轰击敌人的神识。","effect":"单体道术混合伤害。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":100000,"power":1.18,"auto_priority":20}'::jsonb,
    100
  ),
  (
    'skill_soul_bone_poison',
    '九幽蚀骨毒',
    'taoist',
    20,
    18,
    1600,
    '{"kind":"magical","type":"魂修主动","desc":"提取黄泉瘴气炼制而成的无色无味之毒，直接腐蚀敌人的根基。","effect":"单体施毒，目标接下来5回合内每回合受到道术攻击力倍率真实伤害，并降低回血。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":150000,"power":1.3,"auto_priority":40,"poison_turns":5,"poison_mag_multiplier":0.45,"regen_reduce_pct":20}'::jsonb,
    100
  ),
  (
    'skill_soul_gangqi',
    '先天罡气',
    'taoist',
    30,
    0,
    0,
    '{"kind":"passive","type":"魂修被动","desc":"阴阳交汇，在体内凝结出先天罡气，固本培元。","effect":"提升基础物理防御与魔法防御。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":250000,"def_bonus":8,"mdef_bonus":8}'::jsonb,
    100
  ),
  (
    'skill_soul_rejuvenation',
    '枯木逢春诀',
    'taoist',
    40,
    0,
    0,
    '{"kind":"passive","type":"魂修被动","desc":"逆转生死，夺天地造化之生机，战斗中可持续修复受损肉身。","effect":"行动后回复自身最大生命值2%加道术攻击力折算的气血。当前战斗折算为生命与吸血成长。","source":"天水古城·天水书院购买","acquire":"academy","requires_book":false,"gold_cost":400000,"hp_bonus":55,"life_steal_base_pct":2,"life_steal_pct_per_100":4}'::jsonb,
    100
  ),
  (
    'skill_soul_possession',
    '唤灵·附体',
    'taoist',
    60,
    36,
    2600,
    '{"kind":"magical","type":"魂修主动","desc":"通过契约将远古吞天兽的真灵引入体内，短暂进入半妖化形态。","effect":"消耗真元进入附体状态，期间攻击力与双防提升20%。当前战斗折算为道术伤害。","source":"虚空要塞·虚空市集购买","acquire":"void_market","requires_book":false,"gold_cost":750000,"power":1.75,"auto_priority":58,"buff_turns":4,"buff_atk_def_pct":20}'::jsonb,
    100
  ),
  (
    'skill_soul_gu_revenge',
    '蛊毒反噬',
    'taoist',
    100,
    0,
    0,
    '{"kind":"passive","type":"魂修被动","desc":"以自身精血饲养本命蛊，任何触碰你的人都会沾染恶毒诅咒。","effect":"受击时有概率使目标虚弱，使其下一次攻击伤害降低15%。当前战斗折算为防御与减伤。","source":"虚空要塞·虚空市集购买","acquire":"void_market","requires_book":false,"gold_cost":1200000,"def_bonus":6,"mdef_bonus":6,"damage_reduce_pct_per_100":8,"weakness_next_damage_reduce_pct":15}'::jsonb,
    100
  ),
  (
    'skill_soul_karma_fire',
    '幽冥业火',
    'taoist',
    120,
    45,
    3000,
    '{"kind":"magical","type":"魂修主动","desc":"召唤来自九幽地狱的业火，不烧肉身，专烧灵魂。","effect":"单体道术伤害。若目标存在蚀骨毒，则额外结算1回合毒素伤害但不清除毒素。","source":"塔顶封印·怨魂聚合体掉落","requires_book":true,"book_id":"book_soul_karma_fire","drop_rate":"medium","power":2.15,"auto_priority":80,"poison_bonus_tick":1}'::jsonb,
    100
  ),
  (
    'skill_soul_reincarnation',
    '轮回渡厄',
    'taoist',
    150,
    0,
    0,
    '{"kind":"passive","type":"魂修被动","desc":"参悟生死轮回之理，阎王叫你三更死，你可强留到五更。","effect":"提升生命上限。每场战斗一次，受到致命伤害时保留1点生命并获得微量护盾。当前战斗折算为生命与减伤。","source":"塔顶封印·怨魂聚合体掉落","requires_book":true,"book_id":"book_soul_reincarnation","drop_rate":"medium","hp_pct_per_100":20,"damage_reduce_pct_per_100":6,"lethal_guard_once":true}'::jsonb,
    100
  ),
  (
    'skill_soul_all_unity',
    '万法归一',
    'taoist',
    180,
    0,
    0,
    '{"kind":"passive","type":"魂修被动","desc":"魂修大成，将内功、外功、神识三者合一，打破属性壁垒。","effect":"将自身物理与魔法防御总和的5%-10%转化为额外道术攻击力。","source":"判官殿·阎罗判官掉落","requires_book":true,"book_id":"book_soul_all_unity","drop_rate":"low","mag_bonus":14,"def_to_mag_base_pct":5,"def_to_mag_pct_per_100":5}'::jsonb,
    100
  ),
  (
    'skill_soul_yanluo_prison',
    '阎罗镇狱神诀',
    'taoist',
    200,
    95,
    5200,
    '{"kind":"magical","type":"魂修终极技","desc":"化身十殿阎罗，宣判眼前的生灵阳寿已尽，强行打入无间地狱。","effect":"高额单体道术伤害，并附加破绽，使目标3回合内受到的所有伤害额外增加10%。","source":"判官殿·阎罗判官掉落","requires_book":true,"book_id":"book_soul_yanluo_prison","drop_rate":"low","power":3.05,"auto_priority":108,"vulnerability_turns":3,"vulnerability_pct":10}'::jsonb,
    100
  )
on conflict (id) do update set
  name = excluded.name,
  class = excluded.class,
  min_level = excluded.min_level,
  mp_cost = excluded.mp_cost,
  cooldown_ms = excluded.cooldown_ms,
  config = excluded.config,
  max_level = excluded.max_level;

insert into item_templates (id, name, kind, slot, rarity, price, stackable, stats, flags) values
  ('book_sword_wanfa', '一剑破万法技能书', 'book', null, 'epic', 6000, false, '{"skill":"skill_sword_wanfa"}', '{"source":"boss_wraith_aggregate"}'),
  ('book_sword_dayan', '大衍神剑技能书', 'book', null, 'epic', 6800, false, '{"skill":"skill_sword_dayan"}', '{"source":"boss_wraith_aggregate"}'),
  ('book_sword_clear_heart', '剑心通明技能书', 'book', null, 'legendary', 12000, false, '{"skill":"skill_sword_clear_heart"}', '{"source":"boss_yanluo_judge"}'),
  ('book_sword_zhuxian', '诛仙灭世斩技能书', 'book', null, 'legendary', 15000, false, '{"skill":"skill_sword_zhuxian"}', '{"source":"boss_yanluo_judge"}'),
  ('book_spell_xuan_light', '寂灭玄光技能书', 'book', null, 'epic', 6000, false, '{"skill":"skill_spell_xuan_light"}', '{"source":"boss_wraith_aggregate"}'),
  ('book_spell_resonance', '法神共鸣技能书', 'book', null, 'epic', 6800, false, '{"skill":"skill_spell_resonance"}', '{"source":"boss_wraith_aggregate"}'),
  ('book_spell_chaos_orb', '混沌法球技能书', 'book', null, 'legendary', 12000, false, '{"skill":"skill_spell_chaos_orb"}', '{"source":"boss_yanluo_judge"}'),
  ('book_spell_thunder_array', '九天雷劫大阵技能书', 'book', null, 'legendary', 15000, false, '{"skill":"skill_spell_thunder_array"}', '{"source":"boss_yanluo_judge"}'),
  ('book_soul_karma_fire', '幽冥业火技能书', 'book', null, 'epic', 6000, false, '{"skill":"skill_soul_karma_fire"}', '{"source":"boss_wraith_aggregate"}'),
  ('book_soul_reincarnation', '轮回渡厄技能书', 'book', null, 'epic', 6800, false, '{"skill":"skill_soul_reincarnation"}', '{"source":"boss_wraith_aggregate"}'),
  ('book_soul_all_unity', '万法归一技能书', 'book', null, 'legendary', 12000, false, '{"skill":"skill_soul_all_unity"}', '{"source":"boss_yanluo_judge"}'),
  ('book_soul_yanluo_prison', '阎罗镇狱神诀技能书', 'book', null, 'legendary', 15000, false, '{"skill":"skill_soul_yanluo_prison"}', '{"source":"boss_yanluo_judge"}')
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
set config = config || '{
  "desc":"特殊被动。只能在混沌庇护所寻找不动冥王，消耗技能书残页逐级提升。",
  "source":"混沌庇护所·不动冥王",
  "acquire":"chaos_master",
  "requires_book":false,
  "special_upgrade_only":true
}'::jsonb
where id in ('talent_battle_instinct', 'talent_immovable_king');
