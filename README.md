# DCS Terrain RCV Poll
Tallies the votes for a terrain poll, per ranked choice voting rules.

# How it works
- Voters rank terrain candidates in order of preferrence.
- If a terrain does not receive a majority of first-choice votes, the terrain with the fewest first-choice votes will be eliminated. 
    - If there is a tie for fewest first-choice votes, the loser terrains will enter a tie-breaker by checking the second-choice votes.
    - If second-choice votes are tied, the third-choice vote will be checked, and so on, and so forth.
- The votes given to an eliminated terrain are redistributed to the remaining terrains, in order of the voter's preferrence.
### This pattern repeates until a single terrain has majority of first-choice votes.