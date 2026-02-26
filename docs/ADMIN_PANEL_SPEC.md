# PROMPT: Painel Administrativo E-commerce com Tera Templates + Alpine.js

## Contexto do Sistema
Crie um painel administrativo completo para uma plataforma de e-commerce multi-loja usando **Server-Side Rendering (SSR)** com **Tera Templates** e **Alpine.js** para interatividade client-side.

## Stack Tecnológica Obrigatória
- **Backend**: Rust + Loco Framework (já implementado)
- **Templates**: Tera (sintaxe similar a Jinja2/Django)
- **Frontend**: Alpine.js 3.x para reatividade
- **CSS**: TailwindCSS 3.x
- **Ícones**: Heroicons ou Lucide Icons
- **Gráficos**: Chart.js ou ApexCharts (via Alpine.js)

---

## Estrutura de Pastas Esperada

```
assets/
├── static/
│   ├── css/
│   │   └── app.css              # TailwindCSS compilado
│   ├── js/
│   │   ├── alpine.min.js
│   │   └── chart.min.js
│   └── images/
└── views/
    ├── layouts/
    │   ├── base.html            # Layout principal
    │   ├── auth.html            # Layout sem sidebar
    │   └── partials/
    │       ├── header.html
    │       ├── sidebar.html
    │       └── footer.html
    ├── components/
    │   ├── table.html           # Tabela reutilizável
    │   ├── modal.html
    │   ├── form-input.html
    │   ├── pagination.html
    │   └── toast.html
    ├── auth/
    │   ├── login.html
    │   ├── register.html
    │   └── forgot-password.html
    ├── dashboard/
    │   └── index.html           # Dashboard principal
    ├── stores/
    │   ├── list.html
    │   ├── edit.html
    │   └── create.html
    ├── products/
    │   ├── list.html
    │   ├── edit.html
    │   ├── create.html
    │   └── variants.html
    ├── categories/
    │   ├── list.html
    │   └── form.html
    ├── collections/
    │   ├── list.html
    │   └── form.html
    ├── orders/
    │   ├── list.html
    │   └── detail.html
    ├── customers/
    │   ├── list.html
    │   └── detail.html
    └── analytics/
        └── index.html
```

---

## Requisitos Funcionais do Painel

### 1. Autenticação (`/auth`)
- **Login** com email/senha
- **Registro** de nova conta
- **Recuperação de senha**
- **Verificação de email**
- **Magic Link** login
- Salvar JWT em localStorage (Alpine.js persist)

### 2. Dashboard (`/dashboard`)
**Métricas principais:**
- Total de vendas (hoje, semana, mês)
- Número de pedidos (por status)
- Produtos mais vendidos (top 5)
- Carrinhos abandonados (últimas 24h)
- Lead score médio
- Gráfico de vendas (últimos 7 dias)

**Widgets Alpine.js:**
```html
<div x-data="dashboard()" x-init="fetchStats()">
  <div class="grid grid-cols-4 gap-4">
    <div class="stat-card">
      <span x-text="stats.totalSales"></span>
    </div>
  </div>
</div>
```

### 4. Gestão de Produtos (`/products`)
**Rotas da API:**
- `GET /api/v1/products` - Listar produtos
- `GET /api/v1/products/:pid` - Detalhes
- `POST /api/v1/products` - Criar
- `PUT /api/v1/products/:pid` - Atualizar
- `DELETE /api/v1/products/:pid` - Remover
- `POST /api/v1/products/:pid/variants` - Criar variante

**Funcionalidades:**
- Listagem com paginação (cursor-based)
- Filtros: categoria, status, estoque
- Busca por título/SKU
- Upload múltiplo de imagens
- Editor de variantes (tamanho, cor, etc.)
- Gestão de preços e estoque
- SEO: meta title, description

**Alpine.js Store:**
```javascript
Alpine.store('products', {
  items: [],
  pagination: { cursor: null, hasMore: false },
  filters: { category: null, status: 'active' },
  
  async fetchProducts() {
    const res = await fetch(`/api/stores/${storeId}/products?${params}`)
    this.items = await res.json()
  }
})
```

### 5. Gestão de Categorias (`/categories`)
**Rotas da API:**
- `GET /api/v1/categories` - Listar
- `POST /api/v1/categories` - Criar
- `PUT /api/v1/categories/:pid` - Atualizar
- `DELETE /api/v1/categories/:pid` - Remover

**Funcionalidades:**
- Árvore hierárquica (categorias pai/filho)
- Drag & drop para reordenar
- Imagem de capa
- Slug automático

### 6. Gestão de Coleções (`/collections`)
**Rotas da API:**
- `GET /api/v1/collections` - Listar
- `POST /api/v1/collections` - Criar
- `POST /api/v1/collections/:pid/products` - Adicionar produto
- `DELETE /api/v1/collections/:pid/products/:product_pid` - Remover

**Funcionalidades:**
- Adicionar/remover produtos
- Ordenação customizada
- Publicar/despublicar

### 7. Gestão de Pedidos (`/orders`)
**Rotas da API:**
- `GET /api/v1/orders` - Listar
- `GET /api/v1/orders/:pid` - Detalhes
- `PUT /api/v1/orders/:pid/status` - Atualizar status

**Funcionalidades:**
- Listagem com filtros (status, data, cliente)
- Timeline de status do pedido
- Detalhes: itens, endereço, pagamento
- Atualizar status: pending → confirmed → shipped → delivered
- Imprimir nota fiscal
- Rastreamento de envio

**Estados do pedido:**
- pending, confirmed, processing, shipped, delivered, cancelled, refunded

### 8. Gestão de Clientes (`/customers`)
**Rotas da API:**
- `GET /api/v1/customers` - Listar
- `GET /api/v1/customers/:pid` - Detalhes
- `PUT /api/v1/customers/:pid` - Atualizar
- `POST /api/v1/customers/:pid/addresses` - Adicionar endereço

**Funcionalidades:**
- Histórico de compras
- Lead score
- Endereços salvos
- Notas/observações

### 9. Analytics & Relatórios (`/analytics`)
**Funcionalidades:**
- Gráfico de vendas por período
- Produtos mais vendidos
- Taxa de conversão de carrinho
- Carrinhos abandonados
- Funil de vendas
- Lead scoring (cold, cool, warm, hot)

### 10. Carrinho de Compras (Info)
**Rotas da API:**
- `POST /api/v1/carts` - Criar/buscar carrinho
- `GET /api/v1/carts/:pid` - Detalhes
- `POST /api/v1/carts/:pid/items` - Adicionar item
- `PUT /api/v1/carts/:pid/items/:item_id` - Atualizar quantidade
- `DELETE /api/v1/carts/:pid/items/:item_id` - Remover item

*(Esta funcionalidade é principalmente no frontend da loja, não no admin)*

### 11. Pagamentos (Info)
**Rotas da API (Asaas):**
- `POST /api/v1/orders/:order_pid/payments/asaas` - Criar pagamento
- `POST /api/payments/asaas/webhook` - Receber notificação
- `GET /api/payments/asaas/webhooks` - Listar webhooks configurados

**No admin:**
- Visualizar status de pagamento dos pedidos
- Histórico de transações

---

## Componentes Reutilizáveis (Tera + Alpine.js)

### Component: Tabela Genérica
```html
<!-- components/table.html -->
<div x-data="{ selected: [] }">
  <table class="min-w-full divide-y divide-gray-200">
    <thead>
      <tr>
        {% for column in columns %}
        <th>{{ column.label }}</th>
        {% endfor %}
      </tr>
    </thead>
    <tbody>
      {% for row in data %}
      <tr>
        {% for column in columns %}
        <td>{{ row[column.field] }}</td>
        {% endfor %}
      </tr>
      {% endfor %}
    </tbody>
  </table>
  
  {% include "components/pagination.html" %}
</div>
```

### Component: Modal
```html
<!-- components/modal.html -->
<div x-show="open" 
     x-cloak
     @keydown.escape.window="open = false"
     class="fixed inset-0 z-50">
  <div class="modal-backdrop" @click="open = false"></div>
  <div class="modal-content">
    <slot></slot>
  </div>
</div>
```

### Component: Toast/Notificação
```html
<!-- components/toast.html -->
<div x-data="toast()" 
     @toast.window="show($event.detail)">
  <template x-if="visible">
    <div :class="typeClass" x-transition>
      <span x-text="message"></span>
    </div>
  </template>
</div>
```

### Component: Form Input
```html
<!-- components/form-input.html -->
<div class="form-group">
  <label :for="id">{{ label }}</label>
  <input 
    :id="id"
    :type="type"
    x-model="value"
    :class="{'border-red-500': error}"
  />
  <span x-show="error" class="text-red-500 text-sm" x-text="error"></span>
</div>
```

---

## Layout Base (Tera)

```html
<!-- layouts/base.html -->
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% block title %}Admin{% endblock %} | LocoFastStore</title>
  <link rel="stylesheet" href="/static/css/app.css">
  <script defer src="/static/js/alpine.min.js"></script>
</head>
<body x-data="{ sidebarOpen: true }" class="bg-gray-50">
  
  {% include "layouts/partials/sidebar.html" %}
  
  <div class="flex-1 flex flex-col" :class="{ 'ml-64': sidebarOpen }">
    {% include "layouts/partials/header.html" %}
    
    <main class="flex-1 p-6">
      {% include "components/toast.html" %}
      {% block content %}{% endblock %}
    </main>
    
    {% include "layouts/partials/footer.html" %}
  </div>
  
  {% block scripts %}{% endblock %}
</body>
</html>
```

---

## Padrões de Integração com API

### 1. Autenticação Global (Alpine.js)
```javascript
document.addEventListener('alpine:init', () => {
  Alpine.store('auth', {
    token: localStorage.getItem('jwt_token'),
    user: null,
    
    async fetchUser() {
      const res = await fetch('/api/auth/current', {
        headers: { 'Authorization': `Bearer ${this.token}` }
      })
      this.user = await res.json()
    },
    
    logout() {
      localStorage.removeItem('jwt_token')
      window.location = '/auth/login'
    }
  })
})
```

### 2. Fetch Helper
```javascript
Alpine.magic('fetch', () => {
  return async (url, options = {}) => {
    const token = Alpine.store('auth').token
    const res = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
        ...options.headers
      }
    })
    
    if (!res.ok) {
      const error = await res.json()
      throw new Error(error.message || 'Request failed')
    }
    
    return res.json()
  }
})
```

### 3. Formulários com Validação
```javascript
function productForm(initial = {}) {
  return {
    form: {
      title: initial.title || '',
      slug: initial.slug || '',
      description: initial.description || '',
      status: initial.status || 'draft',
      price: initial.price || 0,
      stock: initial.stock || 0
    },
    errors: {},
    loading: false,
    
    async save() {
      this.loading = true
      this.errors = {}
      
      try {
        const storeId = '{{ store.pid }}'
        const method = initial.pid ? 'PUT' : 'POST'
        const url = initial.pid 
          ? `/api/stores/${storeId}/products/${initial.pid}`
          : `/api/stores/${storeId}/products`
        
        await this.$fetch(url, {
          method,
          body: JSON.stringify(this.form)
        })
        
        window.dispatchEvent(new CustomEvent('toast', {
          detail: { message: 'Produto salvo!', type: 'success' }
        }))
        
        window.location = `/stores/${storeId}/products`
      } catch (e) {
        this.errors = e.response?.errors || { _: e.message }
      } finally {
        this.loading = false
      }
    }
  }
}
```

---

## Recursos Visuais Esperados

### Design System
- **Cores primárias**: Indigo/Blue para ações principais
- **Estados**: verde (sucesso), vermelho (erro), amarelo (aviso)
- **Tipografia**: Inter ou System fonts
- **Espaçamento**: baseado em TailwindCSS (4px grid)

### Componentes UI
- Botões com estados (loading, disabled, hover)
- Cards com shadow e border-radius
- Inputs com focus states e validação visual
- Dropdowns/selects estilizados
- Badges de status (active, inactive, pending, etc.)
- Skeleton loaders durante fetch

### Responsividade
- Desktop first (1920px, 1440px, 1024px)
- Tablet (768px): sidebar colapsável
- Mobile (375px): bottom navigation

---

## Requisitos de Performance & UX

1. **SSR**: HTML inicial renderizado no servidor (Tera)
2. **Hydration**: Alpine.js apenas para interatividade
3. **Lazy loading**: imagens e gráficos
4. **Otimistic UI**: atualizar interface antes da resposta da API
5. **Debounce**: em campos de busca (300ms)
6. **Cache**: usar localStorage para dados estáticos (categorias, etc.)
7. **Feedback visual**: loaders, skeleton screens, toast notifications

---

## Endpoints da API (Resumo Completo)

### Autenticação
```
POST   /api/auth/register
POST   /api/auth/login
POST   /api/auth/forgot
POST   /api/auth/reset
GET    /api/auth/current
GET    /api/auth/verify/:token
POST   /api/auth/magic-link
GET    /api/auth/magic-link/:token
POST   /api/auth/resend-verification-mail
```

### Lojas
```
POST   /api/stores
GET    /api/stores
GET    /api/stores/:pid
PUT    /api/stores/:pid
```

### Produtos
```
POST   /api/v1/products
GET    /api/v1/products
GET    /api/v1/products/:pid
PUT    /api/v1/products/:pid
DELETE /api/v1/products/:pid
POST   /api/v1/products/:pid/variants
```

### Categorias
```
POST   /api/v1/categories
GET    /api/v1/categories
GET    /api/v1/categories/:pid
PUT    /api/v1/categories/:pid
DELETE /api/v1/categories/:pid
```

### Coleções
```
POST   /api/v1/collections
GET    /api/v1/collections
GET    /api/v1/collections/:pid
POST   /api/v1/collections/:pid/products
DELETE /api/v1/collections/:pid/products/:product_pid
```

### Carrinhos
```
POST   /api/v1/carts
GET    /api/v1/carts/:pid
POST   /api/v1/carts/:pid/items
PUT    /api/v1/carts/:pid/items/:item_id
DELETE /api/v1/carts/:pid/items/:item_id
```

### Pedidos
```
POST   /api/v1/orders
GET    /api/v1/orders
GET    /api/v1/orders/:pid
PUT    /api/v1/orders/:pid/status
```

### Clientes
```
POST   /api/v1/customers
GET    /api/v1/customers
GET    /api/v1/customers/:pid
PUT    /api/v1/customers/:pid
POST   /api/v1/customers/:pid/addresses
GET    /api/v1/customers/:pid/addresses
```

### Pagamentos
```
POST   /api/v1/orders/:order_pid/payments/asaas
POST   /api/payments/asaas/webhook
GET    /api/payments/asaas/webhooks
```

### Health Check
```
GET    /_health
GET    /_ping
GET    /_readiness
```

---

## Formato de Resposta da API

Todas as respostas seguem o padrão:

**Sucesso:**
```json
{
  "ok": true,
  "data": { ... }
}
```

**Sucesso (paginado):**
```json
{
  "ok": true,
  "data": [...],
  "pagination": {
    "cursor": "next-cursor-value",
    "hasMore": true,
    "count": 20
  }
}
```

**Erro:**
```json
{
  "ok": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Mensagem legível"
  }
}
```

---

## Entregáveis Esperados

1. **Estrutura completa de pastas** conforme especificado
2. **Todos os templates Tera** (.html) para cada funcionalidade
3. **Componentes reutilizáveis** (table, modal, form-input, etc.)
4. **Alpine.js components/stores** para cada módulo
5. **Layout responsivo** com sidebar colapsável
6. **Tema dark mode** (opcional, com Alpine.js persist)
7. **Documentação** de uso dos componentes

---

## Observações Importantes

- **Não usar SPA**: o objetivo é SSR com hydration mínima
- **Evitar bibliotecas pesadas**: usar Alpine.js (15KB) ao invés de React/Vue
- **Acessibilidade**: ARIA labels, navegação por teclado
- **SEO friendly**: meta tags adequadas em cada página
- **Segurança**: CSRF tokens, sanitização de inputs
- **I18n ready**: estrutura preparada para múltiplos idiomas (pt-BR padrão)
