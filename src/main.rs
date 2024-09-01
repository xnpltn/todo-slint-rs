use slint::*;
slint::include_modules!();
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use surrealdb::engine::local::Mem;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
    title: String,
}

#[derive(Debug, Serialize)]
struct Todo {
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main = Main::new()?;
    let db = Rc::from(Surreal::new::<Mem>(()).await?);
    db.use_ns("test").use_db("test").await?;

    let todos_model = Rc::new(VecModel::from(
        main.get_todos().iter().collect::<Vec<SharedString>>(),
    ));

    let weak = main.as_weak();

    let db_1 = Rc::clone(&db);
    main.on_add_todo(move |new| {
        let todos = todos_model.clone();
        let weak = weak.clone();
        let db = db_1.clone();
        let _ = slint::spawn_local(async move {
            let _: Vec<Record> = db
                .create("todo")
                .content(Todo {
                    title: new.to_string(),
                })
                .await
                .unwrap();
            if let Some(ui) = weak.upgrade() {
                todos.push(new);
                ui.set_todos(todos.clone().into());
            }
        });
    });

    let db = Rc::clone(&db);
    let weak = main.as_weak();
    main.on_get_todos(move || {
        let db = db.clone();
        let weak = weak.clone();
        let _ = slint::spawn_local(async move {
            let todos: Vec<Record> = db.select("todo").await.unwrap();
            let todos: Vec<SharedString> =
                todos.into_iter().map(|todo| todo.title.into()).collect();
            let todos = std::rc::Rc::from(VecModel::from(todos));
            if let Some(ui) = weak.upgrade() {
                ui.set_todos(todos.clone().into());
            }
        });
    });

    main.run()?;
    Ok(())
}
