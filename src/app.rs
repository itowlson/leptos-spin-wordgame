use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Serialize, Deserialize};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        <Stylesheet id="leptos" href="/pkg/wordgame.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

const GAME_TIME_SECONDS: i32 = 60;
// const HUMAN_MAKES_WORD_BOOST: i32 = 2;
const COMPUTER_THWARTED_BOOST: i32 = 5;

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let bot_tiles_coll: Vec<_> = (0..5).into_iter().map(|_| create_signal(random_tile())).collect();
    let (bot_tiles, set_bot_tiles) = create_signal(bot_tiles_coll);

    let starter = random_tile().to_string();
    let (in_progress, set_in_progress) = create_signal(starter);

    let (secs_remaining, set_secs_remaining) = create_signal(GAME_TIME_SECONDS);
    let (score, set_score) = create_signal(0);

    create_effect(move |_| {
        set_interval(
            move || set_secs_remaining.update(|s| *s -= 1),
            std::time::Duration::from_secs(1)
        )
    });

    let handle = window_event_listener(ev::keypress, move |ev| {
        let k = ev.key();
        if k.len() == 1 {
            let ch = k.chars().next().unwrap();
            if ch.is_alphabetic() {
                spawn_local(async move {
                    let candidate = format!("{}{}", in_progress.get(), ch.to_uppercase());
                    let (is_prefix, is_word) = eval_word(candidate.clone()).await.unwrap();

                    if matches!(is_word, IsWord::Yes) && candidate.len() > 2 {
                        // set_secs_remaining.update(|s| *s += HUMAN_MAKES_WORD_BOOST);
                        logging::log!("Human completed {candidate}");  // TODO: make a fuss
                        set_score.update(|s| *s -= 1);
                        let starter = random_tile().to_string();
                        set_in_progress.update(|w| *w = starter);
                        return;
                    }
                    if matches!(is_prefix, IsPrefix::No) {
                        // TODO: show a barf
                        logging::log!("Human proposed {candidate} but then computer couldn't move");
                        return;
                    }

                    set_in_progress.update(|w| *w = candidate.clone());
    
                    loop {
                        let bot_tile_chars: Vec<_> = bot_tiles.get().iter().map(|s| s.0.get()).collect();
                        match pick_tile(in_progress.get(), bot_tile_chars.clone()).await.unwrap() {
                            PickTileResult::Complete { index, word } => {
                                logging::log!("Computer forced to complete {word}");  // TODO: make a fuss
                                set_score.update(|s| *s += word.len());
                                let starter = random_tile().to_string();
                                set_in_progress.update(|w| *w = starter);
                                set_bot_tiles.update(|tiles| {
                                    (*tiles)[index].1.update(|ch| *ch = random_tile());
                                });
                                break;
                            },
                            PickTileResult::Extend { index, partial, witness } => {
                                logging::log!("Computer move legitimated by {witness}");
                                set_in_progress.update(|w| *w = partial);
                                set_bot_tiles.update(|tiles| {
                                    (*tiles)[index].1.update(|ch| *ch = random_tile());
                                });
                                break;
                            },
                            PickTileResult::CannotExtend => {
                                logging::log!("Computer cannot move!  Tossing tiles {bot_tile_chars:?}");
                                set_secs_remaining.update(|s| *s += COMPUTER_THWARTED_BOOST);
                                for index in 0..4 {
                                    set_bot_tiles.update(|tiles| {
                                        (*tiles)[index].1.update(|ch| *ch = random_tile());
                                    });
                                }
                            }
                        };
                    }
                });
            }
        }
    });
    on_cleanup(move || handle.remove());

    view! {
        <div>
            <h1><TimeInfo secs_remaining={secs_remaining} /> " | Score: " {score} </h1>
            <BotTiles tiles={bot_tiles} />
            <p />
            <Word word={in_progress} />
        </div>
    }
}

#[component]
fn TimeInfo(secs_remaining: ReadSignal<i32>) -> impl IntoView {
    view! {
        <Show
            when=move || { secs_remaining.get() > 0 }
            fallback=|| view! { <span>"Game over"</span> }
        >
            <span>"Time: " {secs_remaining} " seconds"</span>
        </Show>
    }
}

#[component]
fn BotTiles(tiles: ReadSignal<Vec<(ReadSignal<char>, WriteSignal<char>)>>) -> impl IntoView {
    view! {
        <div>
            { tiles.get().iter().map(|c| view! { <Tile letter={c.0} /> }).collect_view() }
        </div>
    }
}

#[component]
fn Word(word: ReadSignal<String>) -> impl IntoView {
    view! {
        <div><strong>{word}</strong></div>
        // <div>
        //     { word.get().chars().map(|c| view! { <Tile letter={c} /> }).collect_view() }
        // </div>
    }
}

#[component]
fn Tile(letter: ReadSignal<char>) -> impl IntoView {
    view! { <span>"[ " {letter} " ]"</span> }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_spin::ResponseOptions>();
        resp.set_status(404);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}

fn random_tile() -> char {
    use rand::distributions::Distribution;
    let ascii_upper = rand::distributions::Uniform::new_inclusive('A', 'Z');
    ascii_upper.sample(&mut rand::thread_rng())
}

// // for testing purposes
// fn random_tile_index() -> usize {
//     use rand::distributions::Distribution;
//     let indexes = rand::distributions::Uniform::new_inclusive(0, 4);
//     indexes.sample(&mut rand::thread_rng())
// }

// fn random_vowel() -> char {
//     use rand::distributions::Distribution;
//     let vowels_slice: Vec<_> = "AEIOU".chars().collect();
//     let vowels = rand::distributions::Slice::new(&vowels_slice).unwrap();
//     *vowels.sample(&mut rand::thread_rng())
// }

#[derive(Serialize, Deserialize)]
pub enum IsPrefix {
    Yes(String),  // could complete to a word (with witness)
    No,  // could never complete to a word
}

#[derive(Serialize, Deserialize)]
pub enum IsWord {
    Yes,
    No,
}

#[server(EvalWord, "/api")]
pub async fn eval_word(candidate: String) -> Result<(IsPrefix, IsWord), ServerFnError> {
    let candidate = candidate.to_uppercase();
    let candidate_len = candidate.len();

    let words = std::fs::read_to_string("/words.txt")?;

    // This is quite startlingly inefficient!

    let is_prefix = match words.lines().find(|w| w.len() > candidate_len && w.to_uppercase().starts_with(&candidate)) {
        None => IsPrefix::No,
        Some(w) => IsPrefix::Yes(w.to_owned()),
    };

    let is_word = if words.lines().any(|w| w.to_uppercase() == candidate) {
        IsWord::Yes
    } else {
        IsWord::No
    };

    Ok((is_prefix, is_word))
}

#[derive(Serialize, Deserialize)]
pub enum PickTileResult {
    Complete { index: usize, word: String },
    Extend { index: usize, partial: String, witness: String },
    CannotExtend,
}

#[server(PickTile, "/api")]
async fn pick_tile(current: String, available_tiles: Vec<char>) -> Result<PickTileResult, ServerFnError> {
    let current = current.to_uppercase();
    let words = std::fs::read_to_string("/words.txt")?;

    let candidates = (0..available_tiles.len()).map(|index| {
        let ch = available_tiles[index].to_uppercase();
        let prefix = format!("{current}{ch}");
        let prefix_len = prefix.len();
        let is_word = words.lines().any(|w| w.to_uppercase() == prefix);
        let longer = words.lines().find(|w| w.len() > prefix_len && w.to_uppercase().starts_with(&prefix));
        match (longer, is_word) {
            (_, true) => PickTileResult::Complete { index, word: prefix },
            (Some(word), _) => PickTileResult::Extend { index, partial: prefix, witness: word.to_owned() },
            (None, false) => PickTileResult::CannotExtend,
        }
    }).collect::<Vec<_>>();

    let (prefixes, rest): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|c| matches!(c, PickTileResult::Extend { .. }));
    if prefixes.is_empty() {
        let (_, exact): (Vec<_>, Vec<_>) = rest.into_iter().partition(|c| matches!(c, PickTileResult::CannotExtend));
        if exact.is_empty() {
            Ok(PickTileResult::CannotExtend)
        } else {
            Ok(exact.into_iter().next().unwrap())
        }
    } else {
        Ok(prefixes.into_iter().next().unwrap())
    }

    // if let Some(index) = candidates.iter().position(|c| matches!(c, CandidateResult::Prefix { prefix, witness })) {
    //     Ok(PickTileResult::Extend { index, partial: prefix, witness })
    // } else if let Some(index) = candidates.iter().position(|c| matches!(c, CandidateResult::Word(word))) {
    //     Ok(PickTileResult::Complete { index, word })
    // } else {
    //     Ok(PickTileResult::CannotExtend)
    // }
}
