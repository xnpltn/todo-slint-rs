use slint::*;
slint::include_modules!();
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use surrealdb::engine::local::Mem;
use surrealdb::sql::{Id, Thing};
use surrealdb::Surreal;

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
    title: String,
    completed: bool,
}

#[derive(Debug, Serialize)]
struct TodoInput {
    completed: bool,
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main = Main::new()?;
    let db = Rc::from(Surreal::new::<Mem>(()).await?);
    db.use_ns("todo").use_db("todo").await?;

    let todos_model = Rc::new(VecModel::from(
        main.get_todos().iter().collect::<Vec<Todo>>(),
    ));

    main.on_add_todo({
        let db = Rc::clone(&db);
        let weak = main.as_weak();
        move |new| {
            let todos = todos_model.clone();
            let weak = weak.clone();
            let db = db.clone();
            let _ = slint::spawn_local(async move {
                let b: Result<Vec<Record>, surrealdb::Error> = db
                    .create("todo")
                    .content(TodoInput {
                        title: new.to_string(),
                        completed: false,
                    })
                    .await;
                match b {
                    Ok(t) => {
                        if let Id::String(id) = &t[0].id.id {
                            if let Some(ui) = weak.upgrade() {
                                todos.push(Todo {
                                    id: id.into(),
                                    name: new,
                                    completed: false,
                                });
                                ui.set_todos(todos.clone().into());
                            }
                            println!("{t:?}");
                        }
                    }
                    Err(e) => {
                        println!("error adding todos \n: {e:?}")
                    }
                }
            });
        }
    });

    main.on_get_todos({
        let weak = main.as_weak();
        let db = db.clone();
        move || {
            let weak = weak.clone();
            let db = db.clone();
            let _ = slint::spawn_local(async move {
                let todos: Result<Vec<Record>, surrealdb::Error> = db.select("todo").await;
                match todos {
                    Ok(todos) => {
                        let todos: Vec<Todo> = todos
                            .into_iter()
                            .map(|todo| Todo {
                                id: todo.id.id.to_string().into(),
                                name: todo.title.into(),
                                completed: todo.completed,
                            })
                            .collect();
                        let todos = std::rc::Rc::from(VecModel::from(todos));

                        if let Some(ui) = weak.upgrade() {
                            ui.set_todos(todos.clone().into());
                        }
                    }
                    Err(e) => {
                        println!("error getting todos \n: {e:?}")
                    }
                }
            });
        }
    });

    main.run()?;
    Ok(())
}
