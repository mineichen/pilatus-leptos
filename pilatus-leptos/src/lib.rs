use std::borrow::Cow;

use leptos::prelude::*;

mod recipe;

pub use recipe::*;
use thaw::Button;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[server]
pub async fn add_todo(title: String) -> Result<(), ServerFnError> {
    println!("Before {title}");

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    if std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % 2
        == 0
    {
        println!("After Ok");
        Ok(())
    } else {
        println!("After Error");
        Err(ServerFnError::Response("Number was not even".into()))
    }
}

#[component]
pub fn BusyButton() -> impl IntoView {
    let action = Action::new_local(|_: &String| add_todo("So much to do!".to_string()));

    view! {
        <Button on:click=move |_| {
            action.dispatch("input".into());
        }>
            "Add Todo"
        </Button>
        { move|| if action.pending().get() { Cow::Borrowed( "Pending") } else { match  action.value().read().as_ref() {
            Some(Err(e)) => format!("Error: {e}").into(),
            _ => "Ok".into()
        } } }
    }
}
