use candidate::Candidate;
use rfd::FileDialog;
use std::{fs::File, rc::Rc};
use voter::Voter;

mod candidate;
mod voter;

fn main() {
    let files = FileDialog::new()
        .add_filter("Google Form CSV Export", &["csv"])
        .set_directory("example polls")
        .pick_file();

    let poll_file: String;
    if let Some(file) = files {
        poll_file = file.to_str().unwrap().to_owned();
    } else {
        print!("Selection Not Valid!");
        return;
    }

    let mut voters = import_csv_poll(&poll_file).expect("Could not import poll");
    let mut winner_found = false;
    // Perform a runoff eliminations until a majority winner is found
    let mut round_index: i32 = 1;
    while !winner_found && voters[0].votes.len() > 0 {
        println!("\nRound {}:", round_index);
        // Print the current primary votes for the current round.
        show_tallies(&voters);

        // Check if a candidate has a majority of primary votes.
        let majority_result = check_for_majority(&voters);

        if majority_result == None {
            // No candidate has a majority vote, eliminate the candidate with the fewest primary votes.
            println!("  No Majority Winner,");
            let removed_candidates = remove_last_place_candidate(&mut voters);
            println!("  Removing Last Place Candidates:");
            for candidate in removed_candidates {
                println!("      {}", candidate.name);
            }
        } else {
            // Hussah, a candidate has a majority of primary votes!
            winner_found = true;
            println!(
                "  Winner by Majority:\n      {}\n",
                majority_result.unwrap().name
            );
        }
        round_index += 1;
    }
    if !winner_found {
        println!("  Results are tied. No Winner!");
    }
}

pub fn import_csv_poll(file_path: &str) -> Result<Vec<Voter>, String> {
    let file = File::open(file_path).expect("Error");
    let mut reader = csv::Reader::from_reader(file);

    let header_record = reader.headers().expect("Error").clone();
    let mut headers = Vec::new();
    for result in header_record.iter() {
        let header = result.to_owned();
        headers.push(header);
    }
    headers.remove(0);
    headers.remove(0);
    headers.pop();

    let mut candidates: Vec<Candidate> = Vec::new();

    // Gather all candidates from the CSV headers
    for header_string in headers {
        let split_header: Vec<&str> = header_string.split(|c| c == '[' || c == ']').collect();
        if split_header.len() > 1 {
            let candidate = Candidate::new(split_header[1]);
            candidates.push(candidate);
        } else {
            return Err("CSV headers are not formatted correctly".to_string());
        }
    }

    let mut voters: Vec<Voter> = Vec::new();

    for result in reader.records() {
        let record = result.expect("Error");
        let mut answers = Vec::new();
        for answer_string in record.iter() {
            let answer = answer_string.to_owned();
            answers.push(answer);
        }
        answers.remove(0);
        answers.pop();
        let name = answers.remove(0);
        let mut votes: Vec<i32> = Vec::new();
        for answer in answers {
            let vote = extract_number(&answer);
            votes.push(vote);
        }
        let mut candidate_votes: Vec<Rc<Candidate>> = Vec::new();
        for score in 1..votes.len() + 1 {
            let index = votes
                .iter()
                .position(|&x| x == score as i32)
                .expect("Candidate not found");
            candidate_votes.push(Rc::new(candidates[index].clone()));
        }
        let voter = Voter::new(name, candidate_votes);
        voters.push(voter);
    }

    return Ok(voters);
}

/// Check the numeric entries for a majority winner of primary votes.
///
///  # Arguments
///
/// * `numeric_entries` - The list of poll entries.
///
/// # Returns
///
/// The index of the majority winner, otherwise -1 if
/// no majoriy winner exists.
pub fn check_for_majority(voters: &Vec<Voter>) -> Option<Rc<Candidate>> {
    let candidate_tallies = get_candidate_tallies(&voters);

    let most_primary_votes_index = candidate_tallies
        .iter()
        .position(|x| x.1[0] == candidate_tallies.iter().map(|x| x.1[0]).max().unwrap())
        .unwrap();
    let most_primary_votes = candidate_tallies[most_primary_votes_index].1[0];

    let leader_vote_percentage: f32 = most_primary_votes as f32 / voters.len() as f32;

    if leader_vote_percentage > 0.5 {
        return Some(candidate_tallies[most_primary_votes_index].0.clone());
    } else {
        return None;
    }
}

/// Removes the losing candidates from the candidates list and the votes for it
/// from the numeric entries list. Returns a vector of removed candidates.
///
///  # Arguments
///
/// * `candidates` - The list of remaining candidates.
/// * `numeric_entries` - The list of poll entries.
pub fn remove_last_place_candidate(voters: &mut Vec<Voter>) -> Vec<Rc<Candidate>> {
    let candidate_tallies = get_candidate_tallies(voters);
    let min_votes = candidate_tallies.iter().map(|x| x.1[0]).min().unwrap();

    let mut tied_losers: Vec<(Rc<Candidate>, Vec<i32>)> = Vec::new();

    // For each candidate, add it to the loser list if it is equal to the fewest
    // number of primary votes received.
    for (candidate, tallies) in candidate_tallies {
        if tallies[0] == min_votes {
            tied_losers.push((candidate, tallies));
        }
    }

    // Enter into a loser tiebreaker if more than one loser exists.
    if tied_losers.len() > 1 {
        println!("  Entering Into Loser Tiebraker");
        loser_tie_breaker(&mut tied_losers);
    }

    for (candidate, _) in &tied_losers {
        for voter in &mut *voters {
            voter.votes.retain(|x| x != candidate);
        }
    }

    let removed_candidates: Vec<Rc<Candidate>> = tied_losers.iter().map(|x| x.0.clone()).collect();

    return removed_candidates;
}

/// Finds the candidates with the lowest number of subsequent votes and removes it
/// from the candidates list.
///
///  # Arguments
///
/// * `candidates` - The list of remaining candidates.
/// * `tied_candidate_indicies` - The indicies of the tied loser candidates in the candidates list.
/// * `numeric_entries` - The list of poll entries.
pub fn loser_tie_breaker(candidate_tallies: &mut Vec<(Rc<Candidate>, Vec<i32>)>) {
    // In the event of a tie, find which has the fewest 2nd picks. If 2nd picks are a tie, go by 3rd pick, and so on.

    // Go by rounds until only one loser exists.
    // Each round corresponds to a vote tier. 0th round is primary vote, 1st round is secondary, 2nd round is tertiary, etc.
    let mut round: i32 = 1;
    while round < candidate_tallies[0].1.len() as i32 && candidate_tallies.len() > 1 {
        println!("      Tiebreaker Round {}:", round);
        for i in 0..candidate_tallies.len() {
            println!(
                "          {} has {} votes",
                candidate_tallies[i].0.name, candidate_tallies[i].1[round as usize]
            );
        }
        let min_votes = candidate_tallies
            .iter()
            .map(|x| x.1[round as usize])
            .min()
            .unwrap();

        candidate_tallies.retain(|x| x.1[round as usize] == min_votes);
        round += 1;
    }
}

/// Prints the primary tallies from the numeric entries list.
///
///  # Arguments
///
/// * `candidates` - The list of remaining candidates.
/// * `numeric_entries` - The list of poll entries.
pub fn show_tallies(voters: &Vec<Voter>) {
    println!("  Primary Vote Tallies:");
    let candidate_tallies = get_candidate_tallies(&voters);
    for (candidate, tallies) in candidate_tallies {
        println!("      {}: {}", candidate.name, tallies[0]);
    }
}

pub fn get_candidate_tallies(voters: &Vec<Voter>) -> Vec<(Rc<Candidate>, Vec<i32>)> {
    let mut candidate_tallies: Vec<(Rc<Candidate>, Vec<i32>)> = Vec::new();

    for voter in voters {
        for i in 0..voter.votes.len() {
            let vote = &voter.votes[i];
            let index = candidate_tallies.iter().position(|x| x.0 == *vote);
            if let Some(index) = index {
                candidate_tallies[index].1[i] += 1;
            } else {
                let candidate = vote.clone();
                let mut tallies: Vec<i32> = vec![0; voters.len()];
                tallies[i] += 1;
                candidate_tallies.push((candidate, tallies));
            }
        }
    }

    return candidate_tallies;
}

/// Extracts the number from a string.
///
/// # Arguments
///
/// * `entry` - The string to extract the number from.
pub fn extract_number(entry: &str) -> i32 {
    let mut number = String::new();
    for c in entry.chars() {
        if c.is_numeric() {
            number.push(c);
        }
    }
    return number.parse().unwrap();
}
