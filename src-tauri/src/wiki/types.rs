use serde::{Deserialize, Serialize};
use specta::Type;

use super::dto::{WikiItemDto, WikiVehicleDto};

/// Flattened Wiki entity specs for frontend consumption.
/// Covers both items and vehicles — unused fields are None.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WikiEntitySpecs {
    // Identity
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub wiki_url: Option<String>,
    pub description: Option<String>,
    pub classification: Option<String>,
    pub class_name: Option<String>,
    pub item_class: Option<String>,
    pub grade: Option<String>,
    pub manufacturer_name: Option<String>,
    pub manufacturer_code: Option<String>,
    pub game_version: Option<String>,
    pub item_type: Option<String>,
    pub sub_type: Option<String>,
    pub size: Option<f64>,

    // Durability
    pub health: Option<f64>,
    pub resist_physical: Option<f64>,
    pub resist_energy: Option<f64>,
    pub resist_thermal: Option<f64>,
    pub resist_distortion: Option<f64>,
    pub resist_biochemical: Option<f64>,
    pub resist_stun: Option<f64>,

    // Distortion
    pub distortion_max: Option<f64>,
    pub distortion_decay_rate: Option<f64>,
    pub distortion_shutdown_time: Option<f64>,

    // Thermal (items)
    pub max_temp: Option<f64>,
    pub overheat_temp: Option<f64>,
    pub cooling_rate_max: Option<f64>,
    pub misfire_min_temp: Option<f64>,
    pub misfire_max_temp: Option<f64>,
    pub ir_emission: Option<f64>,

    // Power (items)
    pub power_draw: Option<f64>,
    pub power_to_em: Option<f64>,
    pub em_min: Option<f64>,
    pub em_max: Option<f64>,
    pub em_decay: Option<f64>,

    // Power Plant specific
    pub power_output: Option<f64>,

    // Weapon specific
    pub weapon_class: Option<String>,
    pub weapon_type: Option<String>,
    pub damage_per_shot: Option<f64>,
    pub dps_physical: Option<f64>,
    pub dps_energy: Option<f64>,
    pub dps_distortion: Option<f64>,
    pub dps_thermal: Option<f64>,
    pub weapon_range: Option<f64>,
    pub weapon_rpm: Option<f64>,
    pub ammo_speed: Option<f64>,
    pub ammo_range: Option<f64>,
    pub fire_modes: Vec<WikiFireMode>,

    // Vehicle speed
    pub scm_speed: Option<f64>,
    pub max_speed: Option<f64>,
    pub boost_forward: Option<f64>,
    pub boost_backward: Option<f64>,
    pub zero_to_scm: Option<f64>,
    pub zero_to_max: Option<f64>,

    // Vehicle agility
    pub pitch: Option<f64>,
    pub yaw: Option<f64>,
    pub roll: Option<f64>,
    pub pitch_boosted: Option<f64>,
    pub yaw_boosted: Option<f64>,
    pub roll_boosted: Option<f64>,
    pub accel_main: Option<f64>,
    pub accel_retro: Option<f64>,

    // Vehicle quantum
    pub quantum_speed: Option<f64>,
    pub quantum_fuel_capacity: Option<f64>,
    pub quantum_range: Option<f64>,
    pub quantum_spool_time: Option<f64>,

    // Vehicle shield
    pub shield_hp: Option<f64>,
    pub shield_regen: Option<f64>,
    pub shield_face_type: Option<String>,

    // Vehicle armor
    pub armor_health: Option<f64>,
    pub armor_dmg_physical: Option<f64>,
    pub armor_dmg_energy: Option<f64>,
    pub armor_dmg_distortion: Option<f64>,
    pub armor_dmg_thermal: Option<f64>,

    // Vehicle misc
    pub cargo_capacity: Option<f64>,
    pub vehicle_inventory: Option<f64>,
    pub crew_min: Option<u32>,
    pub crew_max: Option<u32>,
    pub msrp: Option<f64>,
    pub pledge_url: Option<String>,
    pub insurance_claim_time: Option<f64>,
    pub insurance_expedite_time: Option<f64>,
    pub insurance_expedite_cost: Option<f64>,
    pub fuel_capacity: Option<f64>,

    // Dimensions
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub mass: Option<f64>,
}

/// Fire mode info for weapons (IPC-safe).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WikiFireMode {
    pub mode: Option<String>,
    pub fire_type: Option<String>,
    pub rounds_per_minute: Option<f64>,
    pub damage_per_second: Option<f64>,
}

// ── From impls ─────────────────────────────────────────────────────────────

impl From<WikiItemDto> for WikiEntitySpecs {
    fn from(dto: WikiItemDto) -> Self {
        let mut specs = Self {
            uuid: dto.uuid,
            name: dto.name,
            wiki_url: dto.web_url,
            description: dto.description.and_then(|d| d.en_en),
            classification: dto.classification,
            class_name: dto.class_name,
            item_class: dto.class,
            grade: dto.grade,
            item_type: dto.item_type,
            sub_type: dto.sub_type,
            game_version: dto.version,
            size: dto.size,
            mass: dto.mass,
            ..Default::default()
        };

        // Manufacturer
        if let Some(mfr) = dto.manufacturer {
            specs.manufacturer_name = mfr.name;
            specs.manufacturer_code = mfr.code;
        }

        // Dimension
        if let Some(dim) = dto.dimension {
            specs.width = dim.width;
            specs.height = dim.height;
            specs.length = dim.length;
        }

        // Power plant
        if let Some(pp) = dto.power_plant {
            specs.power_output = pp.power_output;
        }

        // Heat
        if let Some(h) = dto.heat {
            specs.max_temp = h.max_temperature;
            specs.overheat_temp = h.overheat_temperature;
            specs.cooling_rate_max = h.max_cooling_rate;
            specs.misfire_min_temp = h.misfire_min_temperature;
            specs.misfire_max_temp = h.misfire_max_temperature;
            specs.ir_emission = h.ir_emission;
        }

        // Power
        if let Some(p) = dto.power {
            specs.power_draw = p.power_draw;
            specs.power_to_em = p.power_to_em;
            specs.em_min = p.em_min;
            specs.em_max = p.em_max;
            specs.em_decay = p.decay_rate_em;
        }

        // Durability
        if let Some(d) = dto.durability {
            specs.health = d.health;
            if let Some(r) = d.resistance {
                specs.resist_physical = r.physical;
                specs.resist_energy = r.energy;
                specs.resist_thermal = r.thermal;
                specs.resist_distortion = r.distortion;
                specs.resist_biochemical = r.biochemical;
                specs.resist_stun = r.stun;
            }
        }

        // Distortion
        if let Some(d) = dto.distortion {
            specs.distortion_max = d.maximum;
            specs.distortion_decay_rate = d.decay_rate;
            specs.distortion_shutdown_time = d.shutdown_time;
        }

        // Emission
        if let Some(e) = dto.emission {
            if specs.ir_emission.is_none() {
                specs.ir_emission = e.ir;
            }
            if specs.em_min.is_none() {
                specs.em_min = e.em_min;
            }
            if specs.em_max.is_none() {
                specs.em_max = e.em_max;
            }
        }

        // Vehicle weapon
        if let Some(w) = dto.vehicle_weapon {
            specs.weapon_class = w.class;
            specs.weapon_type = w.weapon_type;
            specs.damage_per_shot = w.damage_per_shot;
            specs.weapon_range = w.range;
            specs.weapon_rpm = w.rpm;

            if let Some(dmg) = w.damage {
                if let Some(dps) = dmg.dps {
                    specs.dps_physical = dps.physical;
                    specs.dps_energy = dps.energy;
                    specs.dps_distortion = dps.distortion;
                    specs.dps_thermal = dps.thermal;
                }
            }

            if let Some(ammo) = w.ammunition {
                specs.ammo_speed = ammo.speed;
                specs.ammo_range = ammo.range;
            }

            if let Some(modes) = w.modes {
                specs.fire_modes = modes
                    .into_iter()
                    .map(|m| WikiFireMode {
                        mode: m.mode,
                        fire_type: m.fire_type,
                        rounds_per_minute: m.rounds_per_minute,
                        damage_per_second: m.damage_per_second,
                    })
                    .collect();
            }
        }

        // Standalone ammunition (non-weapon items like missiles)
        if let Some(ammo) = dto.ammunition {
            if specs.ammo_speed.is_none() {
                specs.ammo_speed = ammo.speed;
            }
            if specs.ammo_range.is_none() {
                specs.ammo_range = ammo.range;
            }
        }

        specs
    }
}

impl From<WikiVehicleDto> for WikiEntitySpecs {
    fn from(dto: WikiVehicleDto) -> Self {
        let mut specs = Self {
            uuid: dto.uuid,
            name: dto.name,
            wiki_url: dto.web_url,
            description: dto.description.and_then(|d| d.get("en_EN").cloned()),
            classification: dto.vehicle_type,
            class_name: dto.class_name,
            item_type: dto.size,
            game_version: dto.version,
            cargo_capacity: dto.cargo_capacity,
            vehicle_inventory: dto.vehicle_inventory,
            mass: dto.mass,
            msrp: dto.msrp,
            pledge_url: dto.pledge_url,
            health: dto.health,
            shield_hp: dto.shield_hp,
            ..Default::default()
        };

        // Manufacturer
        if let Some(mfr) = dto.manufacturer {
            specs.manufacturer_name = mfr.name;
            specs.manufacturer_code = mfr.code;
        }

        // Dimension
        if let Some(dim) = dto.dimension {
            specs.width = dim.width;
            specs.height = dim.height;
            specs.length = dim.length;
        }

        // Speed
        if let Some(s) = dto.speed {
            specs.scm_speed = s.scm;
            specs.max_speed = s.max;
            specs.boost_forward = s.boost_forward;
            specs.boost_backward = s.boost_backward;
            specs.zero_to_scm = s.zero_to_scm;
            specs.zero_to_max = s.zero_to_max;
        }

        // Agility
        if let Some(a) = dto.agility {
            specs.pitch = a.pitch;
            specs.yaw = a.yaw;
            specs.roll = a.roll;
            specs.pitch_boosted = a.pitch_boosted;
            specs.yaw_boosted = a.yaw_boosted;
            specs.roll_boosted = a.roll_boosted;
            if let Some(acc) = a.acceleration {
                specs.accel_main = acc.main;
                specs.accel_retro = acc.retro;
            }
        }

        // Quantum
        if let Some(q) = dto.quantum {
            specs.quantum_speed = q.quantum_speed;
            specs.quantum_fuel_capacity = q.quantum_fuel_capacity;
            specs.quantum_range = q.quantum_range;
            specs.quantum_spool_time = q.quantum_spool_time;
        }

        // Fuel
        if let Some(f) = dto.fuel {
            specs.fuel_capacity = f.capacity;
        }

        // Shield
        if let Some(s) = dto.shield {
            if specs.shield_hp.is_none() {
                specs.shield_hp = s.hp;
            }
            specs.shield_regen = s.regeneration;
            specs.shield_face_type = s.face_type;
        }

        // Armor
        if let Some(a) = dto.armor {
            specs.armor_health = a.health;
            if let Some(dm) = a.damage_multipliers {
                specs.armor_dmg_physical = dm.physical;
                specs.armor_dmg_energy = dm.energy;
                specs.armor_dmg_distortion = dm.distortion;
                specs.armor_dmg_thermal = dm.thermal;
            }
        }

        // Insurance
        if let Some(i) = dto.insurance {
            specs.insurance_claim_time = i.claim_time;
            specs.insurance_expedite_time = i.expedite_time;
            specs.insurance_expedite_cost = i.expedite_cost;
        }

        // Crew
        if let Some(c) = dto.crew {
            specs.crew_min = c.min;
            specs.crew_max = c.max;
        }

        specs
    }
}
