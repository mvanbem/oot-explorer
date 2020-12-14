fn main() {
    let file_data =
        std::fs::read("Legend of Zelda, The - Ocarina Of Time (U) (V1.0) [!].z64").unwrap();

    oot_explorer_web::process_all_scenes(&file_data);
}
