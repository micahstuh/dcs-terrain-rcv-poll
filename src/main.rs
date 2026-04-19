use candidate::Candidate;
use rfd::FileDialog;
use std::{fs::File, rc::Rc};
use tabled::builder::Builder;
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
    let initial_order_tallies = get_candidate_tallies(&voters);
    let initial_order_candidates: Vec<Rc<Candidate>> = initial_order_tallies
        .iter()
        .map(|(candidate, _)| Rc::clone(candidate))
        .collect();
    let mut winner_found = false;
    // Perform a runoff eliminations until a majority winner is found
    let mut round_index: i32 = 1;
    while !winner_found && voters[0].votes.len() > 0 {
        println!("\nRound {}:", round_index);
        // Print the current primary votes for the current round.
        graph_tallies(&voters);

        // Check if a candidate has a majority of primary votes.
        let majority_result = check_for_majority(&voters);

        if majority_result == None {
            // No candidate has a majority vote, eliminate the candidate with the fewest primary votes.
            println!("  No Majority Winner,");
            let removed_candidates =
                remove_last_place_candidate(&mut voters, &initial_order_candidates);
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

/// Imports a CSV poll file and returns a list of voters.
///
/// # Arguments
///
/// * `file_path` - The path to the CSV file.
///
/// # Returns
///
/// A list of voters from the CSV file.
pub fn import_csv_poll(file_path: &str) -> Result<Vec<Voter>, String> {
    let file = File::open(file_path).expect("Error");
    let mut reader = csv::Reader::from_reader(file);

    let header_record = reader.headers().expect("Error").clone();
    let mut headers = Vec::new();
    for result in header_record.iter() {
        let header = result.to_owned();
        headers.push(header);
    }
    // Remove non-candidate questions.
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
        // Remove non-candidate answers.
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

/// Checks if a candidate has a majority of primary votes.
///
///  # Arguments
///
/// * `voters` - The list of voters to check for a majority winner.
///
/// # Returns
///
/// The candidate with a majority of primary votes, or None if no candidate has a majority.
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

/// Removes the losing candidates from the candidates list.
///
///  # Arguments
///
/// * `voters` - The list of voters to remove the last place candidate from.
///
/// # Returns
///
/// A vector of removed candidates.
pub fn remove_last_place_candidate(
    voters: &mut Vec<Voter>,
    initial_order_candidates: &Vec<Rc<Candidate>>,
) -> Vec<Rc<Candidate>> {
    let candidate_tallies = get_candidate_tallies(voters);

    // Get the last item from candidate_tallies. If it has the same values as items above it, add those to tied_losers as well.
    let (last_candidate, last_tallies) = candidate_tallies.last().unwrap();
    let mut tied_losers = vec![(last_candidate.clone(), last_tallies.clone())];
    for (candidate, tallies) in &candidate_tallies[..candidate_tallies.len()] {
        if tallies == last_tallies {
            tied_losers.push((candidate.clone(), tallies.clone()));
        }
    }

    // If losers are tied entirely, remove the one lower on the list of the initial vote set.
    if tied_losers.len() > 1 {
        let max_loser = tied_losers
            .iter()
            .max_by_key(|(candidate, _)| {
                initial_order_candidates
                    .iter()
                    .position(|c| c == candidate)
                    .unwrap()
            })
            .cloned();
        tied_losers = vec![max_loser.unwrap()];
    }

    // Remove the losing candidates from the voters' tallies.
    for (candidate, _) in &tied_losers {
        for voter in &mut *voters {
            voter.votes.retain(|x| x != candidate);
        }
    }

    let removed_candidates: Vec<Rc<Candidate>> = tied_losers.iter().map(|x| x.0.clone()).collect();

    return removed_candidates;
}

/// Get the tallies for each candidate from the list of voters.
///
/// # Arguments
///
/// * `voters` - The list of voters to get the tallies from.
///
/// # Returns
///
/// A vector of tuples containing the candidate and their tallies, ordered by favoritism.
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
                let mut tallies: Vec<i32> = vec![0; voter.votes.len()];
                tallies[i] += 1;
                candidate_tallies.push((candidate, tallies));
            }
        }
    }
    candidate_tallies.sort_by(|a, b| b.1.cmp(&a.1));
    return candidate_tallies;
}

/// Extracts the number from a string.
///
/// # Arguments
///
/// * `entry` - The string to extract the number from.
///
/// # Returns
///
/// The number extracted from the string.
pub fn extract_number(entry: &str) -> i32 {
    let mut number = String::new();
    for c in entry.chars() {
        if c.is_numeric() {
            number.push(c);
        }
    }
    return number.parse().unwrap();
}

pub fn graph_tallies(voters: &Vec<Voter>) {
    let candidate_tallies = get_candidate_tallies(&voters);
    let mut builder = Builder::default();

    let num_rounds = candidate_tallies.len();
    let mut header = vec!["Preference".to_string()];
    for i in 1..=num_rounds {
        header.push(format!("{}", i));
    }
    builder.push_record(&header);

    for (candidate, tallies) in candidate_tallies {
        let mut row = vec![candidate.name.clone()];
        for tally in tallies {
            row.push(tally.to_string());
        }
        builder.push_record(&row);
    }

    let table = builder.build();
    println!("{}", table);
}
