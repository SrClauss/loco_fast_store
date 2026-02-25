# Guia de Implementação do Painel Administrativo

## 📋 Status da Implementação

### ✅ Concluído

1. **Estrutura Base**
   - ✅ Layout principal (`layouts/base.html`)
   - ✅ Layout de autenticação (`layouts/auth.html`)
   - ✅ CSS customizado com TailwindCSS (`assets/static/css/app.css`)
   - ✅ JavaScript base com Alpine.js (`assets/static/js/app.js`)
   - ✅ Configuração TailwindCSS (`tailwind.config.js`)

2. **Páginas de Autenticação**
   - ✅ Login (`admin/login.html`)

3. **Dashboard**
   - ✅ Dashboard principal com métricas (`admin/dashboard.html`)

4. **Gestão de Produtos**
   - ✅ Listagem de produtos (`admin/products/list.html`)
   - ✅ Formulário de produtos (`admin/products/form.html`)

5. **Gestão de Pedidos**
   - ✅ Listagem de pedidos (`admin/orders/list.html`)

## 🚀 Próximos Passos para Completar

### 1. Páginas Restantes (Alta Prioridade)

#### Categorias
- `assets/views/admin/categories/list.html`
- `assets/views/admin/categories/form.html`

#### Coleções
- `assets/views/admin/collections/list.html`
- `assets/views/admin/collections/form.html`

#### Clientes
- `assets/views/admin/customers/list.html`
- `assets/views/admin/customers/detail.html`

#### Lojas
- `assets/views/admin/stores/list.html`
- `assets/views/admin/stores/form.html`

#### Analytics
- `assets/views/admin/analytics/index.html`

#### Detalhes de Pedido
- `assets/views/admin/orders/detail.html`

### 2. Componentes Reutilizáveis

Criar componentes Alpine.js para:
- Modal genérico
- Confirmação de ação
- Upload de imagens
- Seletor de data/hora
- Filtros avançados

### 3. Integração com Backend (CRÍTICO)

**Configurar Tera no Loco:**

```rust
// src/app.rs
use tera::Tera;

pub struct App {
    pub tera: Tera,
}

impl App {
    pub fn new() -> Result<Self> {
        let tera = match Tera::new("assets/views/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        
        Ok(Self { tera })
    }
}
```

**Criar Controller de Views:**

```rust
// src/controllers/admin.rs
use axum::{
    extract::State,
    response::Html,
};
use crate::app::App;

pub async fn dashboard(State(app): State<Arc<App>>) -> Result<Html<String>> {
    let context = tera::Context::new();
    let html = app.tera.render("admin/dashboard.html", &context)?;
    Ok(Html(html))
}

pub async fn products_list(State(app): State<Arc<App>>) -> Result<Html<String>> {
    let context = tera::Context::new();
    let html = app.tera.render("admin/products/list.html", &context)?;
    Ok(Html(html))
}

// ... outras rotas
```

**Adicionar rotas no app.rs:**

```rust
// Rotas admin
.route("/admin/dashboard", get(controllers::admin::dashboard))
.route("/admin/products", get(controllers::admin::products_list))
.route("/admin/products/new", get(controllers::admin::products_new))
.route("/admin/products/:id/edit", get(controllers::admin::products_edit))
.route("/admin/orders", get(controllers::admin::orders_list))
.route("/admin/orders/:id", get(controllers::admin::orders_detail))
// ... demais rotas
```

### 4. Instalação e Build

**Instalar dependências:**

```bash
cd loco_fast_store

# Instalar Node.js dependencies
npm install

# Build do CSS
npm run build:css

# Ou em modo dev (watch)
npm run dev
```

**Adicionar ao Cargo.toml:**

```toml
[dependencies]
tera = "1.19"
```

### 5. Arquivos Estáticos

**Configurar servir arquivos estáticos no Loco:**

```rust
// src/app.rs
use tower_http::services::ServeDir;

// No builder de rotas
.nest_service("/static", ServeDir::new("assets/static"))
```

### 6. Middleware de Autenticação

Criar middleware para proteger rotas admin:

```rust
// src/middleware/auth.rs
use axum::{
    middleware::Next,
    http::Request,
    response::Response,
};

pub async fn require_auth<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    // Verificar token JWT
    // Redirecionar para /admin/login se não autenticado
}
```

## 📦 Estrutura de Arquivos Final

```
loco_fast_store/
├── assets/
│   ├── static/
│   │   ├── css/
│   │   │   ├── app.css (fonte TailwindCSS)
│   │   │   └── output.css (compilado)
│   │   ├── js/
│   │   │   └── app.js
│   │   └── images/
│   │       ├── logo.svg
│   │       └── placeholder.png
│   └── views/
│       └── admin/
│           ├── layouts/
│           │   ├── base.html.tera ✅
│           │   └── auth.html.tera ✅
│           ├── components/
│           │   └── modal.html.tera ✅
│           ├── login.html.tera ✅
│           ├── dashboard.html.tera ✅
│           ├── products/
│           │   ├── list.html.tera ✅
│           │   └── form.html.tera ✅
│           ├── orders/
│           │   ├── list.html.tera ✅
│           │   └── detail.html.tera ⏳
│           ├── categories/
│           │   ├── list.html.tera ✅
│           │   └── form.html.tera ⏳
│           ├── collections/
│           │   ├── list.html.tera ⏳
│           │   └── form.html.tera ⏳
│           ├── customers/
│           │   ├── list.html.tera ✅
│           │   └── detail.html.tera ⏳
│           ├── stores/
│           │   ├── list.html.tera ⏳
│           │   └── form.html.tera ⏳
│           └── analytics/
│               └── index.html.tera ✅
├── src/
│   ├── controllers/
│   │   ├── admin.rs (novo)
│   │   └── ...
│   └── middleware/
│       └── auth.rs (novo)
├── tailwind.config.js ✅
├── package.json ✅
└── README_ADMIN.md ✅
```

## 🎨 Design System

### Cores

- **Primary:** Pink/Rose (#ec4899 - #fb7185)
- **Success:** Green (#22c55e)
- **Warning:** Yellow (#facc15)
- **Error:** Red (#ef4444)
- **Info:** Blue (#3b82f6)

### Tipografia

- **Font Family:** Inter
- **Sizes:** text-sm (14px), text-base (16px), text-lg (18px), text-xl (20px), text-2xl (24px)

### Espaçamento

- **Container:** max-w-7xl mx-auto px-6
- **Card Padding:** px-6 py-4
- **Section Spacing:** space-y-6

### Componentes

Todos os componentes seguem o padrão Material Design com:
- Bordas arredondadas (rounded-lg)
- Sombras suaves (shadow-sm)
- Transições suaves (transition-all duration-200)
- Hover states consistentes

## 🔧 Comandos Úteis

```bash
# Desenvolvimento CSS
npm run dev

# Build de produção
npm run build:css

# Rodar servidor Loco
cargo loco start

# Verificar erros
cargo check
```

## 📝 Checklist de Implementação

### Backend
- [ ] Configurar Tera
- [ ] Criar controller admin.rs
- [ ] Adicionar rotas de views
- [ ] Configurar servir arquivos estáticos
- [ ] Implementar middleware de autenticação
- [ ] Adicionar proteção CSRF

### Frontend
- [ ] Instalar dependências Node.js
- [ ] Compilar TailwindCSS
- [ ] Criar páginas restantes
- [ ] Testar responsividade
- [ ] Implementar dark mode (opcional)

### Testes
- [ ] Testar todas as páginas
- [ ] Validar formulários
- [ ] Testar fluxos de autenticação
- [ ] Testar CRUD completos
- [ ] Verificar performance

## 🚀 Deploy

### Preparação
```bash
# Build CSS
npm run build:css

# Build Rust
cargo build --release
```

### Variáveis de Ambiente
```env
# Já configuradas
ASAAS_API_KEY=...
DATABASE_URL=...
REDIS_URL=...

# Adicionar se necessário
SESSION_SECRET=...
JWT_SECRET=...
```

## 📚 Referências

- [TailwindCSS Documentation](https://tailwindcss.com)
- [Alpine.js Documentation](https://alpinejs.dev)
- [Tera Template Engine](https://tera.netlify.app)
- [Loco Framework](https://loco.rs)
- [Material Design Guidelines](https://m3.material.io)
- [Medusa.js Admin](https://github.com/medusajs/admin) (inspiração visual)

## 💡 Notas Importantes

1. **Alpine.js Stores:** Todos os estados globais (auth, toasts, modal, sidebar) já estão configurados em `app.js`

2. **API Client:** O cliente HTTP já está pronto com autenticação Bearer automática

3. **Formatação:** Funções helper para moeda, data e hora já implementadas

4. **Toasts:** Sistema de notificações toast já funcional

5. **Responsividade:** Todo layout é mobile-first e responsivo

6. **Acessibilidade:** Componentes seguem práticas WCAG básicas

## 🐛 Troubleshooting

### CSS não está carregando
- Verificar se compilou: `npm run build:css`
- Verificar `output.css` foi gerado em `assets/static/css/`
- Verificar se Loco está servindo arquivos estáticos

### Alpine.js não funciona
- Verificar CDN no layout base
- Abrir DevTools e procurar erros JavaScript
- Verificar atributo `x-data` nos componentes

### Templates não renderizam
- Verificar path do Tera: `assets/views/**/*.html`
- Verificar sintaxe Jinja2/Tera
- Verificar logs do servidor Loco

---

**Última atualização:** $(date)
**Versão:** 1.0.0
**Status:** Em Desenvolvimento 🚧
