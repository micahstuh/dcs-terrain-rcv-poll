use std::fs;
use rfd::FileDialog;

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

    let terrain_voting_index_in_file = 2; // 0th datetime, 1st being discordname, etc

    // Must be in order as shown in poll
    let mut terrains: Vec<&str> = Vec::from([
        "Caucasus",
        "Kola",
        "Marianas",
        "NTTR",
        "Persian Gulf",
        "Sinai",
        "South Atlantic",
        "Afghanistan",
        "Syria",
        "Iraq",
    ]);

    // First index is a member's entry, 2nd index is their vote score per terrain
    let mut numeric_entries = import_numeric_entries(
        &poll_file,
        terrains.len() as i32,
        terrain_voting_index_in_file,
    );

    // Perform a runoff eliminations until a majority winner is found
    let mut round_index: i32 = 1;
    let mut winner_found: bool = false;
    while !winner_found && terrains.len() > 0 {
        println!("\nRound {}:", round_index);
        // Print the current primary votes for the current round.
        show_tallies(&terrains, &numeric_entries);

        // Check if a terrain has a majority of primary votes.
        let majority_result = check_for_majority(&numeric_entries);

        if majority_result == -1 {
            // No terrain has a majority vote, eliminate the terrain with the fewest primary votes.
            println!("  No Majority Winner,");
            let removed_terrains = remove_last_place_terrain(&mut terrains, &mut numeric_entries);
            println!("  Removing Last Place Terrains:");
            for terrain in removed_terrains {
                println!("      {}", terrain);
            }
        } else {
            // Hussah, a terrain has a majority of primary votes!
            println!(
                "  Winner by Majority:\n      {}\n",
                terrains[majority_result as usize]
            );
            winner_found = true;
        }
        round_index += 1;
    }
    if !winner_found {
        println!("  Results are tied. No Winner!");
    }
}

/// Convert a Google Form's CSV output to a manageable numeric entries list.
///
///  # Arguments
///
/// * `poll_file` - The filepath to the CSV file to be imported.
/// * `terrain_count` - The number of terrains to expect per entry in the CSV file.
/// * `voting_index` - The answer index of the first terrain per entry in the CSV file.
///
/// # Returns
/// The list of poll entries.
pub fn import_numeric_entries(
    poll_file: &str,
    terrain_count: i32,
    voting_index: i32,
) -> Vec<Vec<i32>> {
    let mut numeric_entries: Vec<Vec<i32>> = Vec::new(); // First vec represents a person's entry, second vec represents each terrain's score

    let poll_contents = fs::read_to_string(poll_file).expect("Error");
    let mut poll_entries = poll_contents.split('\n');
    // Discard the header info
    poll_entries.next();

    // For each entry, tally their votes into a vector
    for entry in poll_entries {
        let mut numeric_entry: Vec<i32> = Vec::new();
        let mut answers = entry.split(',');

        // Skip to the terrain entries
        answers.nth((voting_index - 1) as usize);

        // For each terrain entry, record the score given.
        for _ in 0..terrain_count {
            let answer = answers.next().unwrap().to_owned().replace('"', "");
            let vote_value: i32 = extract_number(&answer);
            numeric_entry.push(vote_value);
        }
        numeric_entries.push(numeric_entry);
    }

    return numeric_entries;
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
pub fn check_for_majority(numeric_entries: &Vec<Vec<i32>>) -> i32 {
    let mut primary_indices: Vec<i32> = Vec::new();
    let terrain_count = numeric_entries[0].len();
    for _ in 0..terrain_count {
        primary_indices.push(0);
    }

    for entry in numeric_entries {
        let min_index = get_index_of_minimum(entry);
        primary_indices[min_index] += 1;
    }

    let most_primary_votes_index = get_index_of_maximum(&primary_indices);
    let most_primary_votes = primary_indices.iter().max().unwrap();

    let leader_vote_percentage: f32 = *most_primary_votes as f32 / numeric_entries.len() as f32;

    if leader_vote_percentage > 0.5 {
        return most_primary_votes_index as i32;
    } else {
        return -1;
    }
}

/// Returns the index of the maximum value in a list.
///
///  # Arguments
///
/// * `vector` - The vector to search.
pub fn get_index_of_maximum(vector: &Vec<i32>) -> usize {
    let mut max_value: i32 = 0;
    let mut max_index: usize = 0;
    for idx in 0..vector.len() {
        if vector[idx] > max_value {
            max_value = vector[idx];
            max_index = idx;
        }
    }
    return max_index;
}

/// Returns the index of the minimum value in a list.
///
///  # Arguments
///
/// * `vector` - The vector to search.
pub fn get_index_of_minimum(vector: &Vec<i32>) -> usize {
    let mut min_value: i32 = i32::MAX;
    let mut min_index: usize = 0;
    for idx in 0..vector.len() {
        if vector[idx] < min_value {
            min_value = vector[idx];
            min_index = idx;
        }
    }
    return min_index;
}

/// Removes the losing terrains from the terrains list and the votes for it
/// from the numeric entries list. Returns a vector of removed terrains.
///
///  # Arguments
///
/// * `terrains` - The list of remaining terrains.
/// * `numeric_entries` - The list of poll entries.
pub fn remove_last_place_terrain(
    terrains: &mut Vec<&str>,
    numeric_entries: &mut Vec<Vec<i32>>,
) -> Vec<String> {
    let mut primary_indices: Vec<i32> = Vec::new();
    let terrain_count = numeric_entries[0].len();
    for _ in 0..terrain_count {
        primary_indices.push(0);
    }

    // Tally up the primary votes for each terrain
    for entry in &mut *numeric_entries {
        let min_index = get_index_of_minimum(&entry);
        primary_indices[min_index] += 1;
    }

    // Get the value of fewest primary votes amongst the terrains.
    let least_primary_votes = *primary_indices.iter().min().unwrap();
    let mut removed_terrains: Vec<String> = Vec::new();

    let mut tied_losers: Vec<usize> = Vec::new();

    // For each terrain, add it to the loser list if it is equal to the fewest
    // number of primary votes received.
    for terrain_index in 0..terrain_count {
        if &primary_indices[terrain_index as usize] == &least_primary_votes {
            tied_losers.push(terrain_index as usize);
        }
    }

    // Enter into a loser tiebreaker if more than one loser exists.
    if tied_losers.len() > 1 {
        println!("  Entering Into Loser Tiebraker");
        loser_tie_breaker(&terrains, &mut tied_losers, &numeric_entries);
    }

    // For each poll entry, remove the votes for terrains that are going to be deleted.
    let mut terrain_index: i32 = 0;
    while terrain_index < terrains.len() as i32 {
        if tied_losers.contains(&(terrain_index as usize)) {
            // Any votes above the removed terrain get demoted by -1.
            for entry in &mut *numeric_entries {
                let eliminated_terrain_value: i32 = entry[terrain_index as usize];
                for terrain_vote_index in 0..entry.len() {
                    if entry[terrain_vote_index] > eliminated_terrain_value {
                        entry[terrain_vote_index] -= 1;
                    }
                }
                entry.remove(terrain_index as usize);
            }

            // Remove the index from any vector that corresponds to the loser terrain.
            removed_terrains.push(terrains[terrain_index as usize].to_owned());
            terrains.remove(terrain_index as usize);
            primary_indices.remove(terrain_index as usize);
            tied_losers.remove(0);
            for i in 0..tied_losers.len() {
                tied_losers[i] -= 1;
            }
            terrain_index -= 1;
        }
        terrain_index += 1;
    }

    return removed_terrains;
}

/// Finds the terrains with the lowest number of subsequent votes and removes it
/// from the terrains list.
///
///  # Arguments
///
/// * `terrains` - The list of remaining terrains.
/// * `tied_terrain_indicies` - The indicies of the tied loser terrains in the terrains list.
/// * `numeric_entries` - The list of poll entries.
pub fn loser_tie_breaker(
    terrains: &Vec<&str>,
    tied_terrain_indicies: &mut Vec<usize>,
    numeric_entries: &Vec<Vec<i32>>,
) {
    // In the event of a tie, find which has the fewest 2nd picks. If 2nd picks are a tie, go by 3rd pick, and so on.

    // Go by rounds until only one loser exists.
    // Each round corresponds to a vote tier. 0th round is primary vote, 1st round is secondary, 2nd round is tertiary, etc.
    let mut round: i32 = 1;
    while round < numeric_entries[0].len() as i32 && tied_terrain_indicies.len() > 1 {
        println!("      Tiebreaker Round {}:", round);
        let mut vote_tallies: Vec<i32> = Vec::new();
        for idx in 0..tied_terrain_indicies.len() {
            vote_tallies.push(0);
            for entry in numeric_entries {
                if entry[tied_terrain_indicies[idx]] == round + 1 {
                    vote_tallies[idx] += 1;
                }
            }
            println!(
                "          {} has {} votes",
                terrains[tied_terrain_indicies[idx]], vote_tallies[idx]
            );
        }

        let min_votes = *vote_tallies.iter().min().unwrap();

        // Remove terrains from tied list if there was a worse score.
        let mut idx: i32 = 0;
        while idx < tied_terrain_indicies.len() as i32 {
            if vote_tallies[idx as usize] != min_votes {
                tied_terrain_indicies.remove(idx as usize);
                vote_tallies.remove(idx as usize);
                idx -= 1;
            }
            idx += 1;
        }
        round += 1;
    }
}

/// Prints the primary tallies from the numeric entries list.
///
///  # Arguments
///
/// * `terrains` - The list of remaining terrains.
/// * `numeric_entries` - The list of poll entries.
pub fn show_tallies(terrains: &Vec<&str>, numeric_entries: &Vec<Vec<i32>>) {
    println!("  Primary Vote Tallies:");
    for terrain_index in 0..terrains.len() {
        let mut primary_votes: i32 = 0;
        for entry in numeric_entries {
            if entry[terrain_index] == 1 {
                primary_votes += 1;
            }
        }
        println!("      {}: {}", terrains[terrain_index], primary_votes);
    }
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