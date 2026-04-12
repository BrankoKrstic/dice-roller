use leptos::prelude::*;

pub const DEFAULT_PAGE_TITLE: &str = "Local Ledger";
pub const ROOMS_PAGE_TITLE: &str = "Rooms";
pub const NOT_FOUND_PAGE_TITLE: &str = "Not Found";

#[derive(Clone, Copy)]
pub struct PageTitleContext {
    title: RwSignal<String>,
}

impl PageTitleContext {
    pub fn get(&self) -> String {
        self.title.get()
    }

    pub fn set(&self, title: impl Into<String>) {
        self.title.set(title.into());
    }
}

pub fn format_document_title(page_title: &str) -> String {
    format!("Dice Roller | {page_title}")
}

pub fn provide_page_title_context() {
    provide_context(PageTitleContext {
        title: RwSignal::new(DEFAULT_PAGE_TITLE.to_string()),
    });
}

pub fn use_page_title_context() -> PageTitleContext {
    use_context::<PageTitleContext>().unwrap_or(PageTitleContext {
        title: RwSignal::new(DEFAULT_PAGE_TITLE.to_string()),
    })
}

pub fn use_static_page_title(title: &'static str) {
    let page_title = use_page_title_context();

    page_title.set(title);
}

#[cfg(test)]
mod tests {
    use super::format_document_title;

    #[test]
    fn formats_document_title_with_brand_prefix() {
        assert_eq!(
            format_document_title("Simulation"),
            "Dice Roller | Simulation",
        );
    }
}
