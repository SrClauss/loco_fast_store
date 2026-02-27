use axum::extract::Multipart;
use axum::extract::Query;
use axum::http::header;
use loco_rs::prelude::*;
use std::io::{Cursor, Write};
use uuid::Uuid;

use crate::{
    dto::{
        entities::{PriceResponse, ProductResponse, VariantResponse},
        response::ApiResponse,
    },
    models::{
        _entities::users,
        product_variants::{CreateVariantParams, Model as VariantModel},
        products::{CreateProductParams, ProductListParams, UpdateProductParams},
    },
};

/// POST /api/v1/products - Cria um produto
#[debug_handler]
async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateProductParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_admin(&user).await?;
    let product = crate::models::products::Model::create_product(&ctx.db, &params).await?;
    format::json(ApiResponse::success(ProductResponse::from(product)))
}

/// GET /api/v1/products - Lista produtos
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    Query(params): Query<ProductListParams>,
) -> Result<Response> {
    let products = crate::models::products::Model::list_for_store(&ctx.db, &params).await?;

    let limit = params.limit.unwrap_or(20).min(100);
    let has_more = products.len() as u64 >= limit;
    let cursor = products.last().map(|p| p.id.to_string());
    let count = products.len();

    let response: Vec<ProductResponse> = products.into_iter().map(ProductResponse::from).collect();
    format::json(ApiResponse::paginated(response, cursor, has_more, count))
}

/// GET /api/v1/products/:pid - Busca produto detalhado (com variantes e preços)
#[debug_handler]
async fn get_one(State(ctx): State<AppContext>, Path(pid): Path<Uuid>) -> Result<Response> {
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;

    // Carrega variantes com preços
    let variants = VariantModel::find_by_product(&ctx.db, product.id).await?;
    let mut variant_responses = Vec::new();
    for v in variants {
        let prices = VariantModel::get_prices(&ctx.db, v.id).await?;
        let price_responses: Vec<PriceResponse> =
            prices.into_iter().map(PriceResponse::from).collect();
        let mut vr = VariantResponse::from(v);
        vr.prices = Some(price_responses);
        variant_responses.push(vr);
    }

    let mut response = ProductResponse::from(product);
    response.variants = Some(variant_responses);

    format::json(ApiResponse::success(response))
}

/// PUT /api/v1/products/:pid - Atualiza produto
#[debug_handler]
async fn update(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
    Json(params): Json<UpdateProductParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_admin(&user).await?;
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;

    let mut active: crate::models::_entities::products::ActiveModel = product.into();
    if let Some(title) = params.title {
        active.title = ActiveValue::set(title);
    }
    if let Some(slug) = params.slug {
        active.slug = ActiveValue::set(slug);
    }
    if let Some(description) = params.description {
        active.description = ActiveValue::set(description);
    }
    if let Some(status) = params.status {
        active.status = ActiveValue::set(status);
    }
    if let Some(product_type) = params.product_type {
        active.product_type = ActiveValue::set(product_type);
    }
    if let Some(category_id) = params.category_id {
        active.category_id = ActiveValue::set(Some(category_id));
    }
    if let Some(tags) = params.tags {
        active.tags = ActiveValue::set(serde_json::json!(tags));
    }
    if let Some(featured) = params.featured {
        active.featured = ActiveValue::set(featured);
    }
    if let Some(metadata) = params.metadata {
        active.metadata = ActiveValue::set(metadata);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(ApiResponse::success(ProductResponse::from(updated)))
}

/// DELETE /api/v1/products/:pid - Soft delete
#[debug_handler]
async fn remove(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_admin(&user).await?;
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;

    let mut active: crate::models::_entities::products::ActiveModel = product.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    active.update(&ctx.db).await?;

    format::json(ApiResponse::<()>::success(()))
}

/// POST /api/v1/products/:pid/variants - Cria variante
#[debug_handler]
async fn create_variant(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(pid): Path<Uuid>,
    Json(params): Json<CreateVariantParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    crate::controllers::guards::ensure_admin(&user).await?;
    let product = crate::models::products::Model::find_by_pid(&ctx.db, &pid).await?;
    let variant = VariantModel::create_variant(&ctx.db, product.id, &params).await?;
    format::json(ApiResponse::success(VariantResponse::from(variant)))
}

/// GET /api/v1/products/export/csv - Exporta todos os produtos em CSV
#[debug_handler]
async fn export_csv(State(ctx): State<AppContext>) -> Result<Response> {
    let products = crate::models::products::Model::list_all_for_store(&ctx.db).await?;

    let mut wtr = csv::Writer::from_writer(Vec::new());
    // Cabeçalho
    wtr.write_record(&[
        "id",
        "pid",
        "title",
        "slug",
        "description",
        "handle",
        "status",
        "product_type",
        "category_id",
        "tags",
        "seo_title",
        "seo_description",
        "weight",
        "featured",
        "created_at",
    ])
    .map_err(|e| Error::string(&e.to_string()))?;

    for p in &products {
        wtr.write_record(&[
            p.id.to_string(),
            p.pid.to_string(),
            p.title.clone(),
            p.slug.clone(),
            p.description.clone(),
            p.handle.clone(),
            p.status.clone(),
            p.product_type.clone(),
            p.category_id.map(|c| c.to_string()).unwrap_or_default(),
            p.tags.to_string(),
            p.seo_title.clone().unwrap_or_default(),
            p.seo_description.clone().unwrap_or_default(),
            p.weight.map(|w| w.to_string()).unwrap_or_default(),
            p.featured.to_string(),
            p.created_at.to_rfc3339(),
        ])
        .map_err(|e| Error::string(&e.to_string()))?;
    }

    let csv_bytes = wtr
        .into_inner()
        .map_err(|e| Error::string(&e.to_string()))?;

    Ok(axum::response::Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"products.csv\"",
        )
        .body(axum::body::Body::from(csv_bytes))
        .map_err(|e| Error::string(&e.to_string()))?)
}

/// GET /api/v1/products/import/template - Gera CSV template para importação em lote
#[debug_handler]
async fn import_template(State(_ctx): State<AppContext>) -> Result<Response> {
    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record(&[
        "title",
        "slug",
        "description",
        "handle",
        "product_type",
        "category_id",
        "tags",
        "seo_title",
        "seo_description",
        "weight",
        "featured",
    ])
    .map_err(|e| Error::string(&e.to_string()))?;
    // Linha de exemplo
    wtr.write_record(&[
        "Meu Produto",
        "meu-produto",
        "Descrição do produto",
        "meu-produto",
        "physical",
        "",
        "tag1,tag2",
        "",
        "",
        "0.5",
        "false",
    ])
    .map_err(|e| Error::string(&e.to_string()))?;

    let csv_bytes = wtr
        .into_inner()
        .map_err(|e| Error::string(&e.to_string()))?;

    Ok(axum::response::Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"produtos_template.csv\"",
        )
        .body(axum::body::Body::from(csv_bytes))
        .map_err(|e| Error::string(&e.to_string()))?)
}

/// POST /api/v1/products/import/csv - Importa produtos em lote via CSV
/// Retorna ZIP com pastas para cada produto (nomeadas pelo slug)
#[debug_handler]
async fn import_csv(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    // Lê o arquivo CSV do multipart
    let mut csv_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::string(&e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "csv" {
            csv_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| Error::string(&e.to_string()))?
                    .to_vec(),
            );
            break;
        }
    }

    let csv_bytes =
        csv_bytes.ok_or_else(|| Error::string("Arquivo CSV não encontrado no multipart"))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(csv_bytes));

    let headers = rdr
        .headers()
        .map_err(|e| Error::string(&e.to_string()))?
        .clone();

    let mut created_slugs: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for (i, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("Linha {}: {}", i + 2, e));
                continue;
            }
        };

        let get = |col: &str| -> String {
            headers
                .iter()
                .position(|h| h == col)
                .and_then(|idx| record.get(idx))
                .unwrap_or("")
                .to_string()
        };

        let title = get("title");
        if title.is_empty() {
            errors.push(format!("Linha {}: título obrigatório", i + 2));
            continue;
        }

        let tags_str = get("tags");
        let tags: Vec<String> = if tags_str.is_empty() {
            vec![]
        } else {
            tags_str.split(',').map(|s| s.trim().to_string()).collect()
        };

        let weight: Option<f64> = get("weight").parse().ok();
        let featured: bool = get("featured").to_lowercase() == "true";
        let category_id: Option<i32> = get("category_id").parse().ok();

        let params = CreateProductParams {
            title: title.clone(),
            slug: {
                let s = get("slug");
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            description: {
                let s = get("description");
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            handle: {
                let s = get("handle");
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            product_type: {
                let s = get("product_type");
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            category_id,
            tags: if tags.is_empty() { None } else { Some(tags) },
            seo_title: {
                let s = get("seo_title");
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            seo_description: {
                let s = get("seo_description");
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            },
            weight,
            featured: Some(featured),
            metadata: None,
        };

        match crate::models::products::Model::create_product(&ctx.db, &params).await {
            Ok(product) => created_slugs.push(product.slug),
            Err(e) => errors.push(format!("Linha {}: {} - {}", i + 2, title, e)),
        }
    }

    // Gera ZIP com estrutura de pastas (uma por slug)
    let zip_bytes = build_slugs_zip(&created_slugs).map_err(|e| Error::string(&e.to_string()))?;

    let result = serde_json::json!({
        "created": created_slugs.len(),
        "slugs": created_slugs,
        "errors": errors,
    });

    // Retorna o ZIP para o cliente fazer download da estrutura de pastas
    Ok(axum::response::Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"produtos_imagens.zip\"",
        )
        .header(
            "X-Import-Result",
            serde_json::to_string(&result).unwrap_or_default(),
        )
        .body(axum::body::Body::from(zip_bytes))
        .map_err(|e| Error::string(&e.to_string()))?)
}

/// POST /api/v1/products/import/images - Upload do ZIP com imagens
/// O ZIP deve ter subpastas nomeadas com o slug do produto, contendo as imagens
#[debug_handler]
async fn import_images(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    let _user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    // Lê o arquivo ZIP do multipart
    let mut zip_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::string(&e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "zip" {
            zip_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| Error::string(&e.to_string()))?
                    .to_vec(),
            );
            break;
        }
    }

    let zip_bytes =
        zip_bytes.ok_or_else(|| Error::string("Arquivo ZIP não encontrado no multipart"))?;

    let mut archive =
        zip::ZipArchive::new(Cursor::new(zip_bytes)).map_err(|e| Error::string(&e.to_string()))?;

    // Coleta todos os dados em memória ANTES dos awaits
    // (ZipFile não é Send, não pode cruzar pontos .await)
    struct FileEntry {
        slug: String,
        filename: String,
        data: Vec<u8>,
    }
    let mut entries: Vec<FileEntry> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| Error::string(&e.to_string()))?;
        let file_path = file.mangled_name();
        let parts: Vec<&str> = file_path.to_str().unwrap_or("").split('/').collect();
        if parts.len() < 2 || file.is_dir() {
            continue;
        }
        let slug = parts[0].to_string();
        let filename = parts[parts.len() - 1].to_string();
        if filename.starts_with('.') || slug.starts_with('.') {
            continue;
        }
        let mut data = Vec::new();
        std::io::copy(&mut file, &mut data).map_err(|e| Error::string(&e.to_string()))?;
        entries.push(FileEntry {
            slug,
            filename,
            data,
        });
    }
    drop(archive); // libera antes dos awaits

    let mut processed: Vec<serde_json::Value> = Vec::new();

    for entry in entries {
        let FileEntry {
            slug,
            filename,
            data,
        } = entry;

        // Busca o produto pelo slug
        use crate::models::_entities::products;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let product = products::Entity::find()
            .filter(products::Column::Slug.eq(slug.as_str()))
            .filter(products::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?;

        if let Some(product) = product {
            // Salva a imagem usando o storage do loco-rs
            let storage_path = format!("products/{}/{}", slug, filename);
            let image_url = {
                use std::path::Path;
                let bytes = axum::body::Bytes::from(data);
                let _ = ctx.storage.upload(Path::new(&storage_path), &bytes).await;
                format!("/storage/{}", storage_path)
            };

            // Registra a imagem no banco
            use crate::models::_entities::product_images;
            let _ = product_images::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                product_id: ActiveValue::set(product.id),
                variant_id: ActiveValue::set(None),
                url: ActiveValue::set(image_url.clone()),
                alt_text: ActiveValue::set(product.title.clone()),
                sort_order: ActiveValue::set(0),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?;

            processed.push(serde_json::json!({
                "slug": slug,
                "file": filename,
                "url": image_url,
            }));
        }
    }

    format::json(ApiResponse::success(serde_json::json!({
        "processed": processed.len(),
        "images": processed,
    })))
}

/// Constrói um ZIP com uma pasta vazia para cada slug
fn build_slugs_zip(slugs: &[String]) -> std::io::Result<Vec<u8>> {
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for slug in slugs {
        // Cria uma pasta para o slug com um arquivo README
        let entry_name = format!("{}/COLOQUE_IMAGENS_AQUI.txt", slug);
        zip.start_file(entry_name, options)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let readme = format!(
            "Coloque as imagens do produto '{}' nesta pasta.\nFormatos suportados: jpg, jpeg, png, webp, gif\n",
            slug
        );
        zip.write_all(readme.as_bytes())?;
    }

    let cursor = zip
        .finish()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(cursor.into_inner())
}

/// GET /api/admin/products - Lista todos produtos (admin)
#[debug_handler]
async fn admin_list(State(ctx): State<AppContext>) -> Result<Response> {
    use crate::models::_entities::products;
    use sea_orm::EntityTrait;

    let products = products::Entity::find().all(&ctx.db).await?;
    let response: Vec<ProductResponse> = products.into_iter().map(ProductResponse::from).collect();
    format::json(ApiResponse::success(response))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/products")
        .add("/", post(create))
        .add("/", get(list))
        .add("/export/csv", get(export_csv))
        .add("/import/template", get(import_template))
        .add("/import/csv", post(import_csv))
        .add("/import/images", post(import_images))
        .add("/{pid}", get(get_one))
        .add("/{pid}", put(update))
        .add("/{pid}", delete(remove))
        .add("/{pid}/variants", post(create_variant))
}

pub fn admin_routes() -> Routes {
    Routes::new()
        .prefix("/api/admin")
        .add("/products", get(admin_list))
}
