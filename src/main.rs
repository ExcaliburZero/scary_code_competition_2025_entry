#![feature(variant_count)]

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::prelude::*;
use std::{io, mem};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

const RESPONSE_TEMPLATES: [&str; 4] = [
    "Hello, {}! Nice to meet you!",
    "Nice to meet you {}!",
    "Hey {}! Did you see the Mets game last night? Was a real nail-biter!",
    "Greetings {}! It is a pleasure to make your acquaintance.",
];

fn main() {
    println!("Please enter your name:");
    print!("> ");
    io::stdout().flush().unwrap();
    let user_name = read_user_name();

    let mut rng = create_rng(&user_name);
    let mut game = Game::from_rng(&mut rng);

    let result = game.play(&mut rng);
    match result {
        GameResult::Win => println!("{}", game.response_template),
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
    println!("{result:?}");
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
        let player = Creature::create_player(rng.random());

        let response_template = RESPONSE_TEMPLATES
            [(rng.random::<u64>() % RESPONSE_TEMPLATES.len() as u64) as usize]
            .to_string();
        let response_template_parts: Vec<String> = response_template
            .split(" ")
            .map(|s| s.to_string())
            .collect();

        let dungeons: Vec<Dungeon> = (0..response_template_parts.len())
            .into_iter()
            .map(|_| Dungeon::from_hash(rng.random()))
            .collect();

        Game {
            player,
            dungeons,
            response_template,
            response_template_parts,
        }
    }

    fn play(&mut self, rng: &mut StdRng) -> GameResult {
        let mut game_result = GameResult::Win;
        for i in 0..self.dungeons.len() {
            let dungeon = &self.dungeons[i];

            for j in 0..5 {
                let mut enemy = Creature::create_player(rng.random());
                //println!("{:?}", enemy);

                let fight_result = self.player.fight(&mut enemy);
                if let FightResult::Loss = fight_result {
                    if self.player.lives == 0 {
                        game_result = GameResult::Loss(i as i32 - 1);
                        break;
                    } else {
                        self.player.lives -= 1;
                        self.player.current_hp = self.player.max_hp;
                    }
                }
            }
        }

        game_result
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
}

impl Creature {
    fn create_player(seed: u64) -> Creature {
        let hp_seed = seed & 0x1111;
        let attack_seed = seed & 0x1111;
        let attack_seed = seed & 0x1111;

        let name = "Tim".to_string();
        let max_hp = 20;
        let current_hp = max_hp;
        let attack = 8;
        let defense = 5;
        let magic = 5;
        let wisdom = 5;
        let speed = 5;
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
        }
    }

    fn fight(&mut self, other: &mut Creature) -> FightResult {
        let mut turn = if self.speed >= other.speed {
            Turn::Player
        } else {
            Turn::Enemy
        };

        loop {
            if self.current_hp < 0 {
                println!("{} is out of health!", self.name);
                return FightResult::Loss;
            } else if other.current_hp < 0 {
                println!("{} is out of health!", other.name);
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
        let damage = self.attack - other.defense;

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
}

impl Dungeon {
    fn from_hash(hash: u64) -> Dungeon {
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
        }
    }

    fn get_name(&self) -> String {
        format!("{:?} of {:?} {:?}", self.location, self.element, self.noun)
    }
}
