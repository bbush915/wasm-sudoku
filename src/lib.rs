use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Puzzle {
  initial_cells: utils::Cells,
  current_cells: utils::Cells,
}

#[wasm_bindgen]
impl Puzzle {
  pub fn new() -> Puzzle {
    let mut puzzle = Puzzle {
      initial_cells: [0; 81],
      current_cells: [0; 81],
    };

    puzzle.generate();

    puzzle
  }

  pub fn cells(&self) -> *const u8 {
    self.current_cells.as_ptr()
  }

  pub fn generate(&mut self) {
    self.initial_cells = generator::generate();
    self.current_cells.copy_from_slice(&self.initial_cells);
  }

  pub fn verify(&self) -> bool {
    return utils::verify(&self.current_cells);
  }

  pub fn solve(&mut self) {
    let mut grid = [0; 81];
    grid.copy_from_slice(&self.initial_cells);

    let solutions = solver::solve(grid, true, false, None);
    self.current_cells = solutions[0];
  }
}

mod generator {
  use rand::seq::SliceRandom;
  use rand::Rng;
  use std::collections;

  use crate::solver;
  use crate::utils;

  pub fn generate() -> utils::Cells {
    let mut grid = generate_valid_grid();
    remove_cells(&mut grid, Some(35));

    grid
  }

  fn generate_valid_grid() -> utils::Cells {
    let mut rng = rand::thread_rng();

    let mut grid = [0; 81];
    let mut indices: collections::HashSet<usize> = (0..81).collect();

    while indices.len() > 0 {
      // NOTE - Pick a random cell.

      let index = indices
        .iter()
        .nth(rng.gen_range(0, indices.len()))
        .unwrap()
        .clone();

      // NOTE - Pick a random candidate value.

      let candidates = utils::get_candidates(&grid, index);
      grid[index] = candidates.choose(&mut rng).unwrap().clone();

      // NOTE - Verify that we can still solve the grid.

      let solutions = solver::solve(grid, true, false, Some(100));

      if solutions.len() == 0 {
        grid[index] = 0;
      } else {
        indices.remove(&index);
      }
    }

    grid
  }

  fn remove_cells(grid: &mut utils::Cells, desired_clue_threshold: Option<u8>) {
    let mut rng = rand::thread_rng();

    let mut indices: Vec<usize> = (0..81).collect();
    indices.shuffle(&mut rng);

    let mut counter = 0;

    for i in indices.into_iter() {
      let old_value = grid[i];

      grid[i] = 0;

      if solver::solve(*grid, false, true, Some(100)).len() > 1 {
        // NOTE - No longer have a unique solution, so need to revert.
        grid[i] = old_value
      } else {
        counter += 1;

        if desired_clue_threshold.is_some() && counter >= (81 - desired_clue_threshold.unwrap()) {
          break;
        }
      }
    }
  }
}

mod solver {
  use crate::utils;

  struct Step {
    index: usize,
    candidates: Vec<u8>,
  }

  pub fn solve(
    mut grid: utils::Cells,
    check_solvable: bool,
    check_unique: bool,
    backtrack_threshold: Option<u32>,
  ) -> Vec<utils::Cells> {
    let mut solutions: Vec<utils::Cells> = Vec::new();

    let mut steps: Vec<Step> = Vec::new();
    let mut backtracks = 0;

    loop {
      if backtrack_threshold.is_some() && backtracks >= backtrack_threshold.unwrap() {
        return solutions;
      }

      match generate_step(&grid) {
        Some(mut step) => match step.candidates.pop() {
          Some(candidate) => {
            // NOTE - Try next candidate for this step.

            grid[step.index] = candidate;
            steps.push(step);
          }

          None => {
            // NOTE - No candidates left to try for this step, so back we go!

            backtracks += 1;

            if !try_backtrack(&mut grid, &mut steps) {
              break;
            }
          }
        },

        None => {
          // NOTE - Unable to generate a new step, which means we found a
          // solution! Add it to the list and use the parameters to determine
          // if we can stop.

          let mut solution: utils::Cells = [0; 81];

          solution.copy_from_slice(&grid);
          solutions.push(solution);

          // NOTE - If we found a solution, we have proved solvability, and can
          // stop looking.

          if check_solvable {
            break;
          }
          // NOTE - If we found multiple solutions, we have disproved uniqueness, and can
          // stop looking.

          if check_unique && solutions.len() > 1 {
            break;
          }

          // NOTE - Continue on!

          backtracks += 1;

          if !try_backtrack(&mut grid, &mut steps) {
            break;
          }
        }
      }
    }

    solutions
  }

  fn generate_step(grid: &utils::Cells) -> Option<Step> {
    let first_empty_cell = grid.iter().position(|&x| x == 0);

    if first_empty_cell == None {
      return None;
    }

    let mut best_cell_index: usize = first_empty_cell.unwrap();
    let mut best_cell_candidates: Vec<u8> = utils::get_candidates(&grid, best_cell_index);

    for i in (best_cell_index + 1)..81 {
      if grid[i] != 0 {
        continue;
      }

      let candidates = utils::get_candidates(&grid, i);

      if candidates.len() < best_cell_candidates.len() {
        best_cell_index = i;
        best_cell_candidates = candidates;
      }
    }

    Some(Step {
      index: best_cell_index,
      candidates: best_cell_candidates,
    })
  }

  fn try_backtrack(grid: &mut utils::Cells, steps: &mut Vec<Step>) -> bool {
    loop {
      match steps.pop() {
        Some(mut step) => match step.candidates.pop() {
          Some(candidate) => {
            grid[step.index] = candidate;
            steps.push(step);

            break true;
          }

          None => grid[step.index] = 0,
        },

        None => break false,
      }
    }
  }
}

mod utils {
  pub type Cells = [u8; 81];

  pub fn verify(grid: &Cells) -> bool {
    for i in 0..81 {
      if grid[i] == 0 {
        continue;
      }

      if !validate_candidate(&grid, i, grid[i], true) {
        return false;
      }
    }
    true
  }

  pub fn get_candidates(grid: &Cells, index: usize) -> Vec<u8> {
    let mut candidates: Vec<u8> = Vec::new();

    for i in 1..10 {
      if validate_candidate(grid, index, i, false) {
        candidates.push(i);
      }
    }

    candidates
  }

  fn validate_candidate(grid: &Cells, index: usize, value: u8, exclude_index: bool) -> bool {
    // NOTE - Check column.

    let column_index = index % 9;

    for i in 0..9 {
      let idx = 9 * i + column_index;

      if exclude_index && idx == index {
        continue;
      }

      if grid[idx] == value {
        return false;
      }
    }

    // NOTE - Check row.

    let row_index = index / 9;

    for j in 0..9 {
      let idx = 9 * row_index + j;

      if exclude_index && idx == index {
        continue;
      }

      if grid[idx] == value {
        return false;
      }
    }

    // NOTE - Check box.

    let band_index = row_index / 3;
    let stack_index = column_index / 3;

    for k in 0..9 {
      let idx = 3 * (9 * band_index + stack_index) + (9 * (k / 3) + (k % 3));

      if exclude_index && idx == index {
        continue;
      }

      if grid[idx] == value {
        return false;
      }
    }

    true
  }
}
