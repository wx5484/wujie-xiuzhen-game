use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub id: i64,
    pub name: String,
    pub level: i32,
    pub hp: i64,
    pub max_hp: i64,
    pub max_mp: i64,
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
    pub dex: i64,
    pub crit_pct: i64,
    pub luck: i64,
    pub heavy_hit_pct: i64,
    pub life_steal_pct: i64,
    pub mana_steal_pct: i64,
    pub paralyze_pct: i64,
    pub petrify_pct: i64,
    pub paralyze_resist_pct: i64,
    pub petrify_resist_pct: i64,
    pub crit_damage_pct: i64,
    pub boss_damage_pct: i64,
    pub damage_deepen_pct: i64,
    pub normal_mob_execute_pct: i64,
    pub damage_reduce_pct: i64,
    pub ignore_def_pct: i64,
    pub guaranteed_hit_pct: i64,
    pub target_max_hp_true_damage_pct: i64,
    pub self_max_mp_true_damage_pct: i64,
    pub creation_strike_pct: i64,
    pub creation_strike_damage_pct: i64,
    pub creation_strike_full_restore: bool,
    pub control_immune: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageReport {
    pub hit: bool,
    pub crit: bool,
    pub heavy: bool,
    pub damage: i64,
    pub remaining_hp: i64,
}

pub fn hit_chance(attacker: &Combatant, defender: &Combatant) -> f64 {
    if attacker.guaranteed_hit_pct >= 100 {
        return 1.0;
    }
    let level_delta = attacker.level - defender.level;
    let luck_bonus = attacker.luck as f64 * 0.0005;
    let hit_bonus = attacker.guaranteed_hit_pct.clamp(0, 99) as f64 / 100.0;
    let base = 0.78 + level_delta as f64 * 0.01 + luck_bonus + hit_bonus;
    base.clamp(0.2, 0.95)
}

pub fn physical_damage<R: Rng>(
    rng: &mut R,
    attacker: &Combatant,
    defender: &Combatant,
    power: f64,
) -> DamageReport {
    if rng.gen::<f64>() > hit_chance(attacker, defender) {
        return DamageReport { hit: false, crit: false, heavy: false, damage: 0, remaining_hp: defender.hp };
    }
    let roll = rng.gen_range(70..=100) as f64 / 100.0;
    let crit_chance = (attacker.crit_pct + attacker.luck / 20).clamp(0, 95);
    let crit = rng.gen_range(0..100) < crit_chance;
    let heavy = rng.gen_range(0..100) < attacker.heavy_hit_pct.clamp(0, 80);
    let crit_bonus = (attacker.crit_damage_pct.clamp(0, 300) as f64) / 100.0;
    let crit_mult = if crit { 1.8 + crit_bonus } else { 1.0 };
    let heavy_mult = if heavy { 1.45 } else { 1.0 };
    let raw = ((attacker.atk as f64 * roll * power * crit_mult * heavy_mult) as i64).max(1);
    let defender_def = effective_defense(defender.def, attacker.ignore_def_pct);
    let mitigation = defender_def + rng.gen_range(0..=defender_def.max(0) / 2);
    let damage = reduce_damage((raw - mitigation).max(1), defender.damage_reduce_pct);
    DamageReport { hit: true, crit, heavy, damage, remaining_hp: (defender.hp - damage).max(0) }
}

pub fn magical_damage<R: Rng>(
    rng: &mut R,
    attacker: &Combatant,
    defender: &Combatant,
    power: f64,
) -> DamageReport {
    let raw = (attacker.mag as f64 * rng.gen_range(80..=115) as f64 / 100.0 * power) as i64;
    let defender_mdef = effective_defense(defender.mdef, attacker.ignore_def_pct);
    let damage = reduce_damage((raw - defender_mdef / 2).max(1), defender.damage_reduce_pct);
    DamageReport { hit: true, crit: false, heavy: false, damage, remaining_hp: (defender.hp - damage).max(0) }
}

pub fn roll_control<R: Rng>(rng: &mut R, attacker: &Combatant, defender: &Combatant) -> Option<&'static str> {
    if defender.control_immune {
        return None;
    }
    let paralyze = (attacker.paralyze_pct - defender.paralyze_resist_pct).max(0);
    let petrify = (attacker.petrify_pct - defender.petrify_resist_pct).max(0);
    let chance = (paralyze + petrify).clamp(0, 80);
    if chance > 0 && rng.gen_range(0..100) < chance {
        Some(if petrify > paralyze { "石化" } else { "麻痹" })
    } else {
        None
    }
}

fn effective_defense(value: i64, ignore_pct: i64) -> i64 {
    value.max(0).saturating_mul(100 - ignore_pct.clamp(0, 95)) / 100
}

fn reduce_damage(value: i64, reduce_pct: i64) -> i64 {
    (value.max(1).saturating_mul(100 - reduce_pct.clamp(0, 90)) / 100).max(1)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn hit_chance_is_clamped() {
        let a = Combatant {
            id: 1,
            name: "a".into(),
            level: 1,
            hp: 100,
            max_hp: 100,
            max_mp: 0,
            atk: 10,
            def: 1,
            mag: 1,
            mdef: 1,
            dex: 1000,
            crit_pct: 0,
            luck: 0,
            heavy_hit_pct: 0,
            life_steal_pct: 0,
            mana_steal_pct: 0,
            paralyze_pct: 0,
            petrify_pct: 0,
            paralyze_resist_pct: 0,
            petrify_resist_pct: 0,
            crit_damage_pct: 0,
            boss_damage_pct: 0,
            damage_deepen_pct: 0,
            normal_mob_execute_pct: 0,
            damage_reduce_pct: 0,
            ignore_def_pct: 0,
            guaranteed_hit_pct: 0,
            target_max_hp_true_damage_pct: 0,
            self_max_mp_true_damage_pct: 0,
            creation_strike_pct: 0,
            creation_strike_damage_pct: 0,
            creation_strike_full_restore: false,
            control_immune: false,
        };
        let mut b = a.clone();
        b.level = 1;
        let mut a = a;
        a.level = 200;
        assert_eq!(hit_chance(&a, &b), 0.95);
    }

    #[test]
    fn damage_keeps_one_point_floor() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let a = Combatant {
            id: 1,
            name: "a".into(),
            level: 1,
            hp: 100,
            max_hp: 100,
            max_mp: 0,
            atk: 1,
            def: 1,
            mag: 1,
            mdef: 1,
            dex: 10,
            crit_pct: 0,
            luck: 0,
            heavy_hit_pct: 0,
            life_steal_pct: 0,
            mana_steal_pct: 0,
            paralyze_pct: 0,
            petrify_pct: 0,
            paralyze_resist_pct: 0,
            petrify_resist_pct: 0,
            crit_damage_pct: 0,
            boss_damage_pct: 0,
            damage_deepen_pct: 0,
            normal_mob_execute_pct: 0,
            damage_reduce_pct: 0,
            ignore_def_pct: 0,
            guaranteed_hit_pct: 0,
            target_max_hp_true_damage_pct: 0,
            self_max_mp_true_damage_pct: 0,
            creation_strike_pct: 0,
            creation_strike_damage_pct: 0,
            creation_strike_full_restore: false,
            control_immune: false,
        };
        let mut b = a.clone();
        b.def = 999;
        let report = physical_damage(&mut rng, &a, &b, 1.0);
        assert!(report.damage <= 1 || !report.hit);
    }
}
