# Loco Fast Store — SDK Alpine.js: Contrato de Implementação de Frontend

> **Uso interno.** Este documento descreve o esquema completo de consumo da API da loja usando Alpine.js e o arquivo `store-sdk.js`. Ao adaptar uma nova loja basta seguir este contrato: importar o Alpine.js, configurar o `STORE_PID` e usar os componentes prontos.

---

## Sumário

1. [Instalação e configuração](#1-instalação-e-configuração)
2. [Convenções da API](#2-convenções-da-api)
3. [Stores globais Alpine](#3-stores-globais-alpine)
4. [Componentes prontos (x-data)](#4-componentes-prontos-x-data)
5. [SDK JavaScript direto (StoreSDK)](#5-sdk-javascript-direto-storesdk)
6. [Referência completa de endpoints](#6-referência-completa-de-endpoints)
7. [Exemplos de páginas Tera completas](#7-exemplos-de-páginas-tera-completas)
8. [Exemplo com framework SPA (Vite + Vanilla)](#8-exemplo-com-framework-spa-vite--vanilla)

---

## 1. Instalação e configuração

### 1.1 Dependências

Apenas **dois arquivos** precisam ser importados no HTML da loja:

```html
<!-- Alpine.js (obrigatório) -->
<script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>

<!-- Configuração da loja (antes do SDK) -->
<script>
  window.STORE_PID = 'uuid-da-sua-loja'; // ← único ajuste necessário
</script>

<!-- SDK da loja -->
<script src="/static/js/store-sdk.js"></script>
```

> **Tera:** use `{{ store.pid }}` para injetar o UUID diretamente:
> ```html
> <script>window.STORE_PID = '{{ store.pid }}';</script>
> ```

### 1.2 Toast de notificações (obrigatório no layout)

Cole este bloco uma única vez no layout base (normalmente em `base.html`):

```html
<div x-data class="fixed bottom-4 right-4 z-50 space-y-2 pointer-events-none">
  <template x-for="t in $store.toasts.items" :key="t.id">
    <div
      x-show="true"
      x-transition
      class="pointer-events-auto bg-white border shadow-lg rounded-lg px-4 py-3 flex items-center gap-3 max-w-sm"
      :class="{
        'border-green-400': t.type === 'success',
        'border-red-400':   t.type === 'error',
        'border-yellow-400':t.type === 'warning',
        'border-blue-400':  t.type === 'info'
      }"
    >
      <span x-text="t.message" class="text-sm text-gray-800 flex-1"></span>
      <button @click="$store.toasts.dismiss(t.id)" class="text-gray-400 hover:text-gray-600">✕</button>
    </div>
  </template>
</div>
```

---

## 2. Convenções da API

| Item | Detalhe |
|------|---------|
| **Base URL** | `/api/stores/{STORE_PID}/...` |
| **Envelope de resposta** | `{ ok: bool, data: T, meta?: { cursor, has_more, count } }` |
| **Valores monetários** | Centavos inteiros. Ex.: `1990` = R$ 19,90. Use `StoreSDK.formatMoney(1990)` |
| **IDs públicos** | UUID no campo `pid`. Nunca use o campo `id` (interno) |
| **Autenticação admin** | Header `Authorization: Bearer <jwt>` |
| **Autenticação customer** | Header `X-Customer-Token: <token>` (gerenciado automaticamente pelo SDK) |
| **Paginação** | Cursor-based. Parâmetros: `cursor`, `limit`. Resposta: `meta.cursor`, `meta.has_more` |
| **Soft delete** | Registros excluídos têm `deleted_at` preenchido e não aparecem nas listagens |

### Formato de erro

```json
{
  "ok": false,
  "error": {
    "code": "ENTITY_NOT_FOUND",
    "message": "Produto não encontrado",
    "details": null
  }
}
```

---

## 3. Stores globais Alpine

Os stores são registrados automaticamente quando o SDK é carregado. Acesse-os em qualquer elemento com `$store`.

---

### `$store.toasts` — Notificações

| Método | Descrição |
|--------|-----------|
| `$store.toasts.success(msg)` | Toast verde |
| `$store.toasts.error(msg)` | Toast vermelho |
| `$store.toasts.warning(msg)` | Toast amarelo |
| `$store.toasts.info(msg)` | Toast azul |
| `$store.toasts.dismiss(id)` | Remove toast manualmente |

```html
<button @click="$store.toasts.success('Salvo!')">Testar</button>
```

---

### `$store.customer` — Cliente logado

| Propriedade / Método | Tipo | Descrição |
|----------------------|------|-----------|
| `isLoggedIn` | `boolean` | `true` se há token de sessão |
| `data` | `object \| null` | Dados do cliente (`pid`, `email`, `first_name`, …) |
| `loading` | `boolean` | Requisição em andamento |
| `fetch()` | `async` | Carrega dados do backend (`GET /auth/me`) |
| `login(email, password)` | `async` | Autentica e preenche `data` |
| `register(params)` | `async` | Cadastra novo cliente |
| `logout()` | `async` | Encerra sessão e limpa `data` |

```html
<!-- Mostra nome do cliente ou link de login -->
<div x-data>
  <span x-show="$store.customer.isLoggedIn" x-text="'Olá, ' + $store.customer.data?.first_name"></span>
  <a x-show="!$store.customer.isLoggedIn" href="/login">Entrar</a>
  <button x-show="$store.customer.isLoggedIn" @click="$store.customer.logout()">Sair</button>
</div>
```

---

### `$store.cart` — Carrinho ativo

| Propriedade / Método | Tipo | Descrição |
|----------------------|------|-----------|
| `data` | `object \| null` | Carrinho completo com `items[]`, `total`, `subtotal`, … |
| `itemCount` | `number` (computed) | Soma das quantidades de todos os itens |
| `isOpen` | `boolean` | Estado da gaveta do carrinho |
| `loading` | `boolean` | Operação em andamento |
| `init()` | `async` | Recupera carrinho salvo ou cria um novo |
| `addItem(variantId, qty)` | `async` | Adiciona item e abre a gaveta |
| `updateItem(itemId, qty)` | `async` | Atualiza quantidade (0 remove) |
| `removeItem(itemId)` | `async` | Remove item |
| `open()` / `close()` | sync | Abre/fecha a gaveta |

```html
<!-- Botão flutuante com contador -->
<button @click="$store.cart.open()" class="fixed bottom-6 right-6 bg-black text-white rounded-full px-4 py-3">
  🛒 <span x-text="$store.cart.itemCount"></span>
</button>
```

---

## 4. Componentes prontos (x-data)

Todos os componentes são **factories** — funções que retornam objetos Alpine.js, usados diretamente no atributo `x-data`.

---

### `ProductList(defaults?)` — Lista de produtos

**Parâmetros defaults:**

| Campo | Tipo | Padrão | Descrição |
|-------|------|--------|-----------|
| `status` | `string` | `'active'` | Filtra por status |
| `category_id` | `number` | `''` | Filtra por categoria |
| `featured` | `boolean` | `''` | Apenas destaques |
| `q` | `string` | `''` | Busca por texto |
| `limit` | `number` | `20` | Itens por página |

**Propriedades:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `products` | `array` | Produtos carregados |
| `loading` | `boolean` | Carregamento em andamento |
| `hasMore` | `boolean` | Há mais páginas |
| `filters` | `object` | Filtros ativos |

**Métodos:**

| Nome | Descrição |
|------|-----------|
| `fetch(reset?)` | Recarrega lista |
| `loadMore()` | Carrega próxima página (infinito scroll) |
| `applyFilter(key, value)` | Altera filtro e recarrega |
| `search(q)` | Busca por texto e recarrega |

**Exemplo completo:**

```html
<div x-data="ProductList({ status: 'active', limit: 12 })" x-init="init()">

  <!-- Filtros -->
  <div class="flex gap-3 mb-6">
    <input
      x-model="filters.q"
      @input.debounce.400ms="search($event.target.value)"
      placeholder="Buscar produtos..."
      class="border rounded px-3 py-2 w-64"
    >
    <select @change="applyFilter('status', $event.target.value)" class="border rounded px-3 py-2">
      <option value="active">Ativos</option>
      <option value="draft">Rascunho</option>
    </select>
  </div>

  <!-- Grid de produtos -->
  <div class="grid grid-cols-3 gap-6">
    <!-- Loading skeleton -->
    <template x-if="loading && products.length === 0">
      <template x-for="i in [1,2,3,4,5,6]" :key="i">
        <div class="animate-pulse bg-gray-100 rounded-lg h-64"></div>
      </template>
    </template>

    <!-- Produtos -->
    <template x-for="p in products" :key="p.pid">
      <a :href="'/produto/' + p.slug" class="border rounded-lg overflow-hidden hover:shadow-lg transition">
        <img :src="p.image_url ?? '/static/images/placeholder.png'" :alt="p.title" class="w-full h-48 object-cover">
        <div class="p-4">
          <h3 x-text="p.title" class="font-semibold text-gray-900"></h3>
          <p class="text-sm text-gray-500 mt-1" x-text="p.description?.slice(0, 80) + '...'"></p>
        </div>
      </a>
    </template>
  </div>

  <!-- Carregar mais -->
  <div class="text-center mt-8" x-show="hasMore">
    <button
      @click="loadMore()"
      :disabled="loading"
      class="px-6 py-2 border rounded-lg hover:bg-gray-50 disabled:opacity-50"
    >
      <span x-show="!loading">Carregar mais</span>
      <span x-show="loading">Carregando…</span>
    </button>
  </div>

  <!-- Estado vazio -->
  <div x-show="!loading && products.length === 0" class="text-center py-16 text-gray-400">
    Nenhum produto encontrado.
  </div>
</div>
```

---

### `ProductDetail(pid)` — Detalhe do produto

**Propriedades:**

| Nome | Tipo | Descrição |
|------|------|-----------|
| `product` | `object \| null` | Dados do produto |
| `variants` | `array` | Variantes disponíveis |
| `selectedVariant` | `object \| null` | Variante selecionada |
| `quantity` | `number` | Quantidade a adicionar |
| `selectedPrice` | `string` (computed) | Preço formatado da variante selecionada |

**Métodos:**

| Nome | Descrição |
|------|-----------|
| `selectVariant(v)` | Seleciona variante |
| `addToCart()` | Adiciona ao `$store.cart` |

**Exemplo:**

```html
<!-- O pid vem do contexto Tera: {{ product.pid }} -->
<div x-data="ProductDetail('{{ product.pid }}')" x-init="init()">

  <div x-show="loading" class="animate-pulse h-96 bg-gray-100 rounded"></div>

  <div x-show="!loading && product" class="grid grid-cols-2 gap-12">
    <!-- Imagem principal -->
    <img :src="product?.image_url ?? '/static/images/placeholder.png'" :alt="product?.title" class="rounded-xl">

    <!-- Informações -->
    <div>
      <h1 x-text="product?.title" class="text-3xl font-bold text-gray-900"></h1>
      <p x-text="product?.description" class="text-gray-600 mt-4"></p>

      <!-- Variantes -->
      <div x-show="variants.length > 1" class="mt-6">
        <p class="text-sm font-medium text-gray-700 mb-2">Variante:</p>
        <div class="flex gap-2 flex-wrap">
          <template x-for="v in variants" :key="v.pid">
            <button
              @click="selectVariant(v)"
              :class="selectedVariant?.pid === v.pid
                ? 'bg-black text-white border-black'
                : 'bg-white text-gray-800 border-gray-300 hover:border-black'"
              class="px-4 py-2 border rounded-lg text-sm font-medium transition"
              x-text="v.title"
            ></button>
          </template>
        </div>
      </div>

      <!-- Preço -->
      <p class="text-2xl font-bold text-gray-900 mt-6" x-text="selectedPrice"></p>

      <!-- Quantidade -->
      <div class="flex items-center gap-3 mt-4">
        <button @click="quantity = Math.max(1, quantity - 1)" class="w-8 h-8 border rounded flex items-center justify-center">-</button>
        <span x-text="quantity" class="w-8 text-center"></span>
        <button @click="quantity++" class="w-8 h-8 border rounded flex items-center justify-center">+</button>
      </div>

      <!-- Botão -->
      <button
        @click="addToCart()"
        :disabled="$store.cart.loading || !selectedVariant"
        class="mt-6 w-full bg-black text-white py-3 rounded-lg font-medium hover:bg-gray-800 disabled:opacity-50 transition"
      >
        <span x-show="!$store.cart.loading">Adicionar ao carrinho</span>
        <span x-show="$store.cart.loading">Adicionando…</span>
      </button>
    </div>
  </div>
</div>
```

---

### `CartDrawer()` — Gaveta lateral do carrinho

**Exemplo completo com overlay:**

```html
<div x-data="CartDrawer()" x-init="init()">

  <!-- Overlay escuro -->
  <div
    x-show="$store.cart.isOpen"
    x-transition:enter="transition ease-out duration-300"
    x-transition:enter-start="opacity-0"
    x-transition:enter-end="opacity-100"
    x-transition:leave="transition ease-in duration-200"
    x-transition:leave-end="opacity-0"
    @click="$store.cart.close()"
    class="fixed inset-0 bg-black/50 z-40"
  ></div>

  <!-- Gaveta -->
  <div
    x-show="$store.cart.isOpen"
    x-transition:enter="transition ease-out duration-300"
    x-transition:enter-start="translate-x-full"
    x-transition:enter-end="translate-x-0"
    x-transition:leave="transition ease-in duration-200"
    x-transition:leave-end="translate-x-full"
    class="fixed top-0 right-0 h-full w-96 bg-white shadow-xl z-50 flex flex-col"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b">
      <h2 class="text-lg font-semibold">Carrinho (<span x-text="$store.cart.itemCount"></span>)</h2>
      <button @click="$store.cart.close()" class="text-gray-400 hover:text-gray-600">✕</button>
    </div>

    <!-- Itens -->
    <div class="flex-1 overflow-y-auto px-6 py-4 space-y-4">
      <template x-if="$store.cart.data?.items?.length === 0">
        <p class="text-center text-gray-400 py-12">Seu carrinho está vazio.</p>
      </template>
      <template x-for="item in ($store.cart.data?.items ?? [])" :key="item.pid">
        <div class="flex items-center gap-4">
          <div class="flex-1">
            <p class="font-medium text-gray-900" x-text="item.title ?? 'Produto'"></p>
            <p class="text-sm text-gray-500" x-text="'Qtd: ' + item.quantity"></p>
          </div>
          <div class="text-right">
            <p class="font-semibold" x-text="formatMoney(item.total)"></p>
            <button
              @click="$store.cart.removeItem(item.id)"
              class="text-xs text-red-500 hover:text-red-700 mt-1"
            >Remover</button>
          </div>
        </div>
      </template>
    </div>

    <!-- Footer com total -->
    <div class="border-t px-6 py-4 space-y-4">
      <div class="flex justify-between font-bold text-lg">
        <span>Total</span>
        <span x-text="formatMoney($store.cart.data?.total ?? 0)"></span>
      </div>
      <a
        href="/checkout"
        class="block w-full text-center bg-black text-white py-3 rounded-lg font-medium hover:bg-gray-800 transition"
      >
        Finalizar compra
      </a>
    </div>
  </div>
</div>
```

---

### `CustomerAuth(opts?)` — Login e cadastro

**Opções:**

| Opção | Tipo | Descrição |
|-------|------|-----------|
| `redirectOnLogin` | `string` | URL para redirecionar após login bem-sucedido |

**Exemplo:**

```html
<div x-data="CustomerAuth({ redirectOnLogin: '/minha-conta' })">

  <!-- Login -->
  <form x-show="mode === 'login'" @submit.prevent="login()" class="space-y-4">
    <h2 class="text-2xl font-bold">Entrar</h2>
    <input x-model="form.email" type="email" placeholder="E-mail" class="w-full border rounded px-3 py-2" required>
    <input x-model="form.password" type="password" placeholder="Senha" class="w-full border rounded px-3 py-2" required>
    <button type="submit" :disabled="loading" class="w-full bg-black text-white py-3 rounded-lg font-medium disabled:opacity-50">
      <span x-show="!loading">Entrar</span>
      <span x-show="loading">Entrando…</span>
    </button>
    <p class="text-center text-sm">
      Não tem conta?
      <a @click.prevent="mode = 'register'" href="#" class="text-black font-medium underline">Criar conta</a>
    </p>
  </form>

  <!-- Cadastro -->
  <form x-show="mode === 'register'" @submit.prevent="register()" class="space-y-4">
    <h2 class="text-2xl font-bold">Criar conta</h2>
    <div class="grid grid-cols-2 gap-3">
      <input x-model="form.first_name" placeholder="Nome" class="border rounded px-3 py-2" required>
      <input x-model="form.last_name" placeholder="Sobrenome" class="border rounded px-3 py-2" required>
    </div>
    <input x-model="form.email" type="email" placeholder="E-mail" class="w-full border rounded px-3 py-2" required>
    <input x-model="form.password" type="password" placeholder="Senha (mín. 8 caracteres)" class="w-full border rounded px-3 py-2" required>
    <label class="flex items-center gap-2 text-sm">
      <input type="checkbox" x-model="form.marketing_consent">
      Aceito receber ofertas e novidades
    </label>
    <button type="submit" :disabled="loading" class="w-full bg-black text-white py-3 rounded-lg font-medium disabled:opacity-50">
      <span x-show="!loading">Criar conta</span>
      <span x-show="loading">Criando…</span>
    </button>
    <p class="text-center text-sm">
      Já tem conta?
      <a @click.prevent="mode = 'login'" href="#" class="text-black font-medium underline">Entrar</a>
    </p>
  </form>
</div>
```

---

### `CheckoutForm()` — Formulário de checkout

**Etapas:**

| Step | Tela |
|------|------|
| 1 | Identificação (email, nome) |
| 2 | Endereço de entrega |
| 3 | Pagamento |
| 4 | Confirmação (pedido criado) |

**Exemplo simplificado:**

```html
<div x-data="CheckoutForm()" x-init="init()" class="max-w-2xl mx-auto">

  <!-- Indicador de etapas -->
  <div class="flex gap-2 mb-8">
    <template x-for="s in [1,2,3]" :key="s">
      <div class="flex-1 h-2 rounded-full"
        :class="s <= step ? 'bg-black' : 'bg-gray-200'">
      </div>
    </template>
  </div>

  <!-- Etapa 1: Identificação -->
  <div x-show="step === 1" class="space-y-4">
    <h2 class="text-xl font-bold">Seus dados</h2>
    <input x-model="form.email" type="email" placeholder="E-mail" class="w-full border rounded px-3 py-2">
    <div class="grid grid-cols-2 gap-3">
      <input x-model="form.first_name" placeholder="Nome" class="border rounded px-3 py-2">
      <input x-model="form.last_name" placeholder="Sobrenome" class="border rounded px-3 py-2">
    </div>
    <button @click="nextStep()" class="w-full bg-black text-white py-3 rounded-lg">Continuar</button>
  </div>

  <!-- Etapa 2: Endereço -->
  <div x-show="step === 2" class="space-y-4">
    <h2 class="text-xl font-bold">Endereço de entrega</h2>
    <input x-model="form.address.postal_code" @blur="fetchAddress()" placeholder="CEP (somente números)" class="w-full border rounded px-3 py-2">
    <input x-model="form.address.address_line_1" placeholder="Rua e número" class="w-full border rounded px-3 py-2">
    <input x-model="form.address.address_line_2" placeholder="Complemento" class="w-full border rounded px-3 py-2">
    <div class="grid grid-cols-2 gap-3">
      <input x-model="form.address.city" placeholder="Cidade" class="border rounded px-3 py-2">
      <input x-model="form.address.state" placeholder="Estado" class="border rounded px-3 py-2" maxlength="2">
    </div>
    <div class="flex gap-3">
      <button @click="prevStep()" class="flex-1 border py-3 rounded-lg">Voltar</button>
      <button @click="nextStep()" class="flex-1 bg-black text-white py-3 rounded-lg">Continuar</button>
    </div>
  </div>

  <!-- Etapa 3: Pagamento -->
  <div x-show="step === 3" class="space-y-4">
    <h2 class="text-xl font-bold">Pagamento</h2>
    <div class="space-y-2">
      <label class="flex items-center gap-3 p-4 border rounded-lg cursor-pointer" :class="form.payment_method === 'pix' ? 'border-black' : ''">
        <input type="radio" x-model="form.payment_method" value="pix">
        <span>PIX (5% de desconto)</span>
      </label>
      <label class="flex items-center gap-3 p-4 border rounded-lg cursor-pointer" :class="form.payment_method === 'credit_card' ? 'border-black' : ''">
        <input type="radio" x-model="form.payment_method" value="credit_card">
        <span>Cartão de crédito (até 12x)</span>
      </label>
    </div>

    <!-- Resumo do pedido -->
    <div class="bg-gray-50 rounded-lg p-4 space-y-2">
      <div class="flex justify-between text-sm">
        <span>Subtotal</span>
        <span x-text="formatMoney($store.cart.data?.subtotal ?? 0)"></span>
      </div>
      <div class="flex justify-between font-bold">
        <span>Total</span>
        <span x-text="formatMoney($store.cart.data?.total ?? 0)"></span>
      </div>
    </div>

    <div class="flex gap-3">
      <button @click="prevStep()" class="flex-1 border py-3 rounded-lg">Voltar</button>
      <button
        @click="submit()"
        :disabled="loading"
        class="flex-1 bg-black text-white py-3 rounded-lg disabled:opacity-50"
      >
        <span x-show="!loading">Finalizar pedido</span>
        <span x-show="loading">Processando…</span>
      </button>
    </div>
  </div>

  <!-- Etapa 4: Confirmação -->
  <div x-show="step === 4" class="text-center py-12 space-y-4">
    <div class="w-16 h-16 bg-green-100 text-green-600 rounded-full flex items-center justify-center mx-auto text-3xl">✓</div>
    <h2 class="text-2xl font-bold">Pedido confirmado!</h2>
    <p class="text-gray-600">Pedido <strong x-text="order?.order_number"></strong> realizado com sucesso.</p>
    <a href="/" class="inline-block mt-4 bg-black text-white px-8 py-3 rounded-lg">Continuar comprando</a>
  </div>
</div>
```

---

### `CustomerAccount()` — Minha conta

```html
<div x-data="CustomerAccount()" x-init="init()">
  <div x-show="loading" class="animate-pulse h-32 bg-gray-100 rounded"></div>

  <div x-show="!loading && profile" class="space-y-8">
    <!-- Perfil -->
    <section>
      <h2 class="text-xl font-bold mb-4">Meus dados</h2>
      <p><strong>Nome:</strong> <span x-text="profile?.first_name + ' ' + profile?.last_name"></span></p>
      <p><strong>E-mail:</strong> <span x-text="profile?.email"></span></p>
      <p><strong>Telefone:</strong> <span x-text="profile?.phone ?? '—'"></span></p>
    </section>

    <!-- Endereços -->
    <section>
      <h2 class="text-xl font-bold mb-4">Meus endereços</h2>
      <template x-if="addresses.length === 0">
        <p class="text-gray-500">Nenhum endereço cadastrado.</p>
      </template>
      <template x-for="a in addresses" :key="a.pid">
        <div class="border rounded-lg p-4">
          <p x-text="a.first_name + ' ' + a.last_name"></p>
          <p x-text="a.address_line_1 + (a.address_line_2 ? ', ' + a.address_line_2 : '')"></p>
          <p x-text="a.city + '/' + a.state + ' — CEP ' + a.postal_code"></p>
        </div>
      </template>
    </section>

    <!-- Sair -->
    <button @click="logout()" class="text-red-600 hover:text-red-700 text-sm font-medium">Sair da conta</button>
  </div>
</div>
```

---

### `SearchBar()` — Busca em tempo real

```html
<div x-data="SearchBar()" @keydown.escape.window="close()" class="relative">
  <div class="relative">
    <input
      x-ref="input"
      x-model="query"
      @input.debounce.300ms="search()"
      @focus="open = results.length > 0"
      placeholder="Buscar produtos…"
      class="w-full border rounded-lg px-4 py-2 pr-10"
    >
    <span x-show="loading" class="absolute right-3 top-2.5 text-gray-400 animate-spin">⟳</span>
  </div>

  <!-- Dropdown de resultados -->
  <div
    x-show="open && results.length > 0"
    @click.outside="close()"
    class="absolute top-full left-0 right-0 mt-1 bg-white border rounded-lg shadow-xl z-50 overflow-hidden"
  >
    <template x-for="p in results" :key="p.pid">
      <a
        :href="'/produto/' + p.slug"
        class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50 transition"
        @click="close()"
      >
        <img :src="p.image_url ?? '/static/images/placeholder.png'" :alt="p.title" class="w-10 h-10 object-cover rounded">
        <div>
          <p class="text-sm font-medium text-gray-900" x-text="p.title"></p>
          <p class="text-xs text-gray-500" x-text="p.description?.slice(0, 50)"></p>
        </div>
      </a>
    </template>
  </div>
</div>
```

---

## 5. SDK JavaScript direto (StoreSDK)

`window.StoreSDK` expõe todos os métodos da API sem Alpine. Útil para scripts customizados.

```js
// Listar produtos em destaque
const { data } = await StoreSDK.products.list({ featured: true, limit: 4 });

// Adicionar ao carrinho programaticamente
await StoreSDK.cart.addItem('uuid-do-carrinho', { variant_id: 42, quantity: 2 });

// Formatar valor
StoreSDK.formatMoney(1990);  // "R$ 19,90"
StoreSDK.formatDate('2026-01-25T10:00:00Z');  // "25 de jan. de 2026"
StoreSDK.slugify('Meu Produto Incrível');  // "meu-produto-incrivel"
```

### Referência dos módulos

| Módulo | Método | Endpoint | Auth |
|--------|--------|----------|------|
| `StoreSDK.auth` | `register(params)` | `POST /auth/register` | — |
| | `login(params)` | `POST /auth/login` | — |
| | `logout()` | `POST /auth/logout` | Token |
| | `me()` | `GET /auth/me` | Token |
| | `isLoggedIn()` | — (localStorage) | — |
| `StoreSDK.products` | `list(params?)` | `GET /products` | — |
| | `get(pid)` | `GET /products/{pid}` | — |
| | `exportCsv()` | `GET /products/export/csv` | JWT |
| | `downloadTemplate()` | `GET /products/import/template` | — |
| `StoreSDK.categories` | `list(params?)` | `GET /categories` | — |
| | `get(pid)` | `GET /categories/{pid}` | — |
| `StoreSDK.collections` | `list()` | `GET /collections` | — |
| | `get(pid)` | `GET /collections/{pid}` | — |
| `StoreSDK.cart` | `getOrCreate()` | `POST /carts` | — |
| | `get(cartPid)` | `GET /carts/{pid}` | — |
| | `addItem(cartPid, params)` | `POST /carts/{pid}/items` | — |
| | `updateItem(cartPid, itemId, qty)` | `PUT /carts/{pid}/items/{id}` | — |
| | `removeItem(cartPid, itemId)` | `DELETE /carts/{pid}/items/{id}` | — |
| `StoreSDK.orders` | `create(params)` | `POST /orders` | Token |
| | `get(pid)` | `GET /orders/{pid}` | — |
| `StoreSDK.customer` | `get(pid)` | `GET /customers/{pid}` | Token |
| | `update(pid, params)` | `PUT /customers/{pid}` | Token |
| | `addresses(pid)` | `GET /customers/{pid}/addresses` | Token |
| | `addAddress(pid, params)` | `POST /customers/{pid}/addresses` | Token |

---

## 6. Referência completa de endpoints

### Autenticação de cliente

```
POST /api/stores/{store_pid}/auth/register
Body: {
  "email": "string",
  "password": "string",
  "first_name": "string",
  "last_name": "string",
  "phone": "string?",
  "marketing_consent": "boolean?"
}
Response: { "token": "string", "customer": { pid, email, first_name, ... } }

POST /api/stores/{store_pid}/auth/login
Body: { "email": "string", "password": "string" }
Response: { "token": "string", "customer": { pid, email, first_name, ... } }

POST /api/stores/{store_pid}/auth/logout
Header: X-Customer-Token: <token>
Response: { "ok": true }

GET /api/stores/{store_pid}/auth/me
Header: X-Customer-Token: <token>
Response: { pid, email, first_name, last_name, phone, has_account, created_at }
```

### Produtos

```
GET  /api/stores/{store_pid}/products
     ?status=active&category_id=1&featured=true&q=texto&limit=20&cursor=xxx
Response: { ok, data: Product[], meta: { cursor, has_more, count } }

GET  /api/stores/{store_pid}/products/{pid}
Response: { ok, data: Product & { variants: Variant[] } }

GET  /api/stores/{store_pid}/products/export/csv
Response: text/csv (download)

GET  /api/stores/{store_pid}/products/import/template
Response: text/csv (download do template)
```

### Categorias

```
GET  /api/stores/{store_pid}/categories
     ?parent_id=1
Response: { ok, data: Category[] }

GET  /api/stores/{store_pid}/categories/{pid}
Response: { ok, data: Category }
```

### Coleções

```
GET  /api/stores/{store_pid}/collections
Response: { ok, data: Collection[] }

GET  /api/stores/{store_pid}/collections/{pid}
Response: { ok, data: Collection & { products: Product[] } }
```

### Carrinho

```
POST /api/stores/{store_pid}/carts?session_id=xxx
Response: { ok, data: Cart & { items: CartItem[] } }

GET  /api/stores/{store_pid}/carts/{pid}
Response: { ok, data: Cart & { items: CartItem[] } }

POST /api/stores/{store_pid}/carts/{pid}/items
Body: { "variant_id": 1, "quantity": 2 }
Response: { ok, data: Cart & { items: CartItem[] } }

PUT  /api/stores/{store_pid}/carts/{pid}/items/{item_id}
Body: { "quantity": 3 }    (quantity: 0 remove o item)
Response: { ok, data: Cart & { items: CartItem[] } }

DELETE /api/stores/{store_pid}/carts/{pid}/items/{item_id}
Response: { ok, data: Cart & { items: CartItem[] } }
```

### Pedidos

```
POST /api/stores/{store_pid}/orders
Header: X-Customer-Token: <token>
Body: {
  "customer_id": 1,
  "shipping_address_id": 1,
  "billing_address_id": 1,
  "payment_method": "pix",
  "notes": "string?"
}
Response: { ok, data: Order & { items: OrderItem[] } }

GET  /api/stores/{store_pid}/orders/{pid}
Response: { ok, data: Order & { items: OrderItem[] } }
```

### Clientes

```
GET  /api/stores/{store_pid}/customers/{pid}
Header: X-Customer-Token: <token>
Response: { ok, data: Customer }

PUT  /api/stores/{store_pid}/customers/{pid}
Header: X-Customer-Token: <token>
Body: { first_name?, last_name?, phone?, marketing_consent? }
Response: { ok, data: Customer }

GET  /api/stores/{store_pid}/customers/{pid}/addresses
Header: X-Customer-Token: <token>
Response: { ok, data: Address[] }

POST /api/stores/{store_pid}/customers/{pid}/addresses
Header: X-Customer-Token: <token>
Body: {
  "first_name": "string",
  "last_name": "string",
  "address_line_1": "string",
  "city": "string",
  "state": "string",
  "postal_code": "string",
  "country": "BR",
  "is_default_shipping": true,
  "is_default_billing": false
}
Response: { ok, data: Address }
```

### Schemas de resposta

```ts
// Produto
type Product = {
  pid: string;
  title: string;
  slug: string;
  description: string;
  handle: string;
  status: 'draft' | 'active' | 'archived';
  product_type: 'physical' | 'digital' | 'service';
  category_id: number | null;
  tags: string[];
  featured: boolean;
  seo_title: string | null;
  seo_description: string | null;
  created_at: string;  // ISO 8601
  variants?: Variant[];
}

// Variante
type Variant = {
  pid: string;
  sku: string;
  title: string;
  option_values: Record<string, string>;
  inventory_quantity: number;
  allow_backorder: boolean;
  sort_order: number;
  prices?: Price[];
}

// Preço
type Price = {
  pid: string;
  amount: number;        // centavos
  currency: string;      // 'BRL'
  region: string | null;
  min_quantity: number;
  max_quantity: number | null;
}

// Carrinho
type Cart = {
  pid: string;
  session_id: string;
  status: 'active' | 'abandoned' | 'completed';
  currency: string;
  subtotal: number;    // centavos
  tax: number;
  shipping: number;
  total: number;
  items?: CartItem[];
}

// Item do carrinho
type CartItem = {
  pid: string;
  variant_id: number;
  quantity: number;
  unit_price: number;  // centavos
  total: number;       // centavos
}

// Pedido
type Order = {
  pid: string;
  order_number: string;
  status: string;
  payment_status: string;
  fulfillment_status: string;
  currency: string;
  subtotal: number;
  tax: number;
  shipping: number;
  discount: number;
  total: number;
  created_at: string;
  items?: OrderItem[];
}
```

---

## 7. Exemplos de páginas Tera completas

### Layout base (`base.html`)

```html
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% block title %}{{ store.name }}{% endblock %}</title>
  <link rel="stylesheet" href="/static/css/output.css">
  <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
  <script>window.STORE_PID = '{{ store.pid }}';</script>
  <script src="/static/js/store-sdk.js"></script>
</head>
<body class="bg-white text-gray-900">

  <!-- Navbar -->
  <header x-data class="border-b px-6 py-4 flex items-center justify-between">
    <a href="/" class="font-bold text-xl">{{ store.name }}</a>

    <div class="flex items-center gap-6">
      <!-- Busca -->
      {% include "store/components/search.html" %}

      <!-- Conta -->
      <a x-show="!$store.customer.isLoggedIn" href="/login" class="text-sm">Entrar</a>
      <a x-show="$store.customer.isLoggedIn" href="/minha-conta" class="text-sm">
        Olá, <span x-text="$store.customer.data?.first_name"></span>
      </a>

      <!-- Carrinho -->
      <button @click="$store.cart.open()" class="relative">
        🛒
        <span
          x-show="$store.cart.itemCount > 0"
          x-text="$store.cart.itemCount"
          class="absolute -top-2 -right-2 bg-black text-white text-xs rounded-full w-5 h-5 flex items-center justify-center"
        ></span>
      </button>
    </div>
  </header>

  <!-- Gaveta do carrinho -->
  {% include "store/components/cart_drawer.html" %}

  <!-- Toasts -->
  <div x-data class="fixed bottom-4 right-4 z-50 space-y-2">
    <template x-for="t in $store.toasts.items" :key="t.id">
      <div x-show="true" x-transition class="bg-white border shadow-lg rounded-lg px-4 py-3 flex items-center gap-3 max-w-xs">
        <span x-text="t.message" class="text-sm flex-1"></span>
        <button @click="$store.toasts.dismiss(t.id)" class="text-gray-400">✕</button>
      </div>
    </template>
  </div>

  <!-- Conteúdo da página -->
  <main>{% block content %}{% endblock %}</main>

  <footer class="border-t px-6 py-8 text-center text-sm text-gray-500">
    © {{ store.name }} — Todos os direitos reservados.
  </footer>

</body>
</html>
```

### Página de produto (`produto.html`)

```html
{% extends "store/base.html" %}
{% block title %}{{ product.title }} — {{ store.name }}{% endblock %}

{% block content %}
<div class="max-w-6xl mx-auto px-6 py-12"
     x-data="ProductDetail('{{ product.pid }}')"
     x-init="init()">
  <div class="grid grid-cols-1 md:grid-cols-2 gap-12">
    <img :src="product?.image_url ?? '/static/images/placeholder.png'" :alt="product?.title" class="rounded-xl w-full">
    <div>
      <h1 x-text="product?.title" class="text-3xl font-bold"></h1>
      <p x-text="product?.description" class="text-gray-600 mt-4 leading-relaxed"></p>

      <template x-if="variants.length > 1">
        <div class="mt-6 flex gap-2 flex-wrap">
          <template x-for="v in variants" :key="v.pid">
            <button @click="selectVariant(v)"
              :class="selectedVariant?.pid === v.pid ? 'bg-black text-white' : 'border hover:border-black'"
              class="px-4 py-2 rounded-lg text-sm font-medium transition" x-text="v.title">
            </button>
          </template>
        </div>
      </template>

      <p class="text-2xl font-bold mt-6" x-text="selectedPrice"></p>

      <div class="flex items-center gap-4 mt-4">
        <button @click="quantity = Math.max(1, quantity - 1)" class="w-9 h-9 border rounded-lg">−</button>
        <span x-text="quantity"></span>
        <button @click="quantity++" class="w-9 h-9 border rounded-lg">+</button>
      </div>

      <button @click="addToCart()"
        :disabled="$store.cart.loading"
        class="mt-6 w-full bg-black text-white py-4 rounded-xl font-semibold hover:bg-gray-800 disabled:opacity-50 transition">
        Adicionar ao carrinho
      </button>
    </div>
  </div>
</div>
{% endblock %}
```

---

## 8. Exemplo com framework SPA (Vite + Vanilla)

Para projetos que **não usam Tera**, importe o SDK via tag `<script>` e configure o `STORE_PID` antes:

### `index.html`

```html
<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <title>Minha Loja</title>
  <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
  <script>window.STORE_PID = 'uuid-da-loja-aqui';</script>
  <script src="/js/store-sdk.js"></script>
</head>
<body>
  <!-- Use x-data diretamente nos elementos -->
  <div x-data="ProductList()" x-init="init()">
    <template x-for="p in products" :key="p.pid">
      <div x-text="p.title"></div>
    </template>
  </div>
</body>
</html>
```

### Uso em script ES module (sem Alpine, apenas StoreSDK)

```js
// Aguarda o SDK estar disponível
document.addEventListener('DOMContentLoaded', async () => {
  const { data: products } = await StoreSDK.products.list({ featured: true });
  console.log(products);

  const cart = await StoreSDK.cart.getOrCreate();
  await StoreSDK.cart.addItem(cart.pid, { variant_id: 1, quantity: 1 });
});
```

---

## Checklist de adaptação de nova loja

- [ ] Definir `window.STORE_PID` com o UUID da loja
- [ ] Importar Alpine.js antes do `store-sdk.js`
- [ ] Adicionar o bloco de toasts no layout base
- [ ] Adicionar o componente `CartDrawer` no layout
- [ ] Configurar as rotas das páginas (produto, categoria, checkout, conta)
- [ ] Personalizar CSS (Tailwind classes ou substituir)
- [ ] Testar fluxo completo: busca → produto → carrinho → checkout → confirmação
