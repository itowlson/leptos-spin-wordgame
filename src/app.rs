use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let bot_tiles_coll: Vec<_> = ['A', 'B', 'C', 'D', 'E'].into_iter().map(|ch| create_signal(ch)).collect();
    let (bot_tiles, set_bot_tiles) = create_signal(bot_tiles_coll);
    let (in_progress, set_in_progress) = create_signal("FIE".to_owned());
    let (secs_remaining, set_secs_remaining) = create_signal(10);
    let (score, _set_score) = create_signal(0);

    // // Creates a reactive value to update the button
    // let (count, set_count) = create_signal(0);
    // let on_click = move |_| {
    //     request_animation_frame(move || { // !!! IMPORTANT !!! You must use request_enimation_frame
    //         set_count.update(|count| *count += 1);
    //         spawn_local(async move {
    //             save_count(count.get()).await.unwrap(); // YOLO
    //         });
    //     });
    // };
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
                leptos::logging::log!("APPENDING! {ch}");
                set_in_progress.update(|w| *w = format!("{w}{}", ch.to_uppercase()));
                // fake bot activity
                set_bot_tiles.update(|tiles| {
                    (*tiles)[3].1.update(|ch| *ch = random_tile());
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
