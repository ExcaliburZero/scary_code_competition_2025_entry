#![feature(variant_count)]

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::{read, read_to_string};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::prelude::*;
use std::{io, mem};

use clap::{Parser, Subcommand};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

const RESPONSE_TEMPLATES: [&str; 9] = [
    "Hello, {}! Nice to meet you!",
    "Nice to meet you {}! You seem like a pleasant person!",
    "Hey {}! Did you see the Mets game last night? Was a real nail-biter!",
    "Greetings {}! It is a pleasure to make your acquaintance.",
    "Good morning {}! Isn't it a lovely day today?",
    "How's it going {}? Did you finish that project you were working on?",
    "Greetings fellow human, {}! Tis a great day to have skin and bones, is it not?",
    "Hey {}! Have you ever wondered about what Mario is thinking? Who knows why he crushes turtles? And why do we think about him as fondly as we think of the mythical (nonexistant?) Dr Pepper? Perchance.",
    "Creation is an inviolate act, and those searching for the divine need not descend to Hell for fuel. That's why I don't go to Denny's anymore, {}.",
];

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value_t = 0)]
    verbose: u32,
}

#[derive(Subcommand)]
enum Commands {
    Greet,
    Test { filepath: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Greet => {
            println!("Please enter your name:");
            print!("> ");
            io::stdout().flush().unwrap();
            let user_name = read_user_name();

            let mut rng = create_rng(&user_name);
            let mut game = Game::from_rng(&mut rng);

            let result = game.play(&mut rng, cli.verbose);
            match result {
                GameResult::Win => println!("{}", game.response_template.replace("{}", &user_name)),
                GameResult::Loss(num_dungeons_completed) => println!(
                    "{}",
                    game.response_template_parts
                        .iter()
                        .take(num_dungeons_completed as usize)
                        .map(|s| s.replace("{}", &user_name))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            }
        }
        Commands::Test { filepath } => {
            let names: Vec<String> = read_to_string(filepath)
                .unwrap()
                .lines()
                .map(String::from)
                .collect();

            println!("name,num_parts,progress,result,greeting");
            for name in names.iter() {
                let mut rng = create_rng(&name);
                let mut game = Game::from_rng(&mut rng);

                let result = game.play(&mut rng, cli.verbose);

                let (progress, result_str) = match result {
                    GameResult::Win => (game.response_template_parts.len(), "win"),
                    GameResult::Loss(progress) => (progress as usize, "loss"),
                };

                println!(
                    "{},{},{},{},\"{}\"",
                    name,
                    game.response_template_parts.len(),
                    progress,
                    result_str,
                    game.response_template_parts
                        .iter()
                        .take(progress as usize)
                        .map(|s| s.replace("{}", &name))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
        }
    }
}

fn read_user_name() -> String {
    let stdin = io::stdin();
    let mut user_name = "".to_string();
    stdin.lock().read_line(&mut user_name).unwrap();
    user_name.trim().to_string()
}

fn create_rng(user_name: &str) -> StdRng {
    let mut s = DefaultHasher::new();
    user_name.hash(&mut s);
    let seed = s.finish();

    StdRng::seed_from_u64(seed)
}

struct Game {
    player: Creature,
    dungeons: Vec<Dungeon>,
    response_template: String,
    response_template_parts: Vec<String>,
}

impl Game {
    fn from_rng(rng: &mut StdRng) -> Game {
        let player = Creature::create(
            CreatureType::Player,
            &StatPattern {
                hp: 1.5 + rng.random_range(-0.5..0.5),
                attack: 2.0 + rng.random_range(-0.5..0.5),
                defense: 1.5 + rng.random_range(-0.5..0.5),
                magic: 2.0 + rng.random_range(-0.5..0.5),
                wisdom: 1.5 + rng.random_range(-0.5..0.5),
                speed: 1.5 + rng.random_range(-0.5..0.5),
            },
            1,
            false,
            rng,
        );

        let response_template = RESPONSE_TEMPLATES
            [(rng.random::<u64>() % RESPONSE_TEMPLATES.len() as u64) as usize]
            .to_string();
        let response_template_parts: Vec<String> = response_template
            .split(" ")
            .map(|s| s.to_string())
            .collect();

        let dungeons: Vec<Dungeon> = (0..response_template_parts.len())
            .into_iter()
            .map(|level| Dungeon::from_hash(rng.random(), (level + 1) as u32))
            .collect();

        Game {
            player,
            dungeons,
            response_template,
            response_template_parts,
        }
    }

    fn play(&mut self, rng: &mut StdRng, log_level: u32) -> GameResult {
        if log_level > 0 {
            println!("{:?}", self.player);
        }

        let mut i = 0;
        let mut max_dungeon_won = 0;
        let mut reentering = false;
        while i < self.dungeons.len() {
            let dungeon = &self.dungeons[i];
            self.player.current_hp = self.player.max_hp;

            if !reentering && log_level > 0 {
                println!("-------------------------");
                println!("{} enters {}", self.player.name, dungeon.get_name());
            }
            reentering = false;

            max_dungeon_won = max(i, max_dungeon_won);

            let mut j = 0;
            while j < 5 {
                let dungeon = &self.dungeons[i];
                let mut enemy = dungeon.create_enemy(rng, j == 4);
                //println!("{:?}", enemy);

                let fight_result = self.player.fight(&mut enemy, log_level);
                if let FightResult::Loss = fight_result {
                    if self.player.lives == 0 {
                        return GameResult::Loss(i as i32);
                    } else {
                        self.player.lives -= 1;
                        self.player.current_hp = self.player.max_hp;

                        if j == 0 && i > 0 {
                            // Go back to the last dungeon if player immediately failed the new dungeon
                            i -= 1;
                        }

                        j = 0;
                        reentering = true;
                        let dungeon = &self.dungeons[i];
                        if log_level > 0 {
                            println!("-------------------------");
                            println!("{} re-enters {}", self.player.name, dungeon.get_name());
                        }
                        break;
                    }
                }
                self.player.award_win(&enemy, log_level);
                j += 1;
            }

            if j != 0 {
                i += 1;
            }
        }

        GameResult::Win
    }
}

#[derive(Debug)]
enum GameResult {
    Win,
    Loss(i32),
}

enum FightResult {
    Win,
    Loss,
}

#[derive(PartialEq)]
enum Turn {
    Player,
    Enemy,
}

#[derive(Debug)]
struct Creature {
    name: String,
    max_hp: i32,
    current_hp: i32,
    attack: i32,
    defense: i32,
    magic: i32,
    wisdom: i32,
    speed: i32,
    exp: i32,
    level: i32,
    lives: i32,
    stat_pattern: StatPattern,
}

impl Creature {
    fn create(
        creature_type: CreatureType,
        stat_pattern: &StatPattern,
        level: u32,
        is_boss: bool,
        rng: &mut StdRng,
    ) -> Creature {
        let name = format!("{}{:?}", if is_boss { "Boss " } else { "" }, creature_type);

        let max_hp = max(
            (20.0 * stat_pattern.hp * level as f32 + (5.0 * rng.random_range(-1.0..1.0))) as i32,
            5,
        );
        let attack = max(
            (5.0 * stat_pattern.attack * level as f32 + (5.0 * rng.random_range(-1.0..1.0))) as i32,
            1,
        );
        let defense = max(
            (5.0 * stat_pattern.defense * level as f32 + (5.0 * rng.random_range(-1.0..1.0)))
                as i32,
            1,
        );
        let magic = max(
            (5.0 * stat_pattern.magic * level as f32 + (5.0 * rng.random_range(-1.0..1.0))) as i32,
            1,
        );
        let wisdom = max(
            (5.0 * stat_pattern.wisdom * level as f32 + (5.0 * rng.random_range(-1.0..1.0))) as i32,
            1,
        );
        let speed = max(
            (5.0 * stat_pattern.speed * level as f32 + (5.0 * rng.random_range(-1.0..1.0))) as i32,
            1,
        );

        let current_hp = max_hp;
        let exp = 0;
        let level = 1;
        let lives = 3;

        Creature {
            name,
            max_hp,
            current_hp,
            attack,
            defense,
            magic,
            wisdom,
            speed,
            exp,
            level,
            lives,
            stat_pattern: stat_pattern.clone(),
        }
    }

    fn award_win(&mut self, other: &Creature, log_level: u32) {
        let exp_increase = max(other.level - (self.level - 1), 1) * 12;

        self.exp += exp_increase;
        while self.exp >= 100 {
            self.level += 1;
            self.lives = min(self.lives + 1, 3);
            if log_level > 0 {
                println!("{} leveled up to level {}!", self.name, self.level);
            }

            self.exp = self.exp - 100;

            self.max_hp += (20.0 * self.stat_pattern.hp) as i32;
            self.attack += (5.0 * self.stat_pattern.attack) as i32;
            self.defense += (5.0 * self.stat_pattern.defense) as i32;
            self.magic += (5.0 * self.stat_pattern.magic) as i32;
            self.wisdom += (5.0 * self.stat_pattern.wisdom) as i32;
            self.speed += (5.0 * self.stat_pattern.speed) as i32;

            self.current_hp = self.max_hp;
        }
    }

    fn fight(&mut self, other: &mut Creature, log_level: u32) -> FightResult {
        let mut turn = if self.speed >= other.speed {
            Turn::Player
        } else {
            Turn::Enemy
        };

        loop {
            if self.current_hp <= 0 {
                if log_level > 0 {
                    println!("{} lost to a {}!", self.name, other.name);
                }
                return FightResult::Loss;
            } else if other.current_hp <= 0 {
                if log_level > 1 {
                    println!(
                        "{} defeated a {}! (HP: {}, Lives: {}, Exp: {})",
                        self.name, other.name, self.current_hp, self.lives, self.exp
                    );
                }
                return FightResult::Win;
            }

            if turn == Turn::Player {
                self.attack(other);
                turn = Turn::Enemy;
            } else {
                other.attack(self);
                turn = Turn::Player;
            }
        }
    }

    fn attack(&self, other: &mut Creature) {
        let physical_damage = max(self.attack - other.defense, 1);
        let magic_damage = max(self.magic - other.wisdom, 1);

        let damage = max(physical_damage, magic_damage);

        /*println!(
            "{} does {} damage to {} ({} -> {})",
            self.name,
            damage,
            other.name,
            other.current_hp,
            other.current_hp - damage
        );*/
        other.current_hp -= damage;
    }
}

#[derive(Clone, Copy, Debug)]
enum CreatureType {
    Player,
    // Common
    Bat,
    Dog,
    Goblin,
    Slime,
    Orc,
    // Uncommon
    Kitsune,
    Pixie,
    // Bosses
    HighOrc,
    Golem,
}

#[derive(Clone, Debug)]
struct StatPattern {
    hp: f32,
    attack: f32,
    defense: f32,
    magic: f32,
    wisdom: f32,
    speed: f32,
}

impl StatPattern {
    fn new(hp: f32, attack: f32, defense: f32, magic: f32, wisdom: f32, speed: f32) -> StatPattern {
        StatPattern {
            hp,
            attack,
            defense,
            magic,
            wisdom,
            speed,
        }
    }

    fn multiply(&mut self, multiplier: f32) {
        self.hp *= multiplier;
        self.attack *= multiplier;
        self.defense *= multiplier;
        self.magic *= multiplier;
        self.wisdom *= multiplier;
        self.speed *= multiplier;
    }
}

fn get_enemy_stat_pattern(enemy_type: CreatureType, multiplier: f32) -> StatPattern {
    let mut stat_pattern = match enemy_type {
        CreatureType::Bat => StatPattern::new(0.4, 0.4, 0.4, 0.4, 0.4, 2.0),
        CreatureType::Dog => StatPattern::new(0.5, 0.6, 0.5, 0.5, 0.5, 2.0),
        CreatureType::Slime => StatPattern::new(0.5, 0.4, 1.0, 0.5, 0.3, 0.5),
        CreatureType::Orc => StatPattern::new(0.6, 0.6, 0.6, 0.2, 0.2, 0.4),
        CreatureType::Kitsune => StatPattern::new(0.7, 0.2, 0.7, 1.0, 1.0, 1.5),
        CreatureType::Pixie => StatPattern::new(0.4, 0.2, 0.5, 2.0, 2.0, 1.5),
        _ => panic!(),
    };

    stat_pattern.multiply(multiplier);

    stat_pattern
}

// https://en.touhouwiki.net/wiki/Category:Locations
#[derive(Debug, FromPrimitive)]
enum LocationType {
    // Earthen
    Cave,
    Ravine,
    Valley,
    Land,
    Plains,
    Hills,
    Path,
    Realm,
    Mountains,
    Canyon,
    Desert,
    Jungle,
    Cliffs,
    Ridge,
    Badlands,
    Mesa,
    Divide,
    Cavern,
    Tree,
    // Buildings
    Castle,
    Temple,
    Ruins,
    Mansion,
    Cemetery,
    Prison,
    Shrine,
    Factory,
    Laboratory,
    Abattoir,
    Hall,
    Bunker,
    Altar,
    Remains,
    // Water
    Pond,
    Canal,
    Sea,
    Lake,
    Geyser,
    Marsh,
    Island,
    Cove,
    Isthmus,
    Shoal,
    Glacier,
    Fjord,
    // Wind
    Skies,
    Void,
    // Fire
    Volcano,
}

#[derive(Debug, FromPrimitive)]
enum ElementAdjective {
    // None
    Unremarkable,
    Lunar,
    Lingering,
    Mysterious,
    False,
    Abyssal,
    Dubious,
    Elegant,
    Moonlit,
    Spatial,
    Unearthly,
    Phantasmagorical,
    Confounding,
    // Fire
    Burning,
    Conflagrant,
    Scorching,
    Blazing,
    Purifying,
    // Water
    Freezing,
    Blizzardous,
    Rainy,
    Drowning,
    // Wind
    Voltaic,
    Wuthering,
    Tempestuous,
    Howling,
    // Earth
    Worldly,
    Twilight,
    Geotic,
    Abundant,
    Crystalline,
}

#[derive(Debug, FromPrimitive)]
enum Noun {
    Heaven,
    Hell,
    Willows,
    Light,
    Dreams,
    Truth,
    Lies,
    Hope,
    Blood,
    Doom,
    Storms,
    Serenity,
    Tranquility,
    Enlightenment,
    Rains,
    Rainbows,
    Pandemonium,
    Fantasies,
    Magic,
    Secrets,
    Flames,
    Pride,
    Obscurity,
    Resolve,
}

#[derive(Debug)]
struct Dungeon {
    pub location: LocationType,
    pub element: ElementAdjective,
    pub noun: Noun,
    pub level: u32,
}

impl Dungeon {
    fn from_hash(hash: u64, level: u32) -> Dungeon {
        let location_part = (hash & 0xFF) % mem::variant_count::<LocationType>() as u64;
        let element_part = ((hash & 0xFF00) >> 8) % mem::variant_count::<ElementAdjective>() as u64;
        let noun_part = ((hash & 0xFF0000) >> 16) % mem::variant_count::<Noun>() as u64;

        let location: LocationType = FromPrimitive::from_u32(location_part as u32).unwrap();
        let element: ElementAdjective = FromPrimitive::from_u32(element_part as u32).unwrap();
        let noun: Noun = FromPrimitive::from_u32(noun_part as u32).unwrap();

        Dungeon {
            location,
            element,
            noun,
            level,
        }
    }

    fn get_name(&self) -> String {
        format!(
            "{:?} of {:?} {:?} (lv. {})",
            self.location, self.element, self.noun, self.level
        )
    }

    fn create_enemy(&self, rng: &mut StdRng, is_boss: bool) -> Creature {
        let creature_type_mapping = HashMap::from([
            (
                (1, false),
                vec![CreatureType::Bat, CreatureType::Dog, CreatureType::Slime],
            ),
            ((1, true), vec![CreatureType::Orc]),
            (
                (2, false),
                vec![
                    CreatureType::Bat,
                    CreatureType::Dog,
                    CreatureType::Slime,
                    CreatureType::Orc,
                ],
            ),
            ((2, true), vec![CreatureType::Kitsune]),
            (
                (3, false),
                vec![
                    CreatureType::Bat,
                    CreatureType::Dog,
                    CreatureType::Slime,
                    CreatureType::Orc,
                    CreatureType::Kitsune,
                ],
            ),
            ((3, true), vec![CreatureType::Pixie]),
        ]);

        let default_types = vec![
            CreatureType::Bat,
            CreatureType::Dog,
            CreatureType::Slime,
            CreatureType::Orc,
            CreatureType::Kitsune,
            CreatureType::Pixie,
        ];

        let creature_types = creature_type_mapping
            .get(&(self.level, is_boss))
            .or(Some(&default_types))
            .unwrap();

        let creature_type = *creature_types.choose(rng).unwrap();
        let stat_pattern = get_enemy_stat_pattern(creature_type, if is_boss { 2.0 } else { 1.0 });
        Creature::create(creature_type, &stat_pattern, self.level + 1, is_boss, rng)
    }
}
