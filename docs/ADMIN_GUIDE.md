# 🎨 Painel Administrativo - Guia Completo

## 📋 Visão Geral

Painel administrativo completo com design Material inspirado no Medusa.js, construído com:
- **Templates:** Tera (Server-Side Rendering)
- **JavaScript:** Alpine.js 3.x (reatividade leve)
- **CSS:** TailwindCSS 3.x
- **Charts:** Chart.js 4.x

## ✅ Páginas Implementadas

### 1. Autenticação
- **Login** (`/admin/login`) - Autenticação com email/senha
- Estados: loading, validação, erros

### 2. Dashboard  
- **Home** (`/admin/dashboard`) - Visão geral com:
  - 4 métricas principais (Receita, Pedidos, Clientes, Produtos)
  - Gráfico de receita temporal
  - Pedidos recentes
  - Produtos mais vendidos
  - Feed de atividades

### 3. Produtos
- **Lista** (`/admin/products`) - Tabela completa com:
  - Filtros (status, categoria)
  - Busca por nome/SKU
  - Paginação
  - Bulk actions
  - Export CSV
  
- **Formulário** (`/admin/products/new`, `/admin/products/:id/edit`) - Com:
  - Informações básicas
  - Gestão de preços e margem
  - Inventário e estoque
  - Upload múltiplo de imagens (drag & drop)
  - Categorias e coleções
  - Tags dinâmicas
  - Status (ativo, rascunho, arquivado)

### 4. Pedidos
- **Lista** (`/admin/orders`) - Com:
  - 4 cards de estatísticas
  - Filtros (status do pedido, status pagamento)
  - Busca
  - Ações rápidas (visualizar, imprimir)
  - Export

### 5. Categorias
- **Gestão** (`/admin/categories`) - Modal-based com:
  - Grid de cards
  - Categorias hierárquicas
  - Auto-geração de slug
  - Contador de produtos

### 6. Clientes
- **Lista** (`/admin/customers`) - Com:
  - Métricas (Total, Novos, Ativos, LTV)
  - Segmentação
  - Informações de compra
  - Ações de contato

### 7. Analytics
- **Dashboard** (`/admin/analytics`) - Com:
  - Seletor de período
  - 4 métricas comparativas
  - Gráficos diversos
  - Top produtos
  - Fontes de tráfego
  - Distribuição geográfica

## 🚀 Como Usar

### 1. Instalar Dependências

```bash
cd loco_fast_store

# Node.js
npm install

# Rust/Loco
cargo build
```

### 2. Compilar CSS

```bash
# Desenvolvimento (watch mode)
npm run dev

# Produção (minified)
npm run build:css
```

### 3. Configurar Backend

Ver arquivo [INSTALACAO_RAPIDA.md](INSTALACAO_RAPIDA.md) para:
- Configuração do Tera
- Criação de controllers
- Adição de rotas
- Servir arquivos estáticos

### 4. Acessar

```bash
# Rodar servidor
cargo loco start

# Acessar no navegador
http://localhost:5150/admin/login
```

## 🎨 Componentes Disponíveis

### Botões
```html
<button class="btn-primary">Primário</button>
<button class="btn-secondary">Secundário</button>
<button class="btn-ghost">Ghost</button>
<button class="btn-sm">Pequeno</button>
```

### Cards
```html
<div class="card">
  <div class="card-header">
    <h3>Título</h3>
  </div>
  <div class="card-body">
    Conteúdo
  </div>
</div>
```

### Formulários
```html
<div>
  <label class="form-label">Label</label>
  <input type="text" class="form-input">
  <p class="form-error">Erro</p>
</div>
```

### Badges
```html
<span class="badge badge-success">Sucesso</span>
<span class="badge badge-warning">Alerta</span>
<span class="badge badge-error">Erro</span>
<span class="badge badge-info">Info</span>
```

### Tabelas
```html
<table class="table">
  <thead>
    <tr>
      <th class="table-header">Coluna</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td class="table-cell">Dado</td>
    </tr>
  </tbody>
</table>
```

## 💡 Alpine.js Helpers

### API Client

```javascript
// GET
const data = await api.get('/products');

// POST
const result = await api.post('/products', { name: 'Product' });

// PUT
await api.put('/products/1', { name: 'Updated' });

// DELETE
await api.delete('/products/1');
```

### Toast Notifications

```javascript
toast.success('Operação realizada!');
toast.error('Erro ao processar');
toast.warning('Atenção!');
toast.info('Informação');
```

### Format Helpers

```javascript
// Moeda
formatCurrency(10000) // R$ 100,00

// Data
formatDate('2024-01-15') // 15 jan 2024

// Data e hora
formatDateTime('2024-01-15T10:30:00') // 15 jan 2024, 10:30
```

### Stores Globais

```javascript
// Auth
$store.auth.login(email, password)
$store.auth.logout()
$store.auth.user

// Toasts
$store.toasts.add({ message, type })
$store.toasts.remove(id)

// Sidebar
$store.sidebar.toggle()
$store.sidebar.open

// Modal
$store.modal.open(title, content)
$store.modal.close()
```

## 📁 Estrutura de Arquivos

```
assets/
├── static/
│   ├── css/
│   │   ├── app.css          # Fonte TailwindCSS
│   │   └── output.css       # Compilado
│   ├── js/
│   │   └── app.js           # Alpine.js stores
│   └── images/
└── views/
    ├── admin/               # Painel administrativo
    │   ├── layouts/
    │   │   ├── base.html.tera        # Layout com sidebar
    │   │   └── auth.html.tera        # Layout autenticação
    │   ├── components/
    │   │   └── modal.html.tera       # Modal reutilizável
    │   ├── login.html.tera
    │   ├── dashboard.html.tera
    │   ├── products/
    │   │   ├── list.html.tera
    │   │   └── form.html.tera
    │   ├── orders/
    │   │   └── list.html.tera
    │   ├── categories/
    │   │   └── list.html.tera
    │   ├── customers/
    │   │   └── list.html.tera
    │   └── analytics/
    │       └── index.html.tera
    └── store/               # Templates da loja (futuro)
```

## 🎯 Features Implementadas

- ✅ Autenticação JWT
- ✅ SSR com Tera templates
- ✅ Reatividade Alpine.js
- ✅ Design responsivo
- ✅ Loading states
- ✅ Skeleton loaders
- ✅ Toast notifications
- ✅ Modal system
- ✅ Form validation
- ✅ Drag & drop upload
- ✅ Paginação
- ✅ Filtros dinâmicos
- ✅ Busca com debounce
- ✅ Bulk actions
- ✅ Export de dados
- ✅ Charts interativos
- ✅ Empty states

## 🔧 Customização

### Cores

Editar `tailwind.config.js`:

```javascript
theme: {
  extend: {
    colors: {
      primary: {
        500: '#ec4899', // Sua cor
      },
    },
  },
}
```

### Componentes CSS

Editar `assets/static/css/app.css`:

```css
@layer components {
  .btn-custom {
    @apply px-4 py-2 rounded-lg bg-purple-500 text-white;
  }
}
```

### Alpine.js Functions

Editar `assets/static/js/app.js`:

```javascript
Alpine.store('myStore', {
  // seu estado
});
```

## 📊 Integrações com Backend

### Exemplo de Controller

```rust
// src/controllers/admin_views.rs
pub async fn products_list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert("current_page", "products");
    
    let html = state.tera.render("admin/products/list.html.tera", &context)?;
    Html(html)
}
```

### Adicionar Rotas

```rust
// Router
.route("/admin/products", get(admin_views::products_list))
.route("/admin/orders", get(admin_views::orders_list))
// ...
```

## 🐛 Troubleshooting

### CSS não aplica
1. Verificar se `output.css` foi gerado
2. Verificar path no layout: `/static/css/output.css`
3. Recompilar: `npm run build:css`

### Alpine.js não funciona
1. Abrir DevTools → Console
2. Verificar erros JavaScript
3. Verificar CDN carregou
4. Verificar `x-data` nos elementos

### Templates não renderizam
1. Verificar path do Tera: `assets/views/**/*.html.tera`
2. Verificar sintaxe Jinja2/Tera
3. Ver logs do servidor

## 📚 Documentação Adicional

- [README_ADMIN.md](README_ADMIN.md) - Documentação completa
- [INSTALACAO_RAPIDA.md](INSTALACAO_RAPIDA.md) - Setup backend
- [RESUMO_IMPLEMENTACAO.md](RESUMO_IMPLEMENTACAO.md) - O que foi feito

## 💬 Suporte

Para dúvidas e issues:
1. Verificar documentação
2. Abrir issue no GitHub
3. Consultar logs do servidor

---

**Status:** ✅ Pronto para integração
**Versão:** 1.0.0
**Última atualização:** 2024
