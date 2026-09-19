use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/invoices/:id/pdf", get(invoice_pdf));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on http://{addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn invoice_pdf(Path(id): Path<String>) -> impl IntoResponse {
    let bytes = build_invoice_pdf(&id);
    ([("Content-Type", "application/pdf")], bytes)
}

/// Builds a one-page A4 invoice PDF, entirely in memory, with no headless
/// browser and no external renderer involved.
fn build_invoice_pdf(id: &str) -> Vec<u8> {
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let font_id = Ref::new(4);
    let content_id = Ref::new(5);
    let font_name = Name(b"F1");

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    let mut page = pdf.page(page_id);
    page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0)); // A4 in points
    page.parent(page_tree_id);
    page.contents(content_id);
    page.resources().fonts().pair(font_name, font_id);
    page.finish();

    // Helvetica is one of the 14 standard PDF fonts, so no font file
    // needs to be embedded or shipped with the binary.
    pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

    let title = format!("Invoice #{id}");
    let lines = [
        (title.as_str(), 20.0),
        ("Billed to: Acme Corp", 12.0),
        ("", 12.0),
        ("1x Consulting Services ......... $1,200.00", 12.0),
        ("", 12.0),
        ("Total: $1,200.00", 14.0),
    ];

    let mut content = Content::new();
    content.begin_text();
    content.set_font(font_name, 20.0);
    content.next_line(56.0, 780.0); // top-left start position, in points

    for (text, size) in lines {
        content.set_font(font_name, size);
        if !text.is_empty() {
            content.show(Str(text.as_bytes()));
        }
        content.next_line(0.0, -28.0); // move down for the next line
    }
    content.end_text();
    pdf.stream(content_id, &content.finish());

    pdf.finish()
}
