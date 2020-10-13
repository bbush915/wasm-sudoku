import { Puzzle } from "wasm-sudoku";
import { memory } from "wasm-sudoku/wasm_sudoku_bg.wasm";

import "./index.css";

function initializeGrid(): void {
  const grid = document.querySelector("#grid") as HTMLDivElement;

  for (let i = 0; i < 81; i++) {
    const cell: HTMLInputElement = document.createElement("input");

    cell.classList.add("cell");
    cell.type = "text";
    cell.inputMode = "numeric";
    cell.dataset.index = String(i);
    cell.oninput = updateCell;

    grid.appendChild(cell);
  }

  refreshGrid();
}

function updateCell(event: Event): void {
  const input = event.target as HTMLInputElement;
  const newValue = parseInt(input.value.substr(input.selectionStart! - 1, 1));

  if (!Number.isInteger(newValue) || newValue < 1 || newValue > 9) {
    const oldValue = input.value.length === 1 ? "" : input.value.substr(input.value.length - input.selectionStart!, 1);
    input.value = oldValue;
    return;
  } else {
    input.value = String(newValue);
  }

  cells[Number(input.dataset.index)] = Number(input.value);
}

function refreshGrid(): void {
  const inputs = document.querySelectorAll(".cell");

  for (let i = 0; i < inputs.length; i++) {
    const input = inputs[i] as HTMLInputElement;
    const index = Number(input.dataset.index);

    input.value = cells[index] ? String(cells[index]) : "";

    if (clueIndices.has(i)) {
      input.classList.add("clue");
      input.readOnly = true;
    } else {
      input.classList.remove("clue");
      input.readOnly = false;
    }
  }
}

function initializeOptions(): void {
  const generateButton = document.querySelector("#generate") as HTMLButtonElement;
  generateButton.onclick = generatePuzzle;

  const verifyButton = document.querySelector("#verify") as HTMLButtonElement;
  verifyButton.onclick = verifyPuzzle;

  const solveButton = document.querySelector("#solve") as HTMLButtonElement;
  solveButton.onclick = solvePuzzle;
}

function generatePuzzle(): void {
  puzzle.generate();

  initializeClues();
  refreshGrid();
}

function verifyPuzzle(): void {
  const result = puzzle.verify();

  let message = "Oops! There is a mistake somewhere.";
  let type = "error";

  if (result) {
    message = cells.every((x) => x !== 0) ? "Congratulations! You solved it." : "Looks good so far!";
    type = "success";
  }

  notificationDiv.firstChild!.textContent = message;

  notificationDiv.className = "";
  notificationDiv.classList.add("visible", type);

  setTimeout(() => {
    notificationDiv.className = "";
  }, 2500);
}

function solvePuzzle(): void {
  puzzle.solve();
  refreshGrid();
}

function initializeClues(): void {
  clueIndices = new Set([...cells].map((x, index) => (x === 0 ? null : index)).filter((x) => x != null) as number[]);
}

const puzzle = Puzzle.new();
const cells = new Uint8Array(memory.buffer, puzzle.cells(), 81);
let clueIndices: Set<number>;

const notificationDiv = document.querySelector("#notification")!;

initializeClues();

initializeGrid();
initializeOptions();
