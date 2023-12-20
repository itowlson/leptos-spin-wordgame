# Half Arsed Word Game in Leptos and Spin

## Rules

- The goal is to force the computer to complete as many words as possible in 60 seconds.
  - You score more for longer words.
  - You lose a point if you complete a word yourself.
- Each turn, hit a letter key to add that letter to the end of the current string.
  - If the result is an English word, you _lose_ a point. (Or, specifically, if the result is what the computer _thinks_ is an English word. It has some bloody funny ideas.)
  - Otherwise, your letter _must_ result in the prefix of some English word.  E.g. if the current string is `AQ`, you can't add a `Z` because no words begin with `AQZ`.  If not, your letter is rejected with no penalty.
- Then the computer tries to add a letter from its list of tiles.
  - If the result is an English word, you score the length of the word in points.
  - If the computer can't make a prefix using its tiles, it refreshes its tiles and tries again. You gain a time bonus every time it refreshes.
  - Otherwise, the computer replaces the tile it used, and it's back to your turn. (The word that the computer found to prove that it's a legit prefix is displayed below the play area, because the computer thinks of some extremely obscure and questionable words that I for one had no idea how to continue.)
- When either side completes a word, a new starter letter is displayed and it's the human's turn.
  - The alleged word is displayed below the play area.

## Known issues

- There is almost no feedback so it can be confusing whose turn it is. (The JS console can help here.)
- Turns are not properly enforced!
- I can't imagine it works on mobile
- No Start button. No rules display. Refresh to restart.
- The word list needs pruning, but without thwarting logophile humans. (Maybe the computer should only offer prefixes that can be completed in 2 or more ways?)
- Look basically it's all known issues loosely hung together on a gossamber thread of demo

## Try it on Fermyon Cloud

https://wordgame-wulxactm.fermyon.app/

## Build it yourself

Prequisites:

- Rust [with the `wasm32-wasi` and `wasm32-unknown-unknown` target](https://developer.fermyon.com/spin/v2/install) - `rustup target add wasm32-wasi` then `rustup target add wasm32-unknown-unknown`
- [Spin](https://developer.fermyon.com/spin/v2/install)
- [`cargo-leptos`](https://github.com/leptos-rs/cargo-leptos#getting-started) - `cargo install --locked cargo-leptos`

Build and run:

- `spin up --build` to build and run the server. It will print the application URL.

Acknowledgements:

- `words.txt` sourced from https://github.com/dwyl/english-words
