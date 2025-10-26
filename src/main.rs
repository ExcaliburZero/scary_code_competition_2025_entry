use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::io::prelude::*;

fn main() {
    println!("Please enter your name:");
    print!("> ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();

    let mut user_name = "".to_string();
    stdin.lock().read_line(&mut user_name).unwrap();
    user_name = user_name.trim().to_string();

    let mut player = Creature::create_player(&user_name);
    println!("{player:?}");

    let mut game_result = Option::None;
    for _ in 0..5 {
        let mut enemy = Creature::create_player("Bat");
        //println!("{:?}", enemy);

        let fight_result = player.fight(&mut enemy);
        if let FightResult::Loss = fight_result {
            if player.lives == 0 {
                game_result = Some(GameResult::Loss);
                break;
            } else {
                player.lives -= 1;
                player.current_hp = player.max_hp;
            }
        }
    }

    println!("{game_result:?}");
}

#[derive(Debug)]
enum GameResult {
    Win(String),
    Loss,
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
    fn create_player(user_name: &str) -> Creature {
        let mut s = DefaultHasher::new();
        user_name.hash(&mut s);
        let seed = s.finish();

        let hp_seed = seed & 0x1111;
        let attack_seed = seed & 0x1111;
        let attack_seed = seed & 0x1111;

        //let name = "Tim".to_string();
        let name = user_name.to_string();
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
