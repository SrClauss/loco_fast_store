use async_trait::async_trait;
use axum::{Extension, Router as AxumRouter};
use fluent_templates::{ArcLoader, FluentLoader};
use loco_rs::{
    app::{AppContext, Initializer},
    controller::views::{engines, ViewEngine},
    Error, Result,
};
use tracing::info;

const I18N_DIR: &str = "assets/i18n";
const I18N_SHARED: &str = "assets/i18n/shared.ftl";
#[allow(clippy::module_name_repetitions)]
pub struct ViewEngineInitializer;

#[async_trait]
impl Initializer for ViewEngineInitializer {
    fn name(&self) -> String {
        "view-engine".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let tera_engine = if std::path::Path::new(I18N_DIR).exists() {
            // try to build loader, but don't fail startup if i18n files are missing or broken
            match ArcLoader::builder(&I18N_DIR, unic_langid::langid!("en-US"))
                .shared_resources(Some(&[I18N_SHARED.into()]))
                .customize(|bundle| bundle.set_use_isolating(false))
                .build()
            {
                Ok(loader) => {
                    let arc = std::sync::Arc::new(loader);
                    info!("locales loaded");

                    engines::TeraView::build()?.post_process(move |tera| {
                        tera.register_function("t", FluentLoader::new(arc.clone()));
                        Ok(())
                    })?
                }
                Err(e) => {
                    tracing::warn!("i18n initialization failed: {}", e);
                    engines::TeraView::build()?
                }
            }
        } else {
            engines::TeraView::build()?
        };

        Ok(router.layer(Extension(ViewEngine::from(tera_engine))))
    }
}
