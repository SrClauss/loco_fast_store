# 🚀 Guia de Instalação e Configuração Rápida

## 📦 Instalação

### 1. Instalar Dependências

```bash
cd loco_fast_store

# Instalar dependências Node.js
npm install

# Instalar dependências Rust (adicionar ao Cargo.toml)
cargo add tera tower-http
```

### 2. Build do CSS

```bash
# Compilar TailwindCSS (produção)
npm run build:css

# OU em modo desenvolvimento (watch)
npm run dev
```

### 3. Configurar Tera no Backend

Editar `src/app.rs`:

```rust
use tera::Tera;
use tower_http::services::ServeDir;
use std::sync::Arc;

pub struct AppState {
    pub tera: Tera,
    // ... outros campos
}

impl AppState {
    pub fn new() -> Result<Self> {
        let mut tera = match Tera::new("assets/views/**/*.html.tera") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Template parsing error: {}", e);
                std::process::exit(1);
            }
        };
        
        // Auto-reload em desenvolvimento
        tera.autoescape_on(vec![".html"]);
        
        Ok(Self {
            tera,
            // ... inicializar outros campos
        })
    }
}
```

### 4. Criar Controllers de Views

Criar `src/controllers/admin_views.rs`:

```rust
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use crate::app::AppState;

// Dashboard
pub async fn dashboard(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("current_page", "dashboard");
    
    match state.tera.render("admin/dashboard.html.tera", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html("Error rendering template".to_string()).into_response()
        }
    }
}

// Login
pub async fn login(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let context = tera::Context::new();
    
    match state.tera.render("admin/login.html.tera", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html("Error rendering template".to_string()).into_response()
        }
    }
}

// Produtos - Lista
pub async fn products_list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("current_page", "products");
    
    match state.tera.render("admin/products/list.html.tera", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html("Error rendering template".to_string()).into_response()
        }
    }
}

// Produtos - Novo
pub async fn products_new(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("current_page", "products");
    
    match state.tera.render("admin/products/form.html.tera", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html("Error rendering template".to_string()).into_response()
        }
    }
}

// Pedidos - Lista
pub async fn orders_list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("current_page", "orders");
    
    match state.tera.render("admin/orders/list.html.tera", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html("Error rendering template".to_string()).into_response()
        }
    }
}

// Categorias - Lista
pub async fn categories_list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("current_page", "categories");
    
    match state.tera.render("admin/categories/list.html.tera", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html("Error rendering template".to_string()).into_response()
        }
    }
}

// Clientes - Lista
pub async fn customers_list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("current_page", "customers");
    
    match state.tera.render("admin/customers/list.html.tera", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template error: {}", e);
            Html("Error rendering template".to_string()).into_response()
        }
    }
}
```

### 5. Adicionar Rotas

Editar onde as rotas são configuradas (geralmente `src/app.rs` ou `src/bin/main.rs`):

```rust
use tower_http::services::ServeDir;

// Servir arquivos estáticos
let app = Router::new()
    // Rotas de views admin
    .route("/admin/login", get(controllers::admin_views::login))
    .route("/admin/dashboard", get(controllers::admin_views::dashboard))
    .route("/admin/products", get(controllers::admin_views::products_list))
    .route("/admin/products/new", get(controllers::admin_views::products_new))
    .route("/admin/orders", get(controllers::admin_views::orders_list))
    .route("/admin/categories", get(controllers::admin_views::categories_list))
    .route("/admin/customers", get(controllers::admin_views::customers_list))
    
    // Servir arquivos estáticos
    .nest_service("/static", ServeDir::new("assets/static"))
    
    // ... demais rotas API
    .with_state(Arc::new(state));
```

### 6. Criar Imagens Placeholder

Criar arquivos de imagem padrão:

```bash
# Criar diretório
mkdir -p assets/static/images

# Você pode adicionar:
# - logo.svg (logo da aplicação)
# - placeholder.png (imagem padrão para produtos sem foto)
# - favicon.svg (ícone do site)
```

### 7. Atualizar Layout Base

Editar `assets/views/layouts/base.html` para usar o CSS compilado:

```html
<!-- ANTES -->
<link rel="stylesheet" href="/static/css/app.css">

<!-- DEPOIS -->
<link rel="stylesheet" href="/static/css/output.css">
```

E no layout de autenticação também (`assets/views/layouts/auth.html`).

## 🧪 Testar

```bash
# Build CSS
npm run build:css

# Rodar servidor
cargo loco start

# Acessar no navegador
# http://localhost:5150/admin/login
# http://localhost:5150/admin/dashboard
# http://localhost:5150/admin/products
```

## 📋 Checklist de Verificação

- [ ] `npm install` executado com sucesso
- [ ] `npm run build:css` gerou `assets/static/css/output.css`
- [ ] Tera adicionado ao `Cargo.toml`
- [ ] `tower-http` adicionado ao `Cargo.toml`
- [ ] `AppState` configurado com Tera
- [ ] Controller `admin_views.rs` criado
- [ ] Rotas de views adicionadas
- [ ] Rota de arquivos estáticos configurada
- [ ] Layouts atualizados para usar `output.css`
- [ ] Servidor inicia sem erros
- [ ] Página de login abre corretamente
- [ ] JavaScript (Alpine.js) funciona
- [ ] Estilos (Tailwind) aplicados

## 🔧 Troubleshooting

### CSS não carrega

```bash
# Verificar se o arquivo foi gerado
ls -la assets/static/css/output.css

# Recompilar
npm run build:css

# Verificar rota de arquivos estáticos no servidor
# Deve estar: .nest_service("/static", ServeDir::new("assets/static"))
```

### Templates não renderizam

```bash
# Verificar path do Tera
# Deve ser: "assets/views/**/*.html.tera"

# Verificar se arquivos existem
ls -la assets/views/admin/

# Verificar logs de erro do servidor
# Ele vai mostrar qual template não encontrou
```

### Alpine.js não funciona

- Abrir DevTools (F12)
- Verificar Console por erros JavaScript
- Verificar se CDN do Alpine.js carrega
- Verificar atributos `x-data`, `x-init` nos elementos

### Páginas retornam 404

- Verificar se rotas foram adicionadas corretamente
- Verificar se controller foi importado
- Verificar se servidor foi reiniciado após mudanças

## 🎨 Próximos Passos

1. **Implementar autenticação real**
   - Middleware de proteção de rotas
   - Session/JWT
   - Logout funcional

2. **Criar páginas faltantes**
   - Detalhes de pedido
   - Editar produto (com ID)
   - Analytics
   - Lojas
   - Coleções

3. **Integrar com API backend**
   - Conectar Alpine.js com endpoints reais
   - Validar respostas
   - Tratamento de erros

4. **Melhorias de UX**
   - Loading states
   - Skeleton loaders
   - Confirmações de ação
   - Validação de formulários client-side

5. **Otimizações**
   - Lazy loading de imagens
   - Code splitting
   - Cache de assets
   - Minificação de JS

## 📚 Documentação de Referência

- **TailwindCSS**: https://tailwindcss.com/docs
- **Alpine.js**: https://alpinejs.dev/start-here
- **Tera Templates**: https://keats.github.io/tera/docs/
- **Tower HTTP**: https://docs.rs/tower-http/latest/tower_http/
- **Axum**: https://docs.rs/axum/latest/axum/

---

**Status:** Pronto para produção após autenticação ✅
**Última atualização:** $(date)
