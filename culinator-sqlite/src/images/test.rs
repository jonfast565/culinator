use super::*;
use culinator_models::{NewRecipe, RecipeRepository};

fn make_recipe(repository: &SqliteCatalogRepository) -> Uuid {
    repository
        .create_recipe(NewRecipe {
            book_id: None,
            symbol: "img_recipe".to_owned(),
            title: "Img Recipe".to_owned(),
            protocol_version: "0.3".to_owned(),
            source_text: "culinator 0.3;\nrecipe img_recipe { title \"Img Recipe\"; }".to_owned(),
        })
        .expect("recipe creation")
        .id
}

#[test]
fn uploads_gets_lists_upserts_and_deletes() {
    let path = std::env::temp_dir().join(format!("culinator-img-{}.sqlite3", Uuid::new_v4()));
    let repository = SqliteCatalogRepository::new(&path);
    repository.initialize().expect("database initialization");
    let recipe_id = make_recipe(&repository);

    let asset = repository
        .upload_recipe_image(
            recipe_id,
            UploadRecipeImageRequest {
                handle: Some("img_cover_test".to_owned()),
                role: "cover".to_owned(),
                operation_symbol: None,
                media_type: "image/png".to_owned(),
                file_name: Some("cover.png".to_owned()),
                data_base64: "aGVsbG8=".to_owned(),
            },
        )
        .expect("upload");
    assert_eq!(asset.handle, "img_cover_test");
    assert_eq!(asset.role, "cover");

    let fetched = repository
        .get_recipe_image(recipe_id, "img_cover_test")
        .expect("get")
        .expect("present");
    assert_eq!(fetched.data_base64, "aGVsbG8=");
    assert_eq!(fetched.asset.media_type, "image/png");

    assert_eq!(
        repository
            .list_recipe_images(recipe_id)
            .expect("list")
            .len(),
        1
    );

    // Uploading the same handle upserts rather than duplicating.
    repository
        .upload_recipe_image(
            recipe_id,
            UploadRecipeImageRequest {
                handle: Some("img_cover_test".to_owned()),
                role: "cover".to_owned(),
                operation_symbol: None,
                media_type: "image/jpeg".to_owned(),
                file_name: None,
                data_base64: "d29ybGQ=".to_owned(),
            },
        )
        .expect("upsert");
    assert_eq!(
        repository
            .list_recipe_images(recipe_id)
            .expect("list2")
            .len(),
        1
    );
    let refetched = repository
        .get_recipe_image(recipe_id, "img_cover_test")
        .expect("get2")
        .expect("present2");
    assert_eq!(refetched.data_base64, "d29ybGQ=");
    assert_eq!(refetched.asset.media_type, "image/jpeg");

    // A missing handle is generated on upload.
    let generated = repository
        .upload_recipe_image(
            recipe_id,
            UploadRecipeImageRequest {
                handle: None,
                role: "step".to_owned(),
                operation_symbol: Some("cook".to_owned()),
                media_type: "image/png".to_owned(),
                file_name: None,
                data_base64: "aGk=".to_owned(),
            },
        )
        .expect("generated upload");
    assert!(generated.handle.starts_with("img_step_"));
    assert_eq!(generated.operation_symbol.as_deref(), Some("cook"));
    assert_eq!(
        repository
            .list_recipe_images(recipe_id)
            .expect("list3")
            .len(),
        2
    );

    assert!(
        repository
            .delete_recipe_image(recipe_id, "img_cover_test")
            .expect("delete")
    );
    assert!(
        repository
            .get_recipe_image(recipe_id, "img_cover_test")
            .expect("get3")
            .is_none()
    );

    let _ = std::fs::remove_file(path);
}
