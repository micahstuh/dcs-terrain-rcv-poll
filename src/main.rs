
use core::num;
use std::{fs, ops::Index};

fn main() {
    let poll_file: String = "data\\2023-06-09 Next Campaign Poll 2.csv".to_owned();
    let terrain_voting_index_in_file = 2;

    let terrains: Vec<&str> = Vec::from([
        "Caucasus",
        "Marianas",
        "Nevada",
        "Persian Gulf",
        "South Atlantic",
        "Syria",
        "Senai",
   ]);

    let numeric_votes = gather_terrain_votes(&poll_file, terrains.len() as i32, terrain_voting_index_in_file);

    let majority_result = check_for_majority(&numeric_votes);

    if majority_result == -1 {

    } else {

    }

}

pub fn gather_terrain_votes(poll_file: &str, terrain_count: i32, voting_index: i32) -> Vec<Vec<i32>> {

    let mut numeric_votes: Vec<Vec<i32>> = Vec::new(); //First vec represents terrian, 2nd vec represents vote value
    for _ in 0..terrain_count {
        let mut terrain: Vec<i32> = Vec::new();
        for _ in 0..terrain_count {
            terrain.push(0);
        }
        numeric_votes.push(terrain);
    }

    let poll_contents = fs::read_to_string(poll_file).expect("Error");
    let mut poll_entries = poll_contents.split('\n');
    poll_entries.next();
    for entry in poll_entries {
        let mut answers = entry.split(',');
        answers.nth((voting_index - 1) as usize);
        for i in 0..terrain_count {
            let answer = answers.next().unwrap().to_owned().replace('"', "");
            let mut vote_value: i32 = 0;
            if answer.len() > 1 {
                if answer.contains("1") {
                    vote_value = 1;
                } else if answer.contains(&terrain_count.to_string()) {
                    vote_value = terrain_count;
                }
            } else {
                vote_value = answer.parse().unwrap();
            }
            numeric_votes[i as usize][vote_value as usize - 1] += 1;
        }
    }

    return numeric_votes;
}

pub fn check_for_majority(numeric_votes: &Vec<Vec<i32>>) -> i32 {

    let mut current_index: usize = 0;
    let mut most_primary_votes_index: usize = 0;
    let mut total_primary_votes: i32 = 0;
    for terrain in numeric_votes {
        let primary_votes: &i32 = terrain.last().unwrap();
        if primary_votes > numeric_votes[most_primary_votes_index].last().unwrap() {
            most_primary_votes_index = current_index;
        }
        total_primary_votes += primary_votes;
        current_index += 1;
    }

    if (numeric_votes[most_primary_votes_index].last().unwrap() / total_primary_votes) as f32 > 0.5 {
        return most_primary_votes_index as i32;
    } else {
        return -1;
    }
}

pub fn remove_last_place_terrain(numeric_votes: &Vec<Vec<i32>>) {
    let mut current_index: usize = 0;
    let mut least_primary_votes_index: usize = 0;
    for terrain in numeric_votes {
        let primary_votes: &i32 = terrain.last().unwrap();
        if primary_votes < numeric_votes[least_primary_votes_index].last().unwrap() {
            least_primary_votes_index = current_index;
        }
        current_index += 1;
    }

    for terrain in numeric_votes {
        if terrain.last().unwrap() == numeric_votes[least_primary_votes_index].last().unwrap() {
            
        }
    }

}