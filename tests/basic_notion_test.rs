use notation::notion::block::{AppendBlockRequest, AppendBlockRequestChild};
use notation::notion::client::NotionClient;

#[tokio::test(flavor = "multi_thread")]
async fn test_create_page() {
    let nc = NotionClient::new().unwrap();
    let nid = nc
        .create_page_by_parent_name(
            nc.parent_page_name(),
            "Some Other Page".to_string(),
            Some("🥵".to_string()),
        )
        .await
        .unwrap();
    println!("Created page with id: {}", nid);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_append_block() {
    let nc = NotionClient::new().unwrap();
    let page_id = nc
        .find_page_by_name("Some Other Page".to_string())
        .await
        .unwrap()[0]
        .clone()
        .id;
    let header_request =
        AppendBlockRequestChild::new_heading_block("This is a heading".to_string(), 1);
    let paragraph_request =
        AppendBlockRequestChild::new_paragraph_block("This is a paragraph".to_string());
    nc.append_block(
        page_id,
        &AppendBlockRequest::new_children(vec![header_request, paragraph_request]),
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_clear() {
    let nc = NotionClient::new().unwrap();
    nc.clear().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_pages() {
    let nc = NotionClient::new().unwrap();
    nc.create_pages("samples_md/small_example/".to_string(), false)
        .await
        .unwrap();
}
