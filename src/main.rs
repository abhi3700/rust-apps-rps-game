use std::collections::HashMap;

use blake3::Hash as Blake3Hash;
use sha3::Sha3_256;

// As database for maintaining scores
struct Player {
    name: String,
    score: u32,
}

/// NOTE: We can also store the game history for each player.
/// Which option was opted by the player, who was the winner
struct Game {
    total_rounds: u32,
    round: Vec<Round>,
}

/// Each round of game has these fields
struct Round {
    id: u32,
    // None if 'Tie'
    winner: Option<String>,
    players: HashMap<String, Choice>,
    timestamp: u32,
}

/// Return hash of player's choice & salt.
/// NOTE: Salt is added to anonymize the choice made by the user, otherwise it
/// becomes very predictable for just 3 (or limited) choices in this case - Rock, Paper, Scissor.
/// Suppose, for example if we salt the choice made by Alice, then the hash committed is
/// unpredictable in terms of guessing the choice made.
///
/// Here, the salt is supposed to be changed on every choice made. Otherwise, the choice becomes predictable.
///
/// Q. Why not reveal the choice during the commit?
/// A. This is because in the world of internet in case of online gaming, there is network latency
/// which is inevitable as participants are most probably from different geographical locations.
/// So, we want a system that locks the choices made and also is secret enough to not get revealed until asked for.
///
/// Q. Why hashing?
/// A. This is because hashes are irreversible. And in cases of limited choices like here - Rock, Paper, Scissor.
/// It is recommended to add 'salt' to the choice before committing the hash.
///
/// Q. Why is it recommended to change the salt on every move selection?
/// A. It is done so that the choice made is unpredictable until asked to reveal.
///
/// Q. Why Blake3 hash function?
/// A. It's very fast on modern computers
///
/// TODO: Need to check the benchmark with 2 functions
fn commit_faster(choice: &str, salt: &str) -> Blake3Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(choice.as_bytes());
    hasher.update(salt.as_bytes());

    let hash = hasher.finalize();

    hash
}

/// Check if the hash of inputs (choice, salt) matches with the commit_hash
fn reveal_faster(commit_hash: Blake3Hash, choice: &str, salt: &str) -> bool {
    let computed_hash = commit_faster(choice, salt);

    computed_hash.eq(&commit_hash)
}

// use sha3::Digest;

/// Q. Why Keccak256 hash function?
/// A. It belongs to SHA3 family which is even stronger than Blake3.
// fn commit_stronger(choice: &str, salt: &str) -> dyn Digest {
//     let mut hasher = Sha3_256::new();
//     hasher.update(choice.as_bytes());
//     hasher.update(choice.as_bytes());
//     // read hash digest
//     let result = hasher.finalize();

//     result
// }

// fn reveal_stronger(commit_hash: String, inputs: &[String]) -> bool {}

/// Define a generic function to get user input
fn collect_input<T: std::str::FromStr>(prompt: &str) -> T {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim().parse() {
            Ok(value) => return value,
            Err(_) => continue,
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
enum Choice {
    Rock,
    Paper,
    Scissors,
    Empty,
}

/// Declare the winner based on the game logic.
/// Winner may be 'None' in cases where everyone is killing everyone.
/// Game rules:
/// - Rock beats scissors.
/// - Scissors beats paper.
/// - Paper beats rock.
/// Score rules:
/// - Tie: when players (all) select same option, score remains unchanged.
///         Suppose 2 of 3 players select Rock, Rock, Scissor.
///         So, 1st, 2nd player gain 1 point each & 3rd player 0
/// - Win/Lose: when either party wins based on game rule.
fn update_scores(
    players_details: &Vec<(String, Blake3Hash, Choice)>,
    players_scores: &mut HashMap<String, u32>,
) {
    // let choices = players_details.iter().map(|x| &x.2).collect::<Vec<_>>();
    for i in 0..players_details.len() {
        for j in i + 1..players_details.len() {
            // println!("comparo b/w players: {}--{}", i, j);
            if players_details[i].2 < players_details[j].2 {
                if players_details[i].2 == Choice::Rock && players_details[j].2 == Choice::Scissors
                {
                    *players_scores
                        .entry(players_details[i].0.clone())
                        .or_insert(0) += 1;
                } else {
                    *players_scores
                        .entry(players_details[j].0.clone())
                        .or_insert(0) += 1;
                }
            } else if players_details[i].2 > players_details[j].2 {
                *players_scores
                    .entry(players_details[i].0.clone())
                    .or_insert(0) += 1;
            }
        }
    }
}

fn main() {
    // maintain a player of HashMap type as no need to sort.
    let mut players_scores = HashMap::<String, u32>::new();

    // 1. collect players' commit-hash turn-wise
    let mut players_details = Vec::<(String, Blake3Hash, Choice)>::new();

    loop {
        // collect players count
        // loop until player count is valid
        let players_count = collect_input::<u32>("Enter number of players: ");
        if players_count < 2 {
            continue;
        }

        // collect players name & commit hashes
        for _ in 0..players_count {
            let player_name = collect_input::<String>("Enter your name: ");
            let player_commit_hash = collect_input::<Blake3Hash>(
                "Enter the commit hash of your choice (Rock, Paper, Scissors) with salt: ",
            );
            players_details.push((player_name.clone(), player_commit_hash, Choice::Empty));
            players_scores.insert(player_name, 0);
        }

        break;
    }

    println!("commit hashes: {:#?}", players_details);

    // 2. store to DB or the values remain on per session

    // 3. reveal the choices & salt & verify with reveal function
    // run in loop and ask for choice & salt. And then collect it for comparison.
    for i in 0..players_details.len() {
        // Keep asking (looping) the player until the choice & salt doesn't match corresponding to the committed hash.
        loop {
            let choice = collect_input::<String>(&format!(
                "{}, please reveal the choice: ",
                players_details[i].0
            ));

            let salt = collect_input::<String>("also please reveal the salt: ");

            if !reveal_faster(players_details[i].1, &choice, &salt) {
                continue;
            }

            // initialize
            let mut choice_enum_variant = Choice::Empty;

            // modify enum variant before added into players details
            if choice == "Rock".to_string() {
                choice_enum_variant = Choice::Rock;
            } else if choice == "Paper".to_string() {
                choice_enum_variant = Choice::Paper;
            } else if choice == "Scissors".to_string() {
                choice_enum_variant = Choice::Scissors;
            }

            // set choice variant to player
            players_details[i].2 = choice_enum_variant;

            break;
        }
    }

    // 4. update the scores
    update_scores(&players_details, &mut players_scores);

    // 5. print the scores
    println!("The game score so far is:");
    for name in players_scores.keys() {
        println!("- {name}: {}", players_scores.get(name).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_commit_blake3_256() {
        let hash = commit_faster("rock", "abhi");
        // println!("{}", hash.to_string());
        // dbg!(hash);
        assert_eq!(
            hash,
            hex!("e59fb98489b367c5b248195c62f176deffeb3da71fbec56d0c42fd88acbe3b2b")
        );
    }

    #[test]
    fn test_reveal_blake3_256() {
        let hash = commit_faster("rock", "abhi");
        assert!(reveal_faster(hash, "rock", "abhi"));
    }

    #[test]
    fn test_update_scores() {
        todo!()
        // define a players details (fetch from `sample.json` file)

        // define a players scores list
    }
}
