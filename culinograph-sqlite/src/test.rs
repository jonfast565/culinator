use super::*;
use culinograph_core::RecipeBook;
use rusqlite::Connection;

#[test] fn migrations_apply_to_memory_database() { let connection=Connection::open_in_memory().expect("open"); migrate(&connection).expect("migrate"); let version:i64=connection.pragma_query_value(None,"user_version",|r|r.get(0)).expect("version"); assert!(version>=5); }
#[test] fn recipe_book_round_trip() { let mut connection=Connection::open_in_memory().expect("open"); migrate(&connection).expect("migrate"); let book=RecipeBook::empty("favorites","Favorites","0.3"); save_recipe_book(&mut connection,&book).expect("save"); let books=list_recipe_books(&connection).expect("list"); assert_eq!(books.len(),1); assert_eq!(books[0].title,"Favorites"); }
