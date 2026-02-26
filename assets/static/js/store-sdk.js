/**
 * ============================================================
 * Loco Fast Store — SDK Alpine.js para lojas
 * ============================================================
 *
 * USO BÁSICO
 * ----------
 * 1. Configure o STORE_PID (UUID da sua loja) abaixo, OU defina
 *    window.STORE_PID antes de carregar este arquivo.
 * 2. Importe Alpine.js e este arquivo no seu HTML:
 *
 *   <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
 *   <script>window.STORE_PID = 'uuid-da-sua-loja';</script>
 *   <script src="/static/js/store-sdk.js"></script>
 *
 * 3. Use os componentes e stores diretamente nos seus templates.
 *
 * CONVENÇÕES DA API
 * -----------------
 * - Base URL:  /api/stores/{STORE_PID}/...
 * - Respostas: { ok: bool, data: T, meta?: { cursor, has_more, count } }
 * - Valores monetários: centavos inteiros (ex.: 1990 = R$ 19,90)
 * - IDs públicos:       UUID strings (campo "pid")
 * - Autenticação (customer): header  X-Customer-Token: <token>
 *
 * STORES GLOBAIS ALPINE
 * ----------------------
 *   $store.customer   — estado do cliente logado
 *   $store.cart       — carrinho ativo
 *   $store.toasts     — fila de notificações toast
 *
 * COMPONENTES (x-data)
 * ---------------------
 *   ProductList()       — lista de produtos com filtros e paginação
 *   ProductDetail(pid)  — produto individual com variantes e preços
 *   CategoryList()      — árvore de categorias
 *   CollectionDetail(pid) — coleção com seus produtos
 *   CartDrawer()        — gaveta lateral do carrinho
 *   CheckoutForm()      — formulário de checkout completo
 *   CustomerAuth()      — login e cadastro de cliente
 *   CustomerAccount()   — dados do perfil e endereços
 *   SearchBar()         — busca de produtos em tempo real
 * ============================================================
 */

// ─── Configuração global ─────────────────────────────────────────────────────

/** UUID da loja. Pode ser sobrescrito com window.STORE_PID antes de carregar. */
const STORE_PID = window.STORE_PID || '';

/** URL base da API. */
const API_BASE = `/api/stores/${STORE_PID}`;

/** Chave no localStorage para o token de sessão do cliente. */
const SESSION_KEY = `lfs_customer_token_${STORE_PID}`;

/** Chave no localStorage para o pid do carrinho ativo. */
const CART_KEY = `lfs_cart_pid_${STORE_PID}`;

/** Chave no localStorage para a session_id anônima do carrinho. */
const SESSION_ID_KEY = `lfs_session_id_${STORE_PID}`;

// ─── Utilitários ─────────────────────────────────────────────────────────────

/**
 * Gera ou recupera a session_id persistente do visitante.
 * Usada para carrinhos anônimos.
 * @returns {string}
 */
function getSessionId() {
  let sid = localStorage.getItem(SESSION_ID_KEY);
  if (!sid) {
    sid = 'sess_' + Math.random().toString(36).slice(2) + Date.now().toString(36);
    localStorage.setItem(SESSION_ID_KEY, sid);
  }
  return sid;
}

/**
 * Formata centavos para moeda.
 * @param {number} cents - Valor em centavos
 * @param {string} [currency='BRL'] - Código ISO 4217
 * @returns {string} Ex.: "R$ 19,90"
 */
function formatMoney(cents, currency = 'BRL') {
  return new Intl.NumberFormat('pt-BR', {
    style: 'currency',
    currency,
  }).format(cents / 100);
}

/**
 * Formata uma string ISO de data para exibição.
 * @param {string} dateStr
 * @returns {string} Ex.: "25 de jan. de 2026"
 */
function formatDate(dateStr) {
  return new Intl.DateTimeFormat('pt-BR', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  }).format(new Date(dateStr));
}

/**
 * Gera slug a partir de texto.
 * @param {string} text
 * @returns {string}
 */
function slugify(text) {
  return text
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '');
}

// ─── Cliente HTTP ─────────────────────────────────────────────────────────────

/**
 * Cliente HTTP interno. Todos os métodos retornam o campo `data` da resposta.
 * Em caso de erro, lança um objeto { code, message }.
 */
const http = {
  /** @returns {Record<string, string>} */
  _headers() {
    const h = { 'Content-Type': 'application/json' };
    const token = localStorage.getItem(SESSION_KEY);
    if (token) h['X-Customer-Token'] = token;
    return h;
  },

  /**
   * @param {string} path   - Caminho relativo ao API_BASE
   * @param {RequestInit} [opts]
   * @returns {Promise<any>}
   */
  async _fetch(path, opts = {}) {
    const res = await fetch(`${API_BASE}${path}`, {
      ...opts,
      headers: { ...this._headers(), ...opts.headers },
    });
    const body = await res.json().catch(() => ({ ok: false, error: { code: 'PARSE_ERROR', message: 'Resposta inválida do servidor' } }));
    if (!res.ok || body.ok === false) {
      throw body.error || { code: String(res.status), message: 'Erro desconhecido' };
    }
    return body.data ?? body;
  },

  get: (path) => http._fetch(path),
  post: (path, data) => http._fetch(path, { method: 'POST', body: JSON.stringify(data) }),
  put: (path, data) => http._fetch(path, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (path) => http._fetch(path, { method: 'DELETE' }),

  /** GET com paginação — retorna { data, meta } */
  async getPaginated(path) {
    const res = await fetch(`${API_BASE}${path}`, { headers: http._headers() });
    const body = await res.json().catch(() => ({}));
    if (!res.ok || body.ok === false) throw body.error || { code: String(res.status), message: 'Erro' };
    return { data: body.data ?? [], meta: body.meta ?? null };
  },
};

// ─── Módulos de API ──────────────────────────────────────────────────────────

/**
 * @namespace StoreSDK
 * API pública. Todos os métodos são assíncronos.
 */
window.StoreSDK = {

  // ── Autenticação de cliente ─────────────────────────────────────────────

  auth: {
    /**
     * Cadastra novo cliente na loja.
     * POST /api/stores/{pid}/auth/register
     * @param {{ email: string, password: string, first_name: string, last_name: string, phone?: string, marketing_consent?: boolean }} params
     * @returns {Promise<{ token: string, customer: object }>}
     */
    register: (params) => http.post('/auth/register', params),

    /**
     * Autentica cliente e armazena token no localStorage.
     * POST /api/stores/{pid}/auth/login
     * @param {{ email: string, password: string }} params
     * @returns {Promise<{ token: string, customer: object }>}
     */
    async login(params) {
      const data = await http.post('/auth/login', params);
      if (data.token) localStorage.setItem(SESSION_KEY, data.token);
      return data;
    },

    /**
     * Encerra a sessão do cliente (remove token).
     * POST /api/stores/{pid}/auth/logout
     * @returns {Promise<void>}
     */
    async logout() {
      await http.post('/auth/logout', {}).catch(() => {});
      localStorage.removeItem(SESSION_KEY);
    },

    /**
     * Retorna os dados do cliente autenticado.
     * GET /api/stores/{pid}/auth/me
     * @returns {Promise<object>}
     */
    me: () => http.get('/auth/me'),

    /**
     * Verifica se há token de sessão armazenado.
     * @returns {boolean}
     */
    isLoggedIn: () => !!localStorage.getItem(SESSION_KEY),
  },

  // ── Produtos ────────────────────────────────────────────────────────────

  products: {
    /**
     * Lista produtos com filtros opcionais e paginação por cursor.
     * GET /api/stores/{pid}/products?status=&category_id=&featured=&q=&limit=&cursor=
     * @param {{ status?: string, category_id?: number, featured?: boolean, q?: string, limit?: number, cursor?: string }} [params]
     * @returns {Promise<{ data: object[], meta: object }>}
     */
    list(params = {}) {
      const qs = new URLSearchParams(Object.entries(params).filter(([, v]) => v !== undefined && v !== '')).toString();
      return http.getPaginated(`/products${qs ? '?' + qs : ''}`);
    },

    /**
     * Busca produto completo com variantes e preços.
     * GET /api/stores/{pid}/products/{pid}
     * @param {string} pid - UUID do produto
     * @returns {Promise<object>}
     */
    get: (pid) => http.get(`/products/${pid}`),

    /**
     * Exporta todos os produtos da loja como CSV.
     * Redireciona o browser para download.
     */
    exportCsv() {
      window.location.href = `${API_BASE}/products/export/csv`;
    },

    /**
     * Baixa o CSV template para importação em lote.
     */
    downloadTemplate() {
      window.location.href = `${API_BASE}/products/import/template`;
    },
  },

  // ── Categorias ──────────────────────────────────────────────────────────

  categories: {
    /**
     * Lista todas as categorias da loja (opcionalmente filtradas por pai).
     * GET /api/stores/{pid}/categories?parent_id=
     * @param {{ parent_id?: number }} [params]
     * @returns {Promise<object[]>}
     */
    async list(params = {}) {
      const qs = params.parent_id ? `?parent_id=${params.parent_id}` : '';
      const res = await http.get(`/categories${qs}`);
      return Array.isArray(res) ? res : res;
    },

    /**
     * Busca categoria pelo PID.
     * GET /api/stores/{pid}/categories/{pid}
     * @param {string} pid
     * @returns {Promise<object>}
     */
    get: (pid) => http.get(`/categories/${pid}`),
  },

  // ── Coleções ────────────────────────────────────────────────────────────

  collections: {
    /**
     * Lista todas as coleções publicadas.
     * GET /api/stores/{pid}/collections
     * @returns {Promise<object[]>}
     */
    async list() {
      const res = await http.get('/collections');
      return Array.isArray(res) ? res : res;
    },

    /**
     * Busca coleção com seus produtos.
     * GET /api/stores/{pid}/collections/{pid}
     * @param {string} pid
     * @returns {Promise<object>}
     */
    get: (pid) => http.get(`/collections/${pid}`),
  },

  // ── Carrinho ────────────────────────────────────────────────────────────

  cart: {
    /**
     * Obtém ou cria o carrinho ativo da sessão.
     * POST /api/stores/{pid}/carts?session_id=
     * @returns {Promise<object>} Carrinho com items[]
     */
    async getOrCreate() {
      const sessionId = getSessionId();
      const cart = await http.getPaginated(`/carts?session_id=${sessionId}`)
        .catch(() => null);
      if (cart && cart.data && cart.data.pid) return cart.data;
      // Endpoint retorna carrinho diretamente via POST
      return http.post('/carts', {}).catch(async () => {
        // fallback: GET com session_id via query (dependendo de como o backend trata)
        return http.get(`/carts?session_id=${sessionId}`);
      });
    },

    /**
     * Busca carrinho pelo PID (UUID).
     * GET /api/stores/{pid}/carts/{cart_pid}
     * @param {string} cartPid
     * @returns {Promise<object>}
     */
    get: (cartPid) => http.get(`/carts/${cartPid}`),

    /**
     * Adiciona ou incrementa item no carrinho.
     * POST /api/stores/{pid}/carts/{cart_pid}/items
     * @param {string} cartPid
     * @param {{ variant_id: number, quantity: number }} params
     * @returns {Promise<object>} Carrinho atualizado
     */
    addItem: (cartPid, params) => http.post(`/carts/${cartPid}/items`, params),

    /**
     * Atualiza quantidade de item. Quantidade 0 remove o item.
     * PUT /api/stores/{pid}/carts/{cart_pid}/items/{item_id}
     * @param {string} cartPid
     * @param {number} itemId
     * @param {number} quantity
     * @returns {Promise<object>} Carrinho atualizado
     */
    updateItem: (cartPid, itemId, quantity) =>
      http.put(`/carts/${cartPid}/items/${itemId}`, { quantity }),

    /**
     * Remove item do carrinho.
     * DELETE /api/stores/{pid}/carts/{cart_pid}/items/{item_id}
     * @param {string} cartPid
     * @param {number} itemId
     * @returns {Promise<object>} Carrinho atualizado
     */
    removeItem: (cartPid, itemId) => http.delete(`/carts/${cartPid}/items/${itemId}`),
  },

  // ── Pedidos ─────────────────────────────────────────────────────────────

  orders: {
    /**
     * Cria pedido a partir do carrinho ativo.
     * POST /api/stores/{pid}/orders
     * @param {{ customer_id: number, shipping_address_id?: number, billing_address_id?: number, payment_method?: string, notes?: string }} params
     * @returns {Promise<object>} Pedido criado com items[]
     */
    create: (params) => http.post('/orders', params),

    /**
     * Busca pedido pelo PID.
     * GET /api/stores/{pid}/orders/{order_pid}
     * @param {string} pid
     * @returns {Promise<object>}
     */
    get: (pid) => http.get(`/orders/${pid}`),
  },

  // ── Cliente (perfil) ────────────────────────────────────────────────────

  customer: {
    /**
     * Busca dados do cliente pelo PID.
     * GET /api/stores/{pid}/customers/{customer_pid}
     * @param {string} pid
     * @returns {Promise<object>}
     */
    get: (pid) => http.get(`/customers/${pid}`),

    /**
     * Atualiza dados do perfil.
     * PUT /api/stores/{pid}/customers/{customer_pid}
     * @param {string} pid
     * @param {{ first_name?: string, last_name?: string, phone?: string, marketing_consent?: boolean }} params
     * @returns {Promise<object>}
     */
    update: (pid, params) => http.put(`/customers/${pid}`, params),

    /**
     * Lista endereços do cliente.
     * GET /api/stores/{pid}/customers/{customer_pid}/addresses
     * @param {string} pid
     * @returns {Promise<object[]>}
     */
    addresses: (pid) => http.get(`/customers/${pid}/addresses`),

    /**
     * Adiciona endereço ao cliente.
     * POST /api/stores/{pid}/customers/{customer_pid}/addresses
     * @param {string} pid
     * @param {{ first_name: string, last_name: string, address_line_1: string, city: string, state: string, postal_code: string, country?: string, phone?: string, is_default_shipping?: boolean, is_default_billing?: boolean }} params
     * @returns {Promise<object>}
     */
    addAddress: (pid, params) => http.post(`/customers/${pid}/addresses`, params),
  },

  // ── Helpers ─────────────────────────────────────────────────────────────

  /** Formata centavos para moeda. Ex.: formatMoney(1990) → "R$ 19,90" */
  formatMoney,
  /** Formata data ISO para pt-BR. */
  formatDate,
  /** Gera slug a partir de texto. */
  slugify,
};

// ─── Alpine.js Stores globais ─────────────────────────────────────────────────

document.addEventListener('alpine:init', () => {

  // ── $store.toasts ──────────────────────────────────────────────────────

  /**
   * Fila de notificações toast.
   *
   * Template de exemplo (coloque no <body>):
   *
   *   <div x-data class="fixed bottom-4 right-4 z-50 space-y-2">
   *     <template x-for="t in $store.toasts.items" :key="t.id">
   *       <div x-show="true" class="bg-white shadow rounded p-3 flex items-center gap-2">
   *         <span x-text="t.message"></span>
   *       </div>
   *     </template>
   *   </div>
   */
  Alpine.store('toasts', {
    items: [],
    /**
     * Exibe um toast.
     * @param {string} message
     * @param {'success'|'error'|'warning'|'info'} [type]
     */
    show(message, type = 'info') {
      const id = Date.now();
      this.items.push({ id, message, type });
      setTimeout(() => this.dismiss(id), 4500);
    },
    success: (msg) => Alpine.store('toasts').show(msg, 'success'),
    error: (msg) => Alpine.store('toasts').show(msg, 'error'),
    warning: (msg) => Alpine.store('toasts').show(msg, 'warning'),
    info: (msg) => Alpine.store('toasts').show(msg, 'info'),
    dismiss(id) { this.items = this.items.filter(t => t.id !== id); },
  });

  // ── $store.customer ────────────────────────────────────────────────────

  /**
   * Estado do cliente logado.
   *
   * Uso:
   *   <span x-show="$store.customer.isLoggedIn" x-text="$store.customer.data?.first_name"></span>
   *   <button @click="$store.customer.logout()">Sair</button>
   */
  Alpine.store('customer', {
    isLoggedIn: StoreSDK.auth.isLoggedIn(),
    data: null,
    loading: false,

    /** Carrega dados do cliente autenticado. Chame no init da página. */
    async fetch() {
      if (!this.isLoggedIn) return;
      this.loading = true;
      try {
        this.data = await StoreSDK.auth.me();
      } catch {
        this.logout();
      } finally {
        this.loading = false;
      }
    },

    /** Efetua login e atualiza o store. */
    async login(email, password) {
      const result = await StoreSDK.auth.login({ email, password });
      this.isLoggedIn = true;
      this.data = result.customer;
      return result;
    },

    /** Cadastra novo cliente. */
    async register(params) {
      const result = await StoreSDK.auth.register(params);
      if (result.token) {
        localStorage.setItem(SESSION_KEY, result.token);
        this.isLoggedIn = true;
        this.data = result.customer;
      }
      return result;
    },

    /** Encerra sessão. */
    async logout() {
      await StoreSDK.auth.logout();
      this.isLoggedIn = false;
      this.data = null;
    },
  });

  // ── $store.cart ────────────────────────────────────────────────────────

  /**
   * Estado do carrinho ativo.
   *
   * Uso:
   *   <span x-text="$store.cart.itemCount + ' itens'"></span>
   *   <span x-text="StoreSDK.formatMoney($store.cart.data?.total ?? 0)"></span>
   *   <button @click="$store.cart.open()">Abrir carrinho</button>
   */
  Alpine.store('cart', {
    data: null,
    isOpen: false,
    loading: false,

    /** Total de itens (soma de quantidades). */
    get itemCount() {
      if (!this.data?.items) return 0;
      return this.data.items.reduce((acc, i) => acc + i.quantity, 0);
    },

    /** Carrega ou cria o carrinho da sessão. */
    async init() {
      const savedPid = localStorage.getItem(CART_KEY);
      if (savedPid) {
        try {
          this.data = await StoreSDK.cart.get(savedPid);
          return;
        } catch { localStorage.removeItem(CART_KEY); }
      }
      await this.refresh();
    },

    /** Cria/recarrega o carrinho. */
    async refresh() {
      this.loading = true;
      try {
        const sessionId = getSessionId();
        // O backend aceita session_id como query param no POST
        this.data = await http._fetch(`/carts?session_id=${sessionId}`, { method: 'POST', body: '{}' })
          .catch(() => http.get(`/carts?session_id=${sessionId}`));
        if (this.data?.pid) localStorage.setItem(CART_KEY, this.data.pid);
      } catch { /* silencioso */ }
      finally { this.loading = false; }
    },

    /** Adiciona produto ao carrinho. */
    async addItem(variantId, quantity = 1) {
      if (!this.data?.pid) await this.refresh();
      this.loading = true;
      try {
        this.data = await StoreSDK.cart.addItem(this.data.pid, { variant_id: variantId, quantity });
        Alpine.store('toasts').success('Produto adicionado ao carrinho!');
        this.isOpen = true;
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Erro ao adicionar produto');
      } finally { this.loading = false; }
    },

    /** Atualiza quantidade de um item. */
    async updateItem(itemId, quantity) {
      this.loading = true;
      try {
        this.data = await StoreSDK.cart.updateItem(this.data.pid, itemId, quantity);
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Erro ao atualizar item');
      } finally { this.loading = false; }
    },

    /** Remove item do carrinho. */
    async removeItem(itemId) {
      this.loading = true;
      try {
        this.data = await StoreSDK.cart.removeItem(this.data.pid, itemId);
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Erro ao remover item');
      } finally { this.loading = false; }
    },

    open() { this.isOpen = true; },
    close() { this.isOpen = false; },
  });

});

// ─── Componentes Alpine.js (factories) ───────────────────────────────────────

/**
 * Lista de produtos com filtros, busca e paginação por cursor.
 *
 * Uso:
 *   <div x-data="ProductList({ status: 'active', limit: 12 })" x-init="init()">
 *     <template x-for="p in products" :key="p.pid">
 *       <div x-text="p.title"></div>
 *     </template>
 *     <button @click="loadMore()" x-show="hasMore">Carregar mais</button>
 *   </div>
 *
 * @param {{ status?: string, category_id?: number, featured?: boolean, q?: string, limit?: number }} [defaults]
 */
window.ProductList = function (defaults = {}) {
  return {
    products: [],
    loading: false,
    hasMore: false,
    cursor: null,
    filters: {
      status: defaults.status ?? 'active',
      category_id: defaults.category_id ?? '',
      featured: defaults.featured ?? '',
      q: defaults.q ?? '',
      limit: defaults.limit ?? 20,
    },

    async init() { await this.fetch(true); },

    /** Busca (re)inicializando a lista. */
    async fetch(reset = true) {
      if (reset) { this.products = []; this.cursor = null; }
      this.loading = true;
      try {
        const params = { ...this.filters };
        if (this.cursor) params.cursor = this.cursor;
        // Remove valores vazios
        Object.keys(params).forEach(k => (params[k] === '' || params[k] === undefined) && delete params[k]);
        const { data, meta } = await StoreSDK.products.list(params);
        this.products = reset ? data : [...this.products, ...data];
        this.hasMore = meta?.has_more ?? false;
        this.cursor = meta?.cursor ?? null;
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Erro ao carregar produtos');
      } finally { this.loading = false; }
    },

    /** Carrega próxima página. */
    async loadMore() { if (this.hasMore && !this.loading) await this.fetch(false); },

    /** Aplica filtro e reinicia lista. */
    async applyFilter(key, value) { this.filters[key] = value; await this.fetch(true); },

    /** Atualiza busca por texto (debounce recomendado no template). */
    async search(q) { this.filters.q = q; await this.fetch(true); },
  };
};

/**
 * Detalhe de produto único com variantes e seleção de opções.
 *
 * Uso:
 *   <div x-data="ProductDetail('uuid-do-produto')" x-init="init()">
 *     <h1 x-text="product?.title"></h1>
 *     <template x-for="v in variants" :key="v.pid">
 *       <button @click="selectVariant(v)" x-text="v.title"></button>
 *     </template>
 *     <span x-text="selectedPrice"></span>
 *     <button @click="addToCart()">Adicionar ao carrinho</button>
 *   </div>
 *
 * @param {string} productPid - UUID do produto
 */
window.ProductDetail = function (productPid) {
  return {
    product: null,
    variants: [],
    selectedVariant: null,
    quantity: 1,
    loading: false,

    async init() {
      this.loading = true;
      try {
        this.product = await StoreSDK.products.get(productPid);
        this.variants = this.product.variants ?? [];
        if (this.variants.length > 0) this.selectedVariant = this.variants[0];
      } catch (e) {
        Alpine.store('toasts').error('Produto não encontrado');
      } finally { this.loading = false; }
    },

    /** Preço formatado da variante selecionada. */
    get selectedPrice() {
      const prices = this.selectedVariant?.prices ?? [];
      const price = prices.find(p => p.min_quantity <= this.quantity) ?? prices[0];
      return price ? StoreSDK.formatMoney(price.amount, price.currency) : 'Consultar';
    },

    /** Seleciona uma variante. */
    selectVariant(variant) { this.selectedVariant = variant; },

    /** Adiciona variante selecionada ao carrinho. */
    async addToCart() {
      if (!this.selectedVariant) {
        Alpine.store('toasts').warning('Selecione uma variante');
        return;
      }
      await Alpine.store('cart').addItem(this.selectedVariant.id, this.quantity);
    },
  };
};

/**
 * Lista de categorias com suporte a hierarquia.
 *
 * Uso:
 *   <div x-data="CategoryList()" x-init="init()">
 *     <template x-for="c in categories" :key="c.pid">
 *       <a :href="'/categoria/' + c.slug" x-text="c.name"></a>
 *     </template>
 *   </div>
 *
 * @param {{ parent_id?: number }} [opts]
 */
window.CategoryList = function (opts = {}) {
  return {
    categories: [],
    loading: false,

    async init() {
      this.loading = true;
      try {
        this.categories = await StoreSDK.categories.list(opts);
      } catch (e) {
        Alpine.store('toasts').error('Erro ao carregar categorias');
      } finally { this.loading = false; }
    },
  };
};

/**
 * Detalhe de coleção com seus produtos.
 *
 * Uso:
 *   <div x-data="CollectionDetail('uuid-da-colecao')" x-init="init()">
 *     <h2 x-text="collection?.title"></h2>
 *     <template x-for="p in products" :key="p.pid">
 *       ...
 *     </template>
 *   </div>
 *
 * @param {string} collectionPid - UUID da coleção
 */
window.CollectionDetail = function (collectionPid) {
  return {
    collection: null,
    products: [],
    loading: false,

    async init() {
      this.loading = true;
      try {
        this.collection = await StoreSDK.collections.get(collectionPid);
        // O endpoint retorna produtos embutidos em collection.products
        this.products = this.collection.products ?? [];
      } catch (e) {
        Alpine.store('toasts').error('Coleção não encontrada');
      } finally { this.loading = false; }
    },
  };
};

/**
 * Gaveta lateral do carrinho.
 *
 * Uso:
 *   <div x-data="CartDrawer()" x-init="init()">
 *     <!-- Botão flutuante -->
 *     <button @click="$store.cart.open()">
 *       Carrinho (<span x-text="$store.cart.itemCount"></span>)
 *     </button>
 *
 *     <!-- Gaveta -->
 *     <div x-show="$store.cart.isOpen" @click.outside="$store.cart.close()">
 *       <template x-for="item in $store.cart.data?.items ?? []" :key="item.pid">
 *         <div>
 *           <span x-text="item.quantity + 'x'"></span>
 *           <span x-text="formatMoney(item.total)"></span>
 *           <button @click="$store.cart.removeItem(item.id)">✕</button>
 *         </div>
 *       </template>
 *       <strong x-text="formatMoney($store.cart.data?.total ?? 0)"></strong>
 *       <a href="/checkout">Finalizar compra</a>
 *     </div>
 *   </div>
 */
window.CartDrawer = function () {
  return {
    async init() {
      await Alpine.store('cart').init();
    },
    formatMoney: StoreSDK.formatMoney,
  };
};

/**
 * Formulário de checkout.
 *
 * Uso:
 *   <div x-data="CheckoutForm()" x-init="init()">
 *     <!-- Etapa 1: Email -->
 *     <input x-model="form.email" type="email" placeholder="Seu e-mail">
 *
 *     <!-- Etapa 2: Endereço -->
 *     <input x-model="form.address.first_name" placeholder="Nome">
 *     <input x-model="form.address.postal_code" @blur="fetchAddress()" placeholder="CEP">
 *
 *     <!-- Resumo -->
 *     <span x-text="formatMoney($store.cart.data?.total ?? 0)"></span>
 *
 *     <!-- Finalizar -->
 *     <button @click="submit()" :disabled="loading">
 *       <span x-show="!loading">Finalizar pedido</span>
 *       <span x-show="loading">Processando...</span>
 *     </button>
 *   </div>
 */
window.CheckoutForm = function () {
  return {
    step: 1,          // 1: identificação | 2: endereço | 3: pagamento | 4: confirmação
    loading: false,
    order: null,
    form: {
      email: '',
      first_name: '',
      last_name: '',
      phone: '',
      address: {
        first_name: '',
        last_name: '',
        address_line_1: '',
        address_line_2: '',
        city: '',
        state: '',
        postal_code: '',
        country: 'BR',
        phone: '',
      },
      payment_method: 'pix',
      notes: '',
    },

    async init() {
      await Alpine.store('cart').init();
      // Preenche email se cliente já estiver logado
      if (Alpine.store('customer').data) {
        const c = Alpine.store('customer').data;
        this.form.email = c.email ?? '';
        this.form.first_name = c.first_name ?? '';
        this.form.last_name = c.last_name ?? '';
      }
    },

    /**
     * Consulta CEP via ViaCEP e preenche endereço.
     * Requer acesso à API pública viacep.com.br.
     */
    async fetchAddress() {
      const cep = this.form.address.postal_code.replace(/\D/g, '');
      if (cep.length !== 8) return;
      try {
        const res = await fetch(`https://viacep.com.br/ws/${cep}/json/`);
        const data = await res.json();
        if (!data.erro) {
          this.form.address.address_line_1 = data.logradouro ?? '';
          this.form.address.city = data.localidade ?? '';
          this.form.address.state = data.uf ?? '';
        }
      } catch { /* silencioso */ }
    },

    /** Avança para o próximo passo. */
    nextStep() { this.step = Math.min(this.step + 1, 3); },
    prevStep() { this.step = Math.max(this.step - 1, 1); },

    /** Submete o pedido. */
    async submit() {
      this.loading = true;
      try {
        // 1. Garante ou cria customer
        let customer = Alpine.store('customer').data;
        if (!customer) {
          try {
            const reg = await StoreSDK.auth.register({
              email: this.form.email,
              first_name: this.form.first_name,
              last_name: this.form.last_name,
              phone: this.form.phone,
              password: Math.random().toString(36).slice(2, 10), // senha aleatória para guest
            });
            customer = reg.customer;
            if (reg.token) localStorage.setItem(SESSION_KEY, reg.token);
          } catch {
            // cliente já existe — tenta login ou usa anônimo
          }
        }

        // 2. Adiciona endereço se necessário
        let addressId = null;
        if (customer?.pid) {
          const addr = await StoreSDK.customer.addAddress(customer.pid, {
            ...this.form.address,
            is_default_shipping: true,
          });
          addressId = addr.id;
        }

        // 3. Cria o pedido
        this.order = await StoreSDK.orders.create({
          customer_id: customer?.id,
          shipping_address_id: addressId,
          payment_method: this.form.payment_method,
          notes: this.form.notes,
        });

        this.step = 4; // confirmação
        localStorage.removeItem(CART_KEY);
        Alpine.store('cart').data = null;
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Erro ao finalizar pedido');
      } finally { this.loading = false; }
    },

    formatMoney: StoreSDK.formatMoney,
  };
};

/**
 * Login e cadastro de cliente.
 *
 * Uso:
 *   <div x-data="CustomerAuth()" x-init="init()">
 *     <!-- Login -->
 *     <div x-show="mode === 'login'">
 *       <input x-model="form.email" type="email" placeholder="E-mail">
 *       <input x-model="form.password" type="password" placeholder="Senha">
 *       <button @click="login()">Entrar</button>
 *       <a @click="mode = 'register'">Criar conta</a>
 *     </div>
 *
 *     <!-- Cadastro -->
 *     <div x-show="mode === 'register'">
 *       <input x-model="form.email" type="email">
 *       <input x-model="form.first_name" placeholder="Nome">
 *       <input x-model="form.last_name" placeholder="Sobrenome">
 *       <input x-model="form.password" type="password">
 *       <button @click="register()">Criar conta</button>
 *       <a @click="mode = 'login'">Já tenho conta</a>
 *     </div>
 *   </div>
 *
 * @param {{ redirectOnLogin?: string }} [opts]
 */
window.CustomerAuth = function (opts = {}) {
  return {
    mode: 'login',    // 'login' | 'register'
    loading: false,
    form: {
      email: '',
      password: '',
      first_name: '',
      last_name: '',
      phone: '',
      marketing_consent: false,
    },

    init() {
      if (Alpine.store('customer').isLoggedIn && opts.redirectOnLogin) {
        window.location.href = opts.redirectOnLogin;
      }
    },

    async login() {
      this.loading = true;
      try {
        await Alpine.store('customer').login(this.form.email, this.form.password);
        Alpine.store('toasts').success('Bem-vindo de volta!');
        if (opts.redirectOnLogin) window.location.href = opts.redirectOnLogin;
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Credenciais inválidas');
      } finally { this.loading = false; }
    },

    async register() {
      this.loading = true;
      try {
        await Alpine.store('customer').register(this.form);
        Alpine.store('toasts').success('Conta criada com sucesso!');
        if (opts.redirectOnLogin) window.location.href = opts.redirectOnLogin;
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Erro ao criar conta');
      } finally { this.loading = false; }
    },
  };
};

/**
 * Página de conta do cliente (perfil + endereços + pedidos).
 *
 * Uso:
 *   <div x-data="CustomerAccount()" x-init="init()">
 *     <p x-text="profile?.first_name + ' ' + profile?.last_name"></p>
 *     <template x-for="a in addresses" :key="a.pid">
 *       <div x-text="a.address_line_1 + ', ' + a.city"></div>
 *     </template>
 *     <button @click="logout()">Sair</button>
 *   </div>
 */
window.CustomerAccount = function () {
  return {
    profile: null,
    addresses: [],
    loading: false,

    async init() {
      if (!Alpine.store('customer').isLoggedIn) {
        window.location.href = '/login';
        return;
      }
      this.loading = true;
      try {
        await Alpine.store('customer').fetch();
        this.profile = Alpine.store('customer').data;
        if (this.profile?.pid) {
          this.addresses = await StoreSDK.customer.addresses(this.profile.pid);
        }
      } catch { /* silencioso */ }
      finally { this.loading = false; }
    },

    async logout() {
      await Alpine.store('customer').logout();
      window.location.href = '/';
    },

    async addAddress(params) {
      try {
        const addr = await StoreSDK.customer.addAddress(this.profile.pid, params);
        this.addresses.push(addr);
        Alpine.store('toasts').success('Endereço adicionado!');
      } catch (e) {
        Alpine.store('toasts').error(e.message || 'Erro ao adicionar endereço');
      }
    },

    formatMoney: StoreSDK.formatMoney,
    formatDate: StoreSDK.formatDate,
  };
};

/**
 * Barra de busca de produtos com debounce.
 *
 * Uso:
 *   <div x-data="SearchBar()" @keydown.escape.window="close()">
 *     <input x-ref="input" x-model="query" @input.debounce.300ms="search()" placeholder="Buscar...">
 *     <ul x-show="results.length > 0">
 *       <template x-for="p in results" :key="p.pid">
 *         <li><a :href="'/produto/' + p.slug" x-text="p.title"></a></li>
 *       </template>
 *     </ul>
 *   </div>
 */
window.SearchBar = function () {
  return {
    query: '',
    results: [],
    loading: false,
    open: false,

    async search() {
      if (this.query.trim().length < 2) { this.results = []; return; }
      this.loading = true;
      try {
        const { data } = await StoreSDK.products.list({ q: this.query, limit: 8 });
        this.results = data;
        this.open = data.length > 0;
      } catch { this.results = []; }
      finally { this.loading = false; }
    },

    close() { this.open = false; this.results = []; },
  };
};

// ─── Inicialização automática ─────────────────────────────────────────────────

document.addEventListener('alpine:init', () => {
  // Carrega dados do cliente logado ao iniciar a página
  Alpine.store('customer').fetch().catch(() => {});
});
