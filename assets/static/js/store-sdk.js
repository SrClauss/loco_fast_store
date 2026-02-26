/**
 * ============================================================
 *  Loco Fast Store — store-sdk.js  v2.0
 * ============================================================
 *
 * ARQUIVO ÚNICO. Importe Alpine.js + este arquivo e pronto.
 *
 *   <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
 *   <script>window.STORE_PID = '{{ store.pid }}';</script>
 *   <script src="/static/js/store-sdk.js"></script>
 *
 * Toda lógica de negócio está aqui. Os templates Tera/HTML
 * usam apenas atributos Alpine (x-data, x-show, @click…).
 * ============================================================
 */

// ─── Configuração ─────────────────────────────────────────────────────────────
const _PID  = window.STORE_PID || '';
const _BASE = `/api/stores/${_PID}`;
const _K    = {
  ctoken : `lfs_ctoken_${_PID}`,
  cart   : `lfs_cart_${_PID}`,
  sid    : `lfs_sid_${_PID}`,
  wish   : `lfs_wish_${_PID}`,
};

// ─── Utilitários públicos ─────────────────────────────────────────────────────

/** Formata centavos → moeda (padrão BRL). Ex.: 1990 → "R$ 19,90" */
function fmtMoney(cents, currency) {
  currency = currency || 'BRL';
  return new Intl.NumberFormat('pt-BR', { style: 'currency', currency }).format((cents || 0) / 100);
}

/** Formata ISO → data pt-BR. Ex.: "25 jan. 2026" */
function fmtDate(s) {
  if (!s) return '—';
  return new Intl.DateTimeFormat('pt-BR', { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(s));
}

/** Formata ISO → data+hora pt-BR. */
function fmtDateTime(s) {
  if (!s) return '—';
  return new Intl.DateTimeFormat('pt-BR', {
    day: '2-digit', month: '2-digit', year: 'numeric',
    hour: '2-digit', minute: '2-digit'
  }).format(new Date(s));
}

/** Session ID anônima persistente para carrinhos sem login. */
function _sid() {
  let v = localStorage.getItem(_K.sid);
  if (!v) {
    v = 'sid_' + Math.random().toString(36).slice(2) + Date.now().toString(36);
    localStorage.setItem(_K.sid, v);
  }
  return v;
}

/** Gera senha criptograficamente segura para clientes guest. */
function _guestPw() {
  const a = new Uint8Array(16);
  crypto.getRandomValues(a);
  return Array.from(a).map(b => b.toString(16).padStart(2,'0')).join('');
}

/** Consulta CEP via ViaCEP. Retorna null se inválido. */
async function cepLookup(cep) {
  const d = cep.replace(/\D/g, '');
  if (d.length !== 8) return null;
  try {
    const r = await fetch(`https://viacep.com.br/ws/${d}/json/`);
    const j = await r.json();
    return j.erro ? null : j;
  } catch { return null; }
}

/** Slugifica texto em português. */
function slugify(t) {
  return t.toLowerCase()
    .normalize('NFD').replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
}

// ─── Tabelas de labels e cores de status ──────────────────────────────────────
const STATUS_LABEL = {
  pending: 'Pendente', confirmed: 'Confirmado', processing: 'Processando',
  shipped: 'Enviado', delivered: 'Entregue', cancelled: 'Cancelado',
  awaiting: 'Aguardando', paid: 'Pago', failed: 'Falhou', refunded: 'Reembolsado',
  not_fulfilled: 'Não enviado', fulfilled: 'Enviado', partially_fulfilled: 'Parcial',
  active: 'Ativo', draft: 'Rascunho', archived: 'Arquivado',
};

const STATUS_CLASS = {
  pending: 'bg-yellow-100 text-yellow-800',
  confirmed: 'bg-blue-100 text-blue-800',
  processing: 'bg-blue-100 text-blue-800',
  shipped: 'bg-indigo-100 text-indigo-800',
  delivered: 'bg-green-100 text-green-800',
  cancelled: 'bg-red-100 text-red-800',
  awaiting: 'bg-yellow-100 text-yellow-800',
  paid: 'bg-green-100 text-green-800',
  failed: 'bg-red-100 text-red-800',
  not_fulfilled: 'bg-orange-100 text-orange-800',
  fulfilled: 'bg-indigo-100 text-indigo-800',
  active: 'bg-green-100 text-green-800',
  draft: 'bg-yellow-100 text-yellow-800',
  archived: 'bg-gray-100 text-gray-700',
};

function statusLabel(s) { return STATUS_LABEL[s] || s; }
function statusClass(s)  { return STATUS_CLASS[s]  || 'bg-gray-100 text-gray-700'; }

// ─── Cliente HTTP interno ─────────────────────────────────────────────────────
const _http = {
  _headers() {
    const h = { 'Content-Type': 'application/json' };
    const t = localStorage.getItem(_K.ctoken);
    if (t) h['X-Customer-Token'] = t;
    return h;
  },

  async _fetch(path, opts) {
    opts = opts || {};
    const res = await fetch(`${_BASE}${path}`, {
      ...opts, headers: { ...this._headers(), ...(opts.headers || {}) }
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok || body.ok === false) throw body.error || { code: String(res.status), message: 'Erro de servidor' };
    return body.data !== undefined ? body.data : body;
  },

  get:  function(p)    { return this._fetch(p); },
  post: function(p, d) { return this._fetch(p, { method: 'POST', body: JSON.stringify(d) }); },
  put:  function(p, d) { return this._fetch(p, { method: 'PUT',  body: JSON.stringify(d) }); },
  del:  function(p)    { return this._fetch(p, { method: 'DELETE' }); },

  async paginate(path) {
    const res = await fetch(`${_BASE}${path}`, { headers: this._headers() });
    const b   = await res.json().catch(() => ({}));
    if (!res.ok || b.ok === false) throw b.error || {};
    return { data: b.data || [], meta: b.meta || null };
  },
};

// ─── StoreSDK — API pública ───────────────────────────────────────────────────
window.StoreSDK = {
  // utilitários expostos
  fmtMoney, fmtDate, fmtDateTime, slugify, cepLookup,
  statusLabel, statusClass,

  // ── Autenticação de cliente ─────────────────────────────────────────────
  auth: {
    isLoggedIn() { return !!localStorage.getItem(_K.ctoken); },

    async login(params) {
      const d = await _http.post('/auth/login', params);
      if (d && d.token) localStorage.setItem(_K.ctoken, d.token);
      return d;
    },

    async register(params) {
      const d = await _http.post('/auth/register', params);
      if (d && d.token) localStorage.setItem(_K.ctoken, d.token);
      return d;
    },

    async logout() {
      await _http.post('/auth/logout', {}).catch(() => {});
      localStorage.removeItem(_K.ctoken);
    },

    me() { return _http.get('/auth/me'); },
  },

  // ── Produtos ────────────────────────────────────────────────────────────
  products: {
    list(params) {
      params = params || {};
      const qs = new URLSearchParams(
        Object.fromEntries(Object.entries(params).filter(function([,v]){ return v !== '' && v != null; }))
      );
      return _http.paginate('/products' + (qs.toString() ? '?' + qs : ''));
    },
    get(pid) { return _http.get('/products/' + pid); },
    exportCsv() { window.location.href = _BASE + '/products/export/csv'; },
  },

  // ── Categorias ──────────────────────────────────────────────────────────
  categories: {
    list(params) {
      params = params || {};
      const qs = new URLSearchParams(
        Object.fromEntries(Object.entries(params).filter(function([,v]){ return !!v; }))
      );
      return _http.get('/categories' + (qs.toString() ? '?' + qs : ''));
    },
    get(pid) { return _http.get('/categories/' + pid); },
  },

  // ── Coleções ────────────────────────────────────────────────────────────
  collections: {
    list() { return _http.get('/collections'); },
    get(pid) { return _http.get('/collections/' + pid); },
  },

  // ── Carrinho ────────────────────────────────────────────────────────────
  cart: {
    async getOrCreate() {
      const saved = localStorage.getItem(_K.cart);
      if (saved) {
        try {
          const c = await _http.get('/carts/' + saved);
          if (c && c.pid) return c;
        } catch(e) { localStorage.removeItem(_K.cart); }
      }
      const sid = _sid();
      let c;
      try {
        c = await _http._fetch('/carts?session_id=' + sid, { method: 'POST', body: '{}' });
      } catch(e) {
        c = await _http.get('/carts?session_id=' + sid);
      }
      if (c && c.pid) localStorage.setItem(_K.cart, c.pid);
      return c;
    },
    get(pid) { return _http.get('/carts/' + pid); },
    addItem(pid, params) { return _http.post('/carts/' + pid + '/items', params); },
    updateItem(pid, itemId, qty) { return _http.put('/carts/' + pid + '/items/' + itemId, { quantity: qty }); },
    removeItem(pid, itemId) { return _http.del('/carts/' + pid + '/items/' + itemId); },
  },

  // ── Pedidos ─────────────────────────────────────────────────────────────
  orders: {
    create(params) { return _http.post('/orders', params); },
    get(pid)       { return _http.get('/orders/' + pid); },
    listForCustomer(customerPid, cursor) {
      const qs = cursor ? '?cursor=' + cursor : '';
      return _http.paginate('/customers/' + customerPid + '/orders' + qs);
    },
  },

  // ── Cliente ─────────────────────────────────────────────────────────────
  customer: {
    get(pid)           { return _http.get('/customers/' + pid); },
    update(pid, p)     { return _http.put('/customers/' + pid, p); },
    addresses(pid)     { return _http.get('/customers/' + pid + '/addresses'); },
    addAddress(pid, p) { return _http.post('/customers/' + pid + '/addresses', p); },
  },

  // ── Wishlist (localStorage) ─────────────────────────────────────────────
  wishlist: {
    _load() { try { return JSON.parse(localStorage.getItem(_K.wish) || '[]'); } catch { return []; } },
    _save(a) { localStorage.setItem(_K.wish, JSON.stringify(a)); },
    all()    { return this._load(); },
    has(pid) { return this._load().includes(pid); },
    toggle(pid) {
      const a = this._load();
      const i = a.indexOf(pid);
      i === -1 ? a.push(pid) : a.splice(i, 1);
      this._save(a);
      return i === -1; // true = adicionado, false = removido
    },
    clear() { localStorage.removeItem(_K.wish); },
  },
};

// ─── Alpine.js Stores ─────────────────────────────────────────────────────────
document.addEventListener('alpine:init', function() {

  // $store.toasts — notificações toast
  // Layout mínimo necessário (cole uma vez no base layout):
  //
  //   <div x-data class="fixed bottom-4 right-4 z-50 space-y-2 pointer-events-none">
  //     <template x-for="t in $store.toasts.items" :key="t.id">
  //       <div x-show="true" x-transition
  //            class="bg-white border shadow-lg rounded-xl px-4 py-3 flex items-center gap-3 max-w-sm pointer-events-auto"
  //            :class="{'border-green-400':t.type==='success','border-red-400':t.type==='error','border-yellow-400':t.type==='warning','border-blue-400':t.type==='info'}">
  //         <span x-text="t.message" class="flex-1 text-sm text-gray-800"></span>
  //         <button @click="$store.toasts.dismiss(t.id)" class="text-gray-400 hover:text-gray-600 text-xs">✕</button>
  //       </div>
  //     </template>
  //   </div>
  Alpine.store('toasts', {
    items: [],
    _add(message, type) {
      const id = Date.now() + Math.random();
      this.items.push({ id, message, type });
      const self = this;
      setTimeout(function() { self.dismiss(id); }, 5000);
    },
    success: function(m) { this._add(m, 'success'); },
    error:   function(m) { this._add(m, 'error'); },
    warning: function(m) { this._add(m, 'warning'); },
    info:    function(m) { this._add(m, 'info'); },
    dismiss: function(id) { this.items = this.items.filter(function(t) { return t.id !== id; }); },
  });

  // $store.customer — estado do cliente logado
  Alpine.store('customer', {
    ok:      StoreSDK.auth.isLoggedIn(),
    data:    null,
    loading: false,

    async boot() {
      if (!this.ok) return;
      this.loading = true;
      try { this.data = await StoreSDK.auth.me(); }
      catch(e) { this.ok = false; localStorage.removeItem(_K.ctoken); }
      finally { this.loading = false; }
    },

    async login(email, password) {
      const r = await StoreSDK.auth.login({ email, password });
      this.ok   = true;
      this.data = (r && r.customer) ? r.customer : null;
      return r;
    },

    async register(params) {
      const r = await StoreSDK.auth.register(params);
      this.ok   = true;
      this.data = (r && r.customer) ? r.customer : null;
      return r;
    },

    async logout() {
      await StoreSDK.auth.logout();
      this.ok   = false;
      this.data = null;
    },
  });

  // $store.cart — carrinho reativo global
  Alpine.store('cart', {
    data:    null,
    open:    false,
    loading: false,

    get count() { return ((this.data && this.data.items) ? this.data.items : []).reduce(function(s,i){ return s + i.quantity; }, 0); },
    get total() { return (this.data && this.data.total) ? this.data.total : 0; },
    get items() { return (this.data && this.data.items) ? this.data.items : []; },

    async boot() {
      this.loading = true;
      try { this.data = await StoreSDK.cart.getOrCreate(); }
      catch(e) { console.warn('cart.boot:', e); }
      finally { this.loading = false; }
    },

    async addItem(variantId, quantity) {
      quantity = quantity || 1;
      if (!this.data || !this.data.pid) await this.boot();
      this.loading = true;
      try {
        this.data = await StoreSDK.cart.addItem(this.data.pid, { variant_id: variantId, quantity: quantity });
        Alpine.store('toasts').success('Produto adicionado ao carrinho!');
        this.open = true;
      } catch(e) {
        Alpine.store('toasts').error((e && e.message) ? e.message : 'Não foi possível adicionar o produto');
      } finally { this.loading = false; }
    },

    async updateItem(itemId, quantity) {
      this.loading = true;
      try { this.data = await StoreSDK.cart.updateItem(this.data.pid, itemId, quantity); }
      catch(e) { Alpine.store('toasts').error((e && e.message) ? e.message : 'Erro ao atualizar item'); }
      finally { this.loading = false; }
    },

    async removeItem(itemId) {
      this.loading = true;
      try {
        this.data = await StoreSDK.cart.removeItem(this.data.pid, itemId);
        Alpine.store('toasts').info('Item removido do carrinho');
      } catch(e) { Alpine.store('toasts').error((e && e.message) ? e.message : 'Erro ao remover item'); }
      finally { this.loading = false; }
    },
  });

  // $store.wishlist — favoritos
  Alpine.store('wishlist', {
    pids: StoreSDK.wishlist.all(),
    get count() { return this.pids.length; },
    has(pid)    { return this.pids.includes(pid); },
    toggle(pid) {
      const added = StoreSDK.wishlist.toggle(pid);
      this.pids   = StoreSDK.wishlist.all();
      Alpine.store('toasts')[added ? 'success' : 'info'](
        added ? 'Adicionado aos favoritos!' : 'Removido dos favoritos'
      );
      return added;
    },
  });

  // Boot automático
  requestAnimationFrame(async function() {
    if (!_PID) return;
    await Alpine.store('cart').boot().catch(function(){});
    await Alpine.store('customer').boot().catch(function(){});
  });
});

// ─── Componentes (x-data factories) ──────────────────────────────────────────

/**
 * ProductList(defaults)
 * Lista de produtos com filtros, busca debounce e paginação cursor.
 *
 * @param {object} defaults - { status, category_id, featured, q, limit }
 *
 * Uso mínimo:
 *   <div x-data="ProductList({ limit: 12 })" x-init="init()">
 *     <input x-model="filters.q" @input.debounce.400ms="search($event.target.value)">
 *     <select x-model="filters.status" @change="filter('status',$event.target.value)">
 *       <option value="active">Ativos</option>
 *     </select>
 *     <template x-for="p in products" :key="p.pid">
 *       <a :href="'/produto/'+p.slug">
 *         <img :src="p.image_url||'/static/images/placeholder.png'" :alt="p.title">
 *         <p x-text="p.title"></p>
 *       </a>
 *     </template>
 *     <button @click="loadMore()" x-show="hasMore" :disabled="loading">Carregar mais</button>
 *   </div>
 */
window.ProductList = function(defaults) {
  defaults = defaults || {};
  return {
    products: [],
    loading:  false,
    hasMore:  false,
    cursor:   null,
    filters: {
      status:      defaults.status      !== undefined ? defaults.status      : 'active',
      category_id: defaults.category_id !== undefined ? defaults.category_id : '',
      featured:    defaults.featured    !== undefined ? defaults.featured    : '',
      q:           defaults.q           !== undefined ? defaults.q           : '',
      limit:       defaults.limit       !== undefined ? defaults.limit       : 20,
    },

    async init()       { await this._load(true); },
    async loadMore()   { if (this.hasMore && !this.loading) await this._load(false); },
    async search(q)    { this.filters.q = q; await this._load(true); },
    async filter(k, v) { this.filters[k] = v; await this._load(true); },
    async refresh()    { await this._load(true); },

    async _load(reset) {
      if (reset) { this.products = []; this.cursor = null; }
      this.loading = true;
      try {
        const p = Object.assign({}, this.filters);
        if (this.cursor) p.cursor = this.cursor;
        Object.keys(p).forEach(function(k){ if (p[k] === '' || p[k] == null) delete p[k]; });
        const result = await StoreSDK.products.list(p);
        const data   = result.data || [];
        const meta   = result.meta || {};
        this.products = reset ? data : this.products.concat(data);
        this.hasMore  = !!meta.has_more;
        this.cursor   = meta.cursor || null;
      } catch(e) {
        Alpine.store('toasts').error((e && e.message) ? e.message : 'Erro ao carregar produtos');
      } finally { this.loading = false; }
    },

    fmtMoney: fmtMoney,
    statusLabel: statusLabel,
    statusClass: statusClass,
  };
};

/**
 * ProductDetail(pid)
 * Detalhe de produto: galeria, seleção de variantes/opções, preço por quantidade, carrinho.
 *
 * Uso mínimo:
 *   <div x-data="ProductDetail('{{ product.pid }}')" x-init="init()">
 *     <img :src="mainImage" :alt="product?.title">
 *     <!-- Galeria -->
 *     <template x-for="(img,i) in images" :key="i">
 *       <img :src="img" @click="mainImage=img" :class="{'ring-2':mainImage===img}">
 *     </template>
 *     <h1 x-text="product?.title"></h1>
 *     <p x-text="selectedPrice"></p>
 *     <!-- Opções (Cor, Tamanho…) -->
 *     <template x-for="opt in options" :key="opt.name">
 *       <div>
 *         <p x-text="opt.name"></p>
 *         <template x-for="val in opt.values" :key="val">
 *           <button @click="selectOption(opt.name, val)"
 *                   :class="{'ring-2': selectedOptions[opt.name]===val}"
 *                   x-text="val">
 *           </button>
 *         </template>
 *       </div>
 *     </template>
 *     <!-- Quantidade -->
 *     <button @click="qty > 1 && qty--">-</button>
 *     <span x-text="qty"></span>
 *     <button @click="qty++">+</button>
 *     <!-- Ações -->
 *     <button @click="addToCart()" :disabled="!inStock || $store.cart.loading">
 *       <span x-show="inStock">Adicionar ao carrinho</span>
 *       <span x-show="!inStock">Sem estoque</span>
 *     </button>
 *     <button @click="toggleWishlist()">
 *       <span x-show="!inWishlist">♡ Favoritar</span>
 *       <span x-show="inWishlist">♥ Favoritado</span>
 *     </button>
 *   </div>
 */
window.ProductDetail = function(pid) {
  return {
    product:         null,
    variants:        [],
    images:          [],
    mainImage:       '',
    selectedVariant: null,
    selectedOptions: {},
    qty:             1,
    loading:         false,
    inWishlist:      StoreSDK.wishlist.has(pid),

    async init() {
      this.loading = true;
      try {
        this.product  = await StoreSDK.products.get(pid);
        this.variants = (this.product && this.product.variants) ? this.product.variants : [];
        this.images   = (this.product && this.product.images)
          ? this.product.images.map(function(i){ return i.url || i; })
          : [];
        this.mainImage = this.images[0] || '/static/images/placeholder.png';
        if (this.variants.length > 0) this._pick(this.variants[0]);
      } catch(e) {
        Alpine.store('toasts').error('Produto não encontrado');
      } finally { this.loading = false; }
    },

    _pick(v) { this.selectedVariant = v; },

    selectVariant(v) { this._pick(v); },

    selectOption(key, value) {
      this.selectedOptions[key] = value;
      const opts = this.selectedOptions;
      const match = this.variants.find(function(v) {
        const o = v.option_values || {};
        return Object.keys(opts).every(function(k){ return o[k] === opts[k]; });
      });
      if (match) this._pick(match);
    },

    get options() {
      const map = {};
      this.variants.forEach(function(v) {
        Object.entries(v.option_values || {}).forEach(function([k, val]) {
          if (!map[k]) map[k] = [];
          if (!map[k].includes(val)) map[k].push(val);
        });
      });
      return Object.entries(map).map(function([name, values]) { return { name, values }; });
    },

    get selectedPrice() {
      const prices = (this.selectedVariant && this.selectedVariant.prices) ? this.selectedVariant.prices : [];
      if (!prices.length) return 'Consultar';
      const qty = this.qty;
      const price = prices.slice().sort(function(a,b){ return b.min_quantity - a.min_quantity; })
        .find(function(p){ return qty >= p.min_quantity; }) || prices[0];
      return fmtMoney(price.amount, price.currency || 'BRL');
    },

    get inStock() {
      if (!this.selectedVariant) return false;
      return this.selectedVariant.inventory_quantity > 0 || !!this.selectedVariant.allow_backorder;
    },

    async addToCart() {
      if (!this.selectedVariant) { Alpine.store('toasts').warning('Selecione uma opção'); return; }
      if (!this.inStock)          { Alpine.store('toasts').warning('Produto sem estoque'); return; }
      await Alpine.store('cart').addItem(this.selectedVariant.id, this.qty);
    },

    toggleWishlist() {
      const added   = Alpine.store('wishlist').toggle(this.product.pid);
      this.inWishlist = added;
    },

    fmtMoney: fmtMoney,
  };
};

/**
 * CategoryList(opts)
 * Lista/árvore de categorias.
 *
 * Uso mínimo:
 *   <div x-data="CategoryList()" x-init="init()">
 *     <template x-for="c in categories" :key="c.pid">
 *       <a :href="'/categoria/'+c.slug" x-text="c.name"></a>
 *     </template>
 *   </div>
 */
window.CategoryList = function(opts) {
  opts = opts || {};
  return {
    categories: [],
    loading:    false,
    async init() {
      this.loading = true;
      try { this.categories = (await StoreSDK.categories.list(opts)) || []; }
      catch(e) { Alpine.store('toasts').error('Erro ao carregar categorias'); }
      finally { this.loading = false; }
    },
  };
};

/**
 * CollectionDetail(pid)
 * Coleção com seus produtos.
 *
 * Uso mínimo:
 *   <div x-data="CollectionDetail('{{ collection.pid }}')" x-init="init()">
 *     <h2 x-text="collection?.title"></h2>
 *     <template x-for="p in products" :key="p.pid">…</template>
 *   </div>
 */
window.CollectionDetail = function(pid) {
  return {
    collection: null,
    products:   [],
    loading:    false,
    async init() {
      this.loading = true;
      try {
        this.collection = await StoreSDK.collections.get(pid);
        this.products   = (this.collection && this.collection.products) ? this.collection.products : [];
      } catch(e) { Alpine.store('toasts').error('Coleção não encontrada'); }
      finally { this.loading = false; }
    },
    fmtMoney: fmtMoney,
  };
};

/**
 * CartPage()
 * Página dedicada do carrinho com edição de quantidades.
 *
 * Uso mínimo:
 *   <div x-data="CartPage()" x-init="init()">
 *     <template x-for="item in $store.cart.items" :key="item.pid">
 *       <div>
 *         <span x-text="item.quantity"></span>
 *         <button @click="dec(item)">-</button>
 *         <button @click="inc(item)">+</button>
 *         <button @click="$store.cart.removeItem(item.id)">✕</button>
 *         <span x-text="fmtMoney(item.total)"></span>
 *       </div>
 *     </template>
 *     <strong x-text="fmtMoney($store.cart.total)"></strong>
 *     <a href="/checkout">Finalizar compra</a>
 *   </div>
 */
window.CartPage = function() {
  return {
    async init()    { await Alpine.store('cart').boot(); },
    async inc(item) { await Alpine.store('cart').updateItem(item.id, item.quantity + 1); },
    async dec(item) {
      if (item.quantity > 1) await Alpine.store('cart').updateItem(item.id, item.quantity - 1);
      else await Alpine.store('cart').removeItem(item.id);
    },
    fmtMoney: fmtMoney,
  };
};

/**
 * CartDrawer()
 * Mini-carrinho em gaveta lateral.
 *
 * Uso mínimo:
 *   <div x-data="CartDrawer()" x-init="init()">
 *     <button @click="$store.cart.open=true">
 *       🛒 <span x-text="$store.cart.count"></span>
 *     </button>
 *     <div x-show="$store.cart.open" @click="$store.cart.open=false"
 *          class="fixed inset-0 bg-black/40 z-40"></div>
 *     <div x-show="$store.cart.open" x-transition
 *          class="fixed right-0 top-0 h-full w-96 bg-white z-50 shadow-xl flex flex-col">
 *       <div class="flex-1 overflow-y-auto p-6 space-y-4">
 *         <template x-for="item in $store.cart.items" :key="item.pid">
 *           <div class="flex gap-3">
 *             <div class="flex-1">
 *               <p x-text="item.title"></p>
 *               <div class="flex items-center gap-2 mt-1">
 *                 <button @click="dec(item)">-</button>
 *                 <span x-text="item.quantity"></span>
 *                 <button @click="inc(item)">+</button>
 *               </div>
 *             </div>
 *             <p x-text="fmtMoney(item.total)"></p>
 *           </div>
 *         </template>
 *         <p x-show="!$store.cart.count" class="text-center text-gray-400">Carrinho vazio</p>
 *       </div>
 *       <div class="p-6 border-t">
 *         <div class="flex justify-between font-bold mb-4">
 *           <span>Total</span><span x-text="fmtMoney($store.cart.total)"></span>
 *         </div>
 *         <a href="/checkout"
 *            class="block w-full text-center bg-black text-white py-3 rounded-lg font-semibold">
 *           Finalizar compra
 *         </a>
 *       </div>
 *     </div>
 *   </div>
 */
window.CartDrawer = function() {
  return {
    async init()    { await Alpine.store('cart').boot(); },
    async inc(item) { await Alpine.store('cart').updateItem(item.id, item.quantity + 1); },
    async dec(item) {
      if (item.quantity > 1) await Alpine.store('cart').updateItem(item.id, item.quantity - 1);
      else await Alpine.store('cart').removeItem(item.id);
    },
    fmtMoney: fmtMoney,
  };
};

/**
 * SearchBar(opts)
 * Busca em tempo real com dropdown de resultados.
 *
 * Uso mínimo:
 *   <div x-data="SearchBar()" @keydown.escape.window="close()">
 *     <input x-model="query" @input.debounce.300ms="search()"
 *            placeholder="Buscar produtos…">
 *     <div x-show="open" @click.outside="close()">
 *       <template x-for="p in results" :key="p.pid">
 *         <a :href="'/produto/'+p.slug" @click="close()" x-text="p.title"></a>
 *       </template>
 *     </div>
 *   </div>
 */
window.SearchBar = function(opts) {
  opts = opts || {};
  return {
    query:   '',
    results: [],
    loading: false,
    open:    false,
    limit:   opts.limit || 8,

    async search() {
      if (this.query.trim().length < 2) { this.results = []; this.open = false; return; }
      this.loading = true;
      try {
        const r = await StoreSDK.products.list({ q: this.query, limit: this.limit, status: 'active' });
        this.results = r.data || [];
        this.open    = this.results.length > 0;
      } catch(e) { this.results = []; }
      finally { this.loading = false; }
    },

    close() { this.open = false; this.query = ''; this.results = []; },
    fmtMoney: fmtMoney,
  };
};

/**
 * CustomerAuth(opts)
 * Login + cadastro de cliente em um único componente.
 *
 * Uso mínimo:
 *   <div x-data="CustomerAuth({ redirectOnLogin: '/minha-conta' })">
 *     <!-- LOGIN -->
 *     <div x-show="mode==='login'">
 *       <input x-model="form.email" type="email" placeholder="E-mail">
 *       <input x-model="form.password" type="password" placeholder="Senha">
 *       <p x-show="errors.auth" x-text="errors.auth" class="text-red-500 text-sm"></p>
 *       <button @click="login()" :disabled="loading">Entrar</button>
 *       <button @click="mode='register'">Criar conta</button>
 *     </div>
 *     <!-- CADASTRO -->
 *     <div x-show="mode==='register'">
 *       <input x-model="form.first_name" placeholder="Nome">
 *       <input x-model="form.last_name" placeholder="Sobrenome">
 *       <input x-model="form.email" type="email">
 *       <input x-model="form.password" type="password">
 *       <button @click="register()" :disabled="loading">Criar conta</button>
 *       <button @click="mode='login'">Já tenho conta</button>
 *     </div>
 *   </div>
 */
window.CustomerAuth = function(opts) {
  opts = opts || {};
  return {
    mode:    'login',
    loading: false,
    errors:  {},
    form: {
      email: '', password: '', first_name: '', last_name: '',
      phone: '', marketing_consent: false,
    },

    init() {
      if (StoreSDK.auth.isLoggedIn() && opts.redirectOnLogin) {
        window.location.href = opts.redirectOnLogin;
      }
    },

    async login() {
      this.errors = {}; this.loading = true;
      try {
        await Alpine.store('customer').login(this.form.email, this.form.password);
        Alpine.store('toasts').success('Bem-vindo de volta!');
        if (opts.redirectOnLogin) window.location.href = opts.redirectOnLogin;
      } catch(e) {
        this.errors.auth = (e && e.message) ? e.message : 'E-mail ou senha inválidos';
        Alpine.store('toasts').error(this.errors.auth);
      } finally { this.loading = false; }
    },

    async register() {
      this.errors = {}; this.loading = true;
      try {
        await Alpine.store('customer').register(this.form);
        Alpine.store('toasts').success('Conta criada! Bem-vindo(a)!');
        if (opts.redirectOnLogin) window.location.href = opts.redirectOnLogin;
      } catch(e) {
        this.errors.auth = (e && e.message) ? e.message : 'Erro ao criar conta';
        Alpine.store('toasts').error(this.errors.auth);
      } finally { this.loading = false; }
    },
  };
};

/**
 * CustomerAccount()
 * Painel do cliente: perfil, endereços, histórico de pedidos.
 *
 * Uso mínimo:
 *   <div x-data="CustomerAccount()" x-init="init()">
 *     <button @click="tab='profile'">Perfil</button>
 *     <button @click="tab='addresses'">Endereços</button>
 *     <button @click="tab='orders'">Pedidos</button>
 *     <div x-show="tab==='profile'">
 *       <p x-text="profile?.first_name + ' ' + profile?.last_name"></p>
 *       <p x-text="profile?.email"></p>
 *     </div>
 *     <div x-show="tab==='orders'">
 *       <template x-for="o in orders" :key="o.pid">
 *         <div x-text="o.order_number + ' — ' + fmtMoney(o.total)"></div>
 *       </template>
 *     </div>
 *     <button @click="logout()">Sair</button>
 *   </div>
 */
window.CustomerAccount = function() {
  return {
    tab:       'profile',
    profile:   null,
    addresses: [],
    orders:    [],
    loading:   false,

    async init() {
      if (!StoreSDK.auth.isLoggedIn()) { window.location.href = '/conta/login'; return; }
      this.loading = true;
      try {
        await Alpine.store('customer').boot();
        this.profile = Alpine.store('customer').data;
        if (this.profile && this.profile.pid) {
          const [addrs, ords] = await Promise.all([
            StoreSDK.customer.addresses(this.profile.pid).catch(function(){ return []; }),
            StoreSDK.orders.listForCustomer(this.profile.pid)
              .then(function(r){ return r.data || []; }).catch(function(){ return []; }),
          ]);
          this.addresses = addrs;
          this.orders    = ords;
        }
      } finally { this.loading = false; }
    },

    async logout() {
      await Alpine.store('customer').logout();
      window.location.href = '/conta/login';
    },

    async addAddress(params) {
      try {
        const addr = await StoreSDK.customer.addAddress(this.profile.pid, params);
        this.addresses.push(addr);
        Alpine.store('toasts').success('Endereço adicionado!');
      } catch(e) {
        Alpine.store('toasts').error((e && e.message) ? e.message : 'Erro ao adicionar endereço');
      }
    },

    fmtMoney: fmtMoney,
    fmtDate:  fmtDate,
    statusLabel: statusLabel,
    statusClass: statusClass,
  };
};

/**
 * CheckoutForm()
 * Checkout completo em 3 etapas: identificação → endereço → pagamento → confirmação.
 *
 * Uso mínimo:
 *   <div x-data="CheckoutForm()" x-init="init()">
 *     <!-- Step 1 -->
 *     <div x-show="step===1">
 *       <input x-model="form.email" type="email" placeholder="E-mail">
 *       <input x-model="form.first_name" placeholder="Nome">
 *       <input x-model="form.last_name" placeholder="Sobrenome">
 *       <button @click="nextStep()">Continuar</button>
 *     </div>
 *     <!-- Step 2 -->
 *     <div x-show="step===2">
 *       <input x-model="form.addr.postal_code" @blur="lookupCep()" placeholder="CEP">
 *       <input x-model="form.addr.address_line_1" placeholder="Rua e número">
 *       <input x-model="form.addr.city" placeholder="Cidade">
 *       <input x-model="form.addr.state" placeholder="UF" maxlength="2">
 *       <button @click="prevStep()">Voltar</button>
 *       <button @click="nextStep()">Continuar</button>
 *     </div>
 *     <!-- Step 3 -->
 *     <div x-show="step===3">
 *       <label><input type="radio" x-model="form.payment" value="pix"> PIX</label>
 *       <label><input type="radio" x-model="form.payment" value="credit_card"> Cartão</label>
 *       <div x-text="fmtMoney($store.cart.total)"></div>
 *       <p x-show="error" x-text="error" class="text-red-500"></p>
 *       <button @click="prevStep()">Voltar</button>
 *       <button @click="submit()" :disabled="loading">Finalizar pedido</button>
 *     </div>
 *     <!-- Step 4 (confirmação) -->
 *     <div x-show="step===4">
 *       <p x-text="'Pedido ' + order?.order_number + ' realizado!'"></p>
 *       <a href="/">Continuar comprando</a>
 *     </div>
 *   </div>
 */
window.CheckoutForm = function() {
  return {
    step:    1,
    loading: false,
    order:   null,
    error:   '',
    form: {
      email: '', first_name: '', last_name: '', phone: '',
      addr: {
        first_name: '', last_name: '',
        address_line_1: '', address_line_2: '',
        city: '', state: '', postal_code: '', country: 'BR', phone: '',
      },
      payment: 'pix',
      notes:   '',
    },

    async init() {
      await Alpine.store('cart').boot();
      if (Alpine.store('cart').count === 0) { window.location.href = '/carrinho'; return; }
      const cd = Alpine.store('customer').data;
      if (cd) {
        this.form.email      = cd.email      || '';
        this.form.first_name = cd.first_name || '';
        this.form.last_name  = cd.last_name  || '';
        this.form.phone      = cd.phone      || '';
      }
    },

    nextStep() { this.step = Math.min(this.step + 1, 3); },
    prevStep() { this.step = Math.max(this.step - 1, 1); },

    async lookupCep() {
      const d = await cepLookup(this.form.addr.postal_code);
      if (d) {
        this.form.addr.address_line_1 = d.logradouro || '';
        this.form.addr.city           = d.localidade || '';
        this.form.addr.state          = d.uf          || '';
      }
    },

    async submit() {
      this.error = ''; this.loading = true;
      try {
        let customerId  = Alpine.store('customer').data && Alpine.store('customer').data.id;
        let customerPid = Alpine.store('customer').data && Alpine.store('customer').data.pid;

        if (!customerPid) {
          const reg = await StoreSDK.auth.register({
            email:      this.form.email,
            first_name: this.form.first_name,
            last_name:  this.form.last_name,
            phone:      this.form.phone,
            password:   _guestPw(),
          }).catch(function(){ return null; });
          if (reg && reg.customer) {
            customerId  = reg.customer.id;
            customerPid = reg.customer.pid;
          }
        }

        let addrId = null;
        if (customerPid) {
          const fAddr = Object.assign({}, this.form.addr, {
            first_name:          this.form.addr.first_name || this.form.first_name,
            last_name:           this.form.addr.last_name  || this.form.last_name,
            is_default_shipping: true,
            is_default_billing:  true,
          });
          const addr = await StoreSDK.customer.addAddress(customerPid, fAddr).catch(function(){ return null; });
          if (addr) addrId = addr.id;
        }

        this.order = await StoreSDK.orders.create({
          customer_id:         customerId,
          shipping_address_id: addrId,
          billing_address_id:  addrId,
          payment_method:      this.form.payment,
          notes:               this.form.notes,
        });

        localStorage.removeItem(_K.cart);
        Alpine.store('cart').data = null;
        this.step = 4;
      } catch(e) {
        this.error = (e && e.message) ? e.message : 'Erro ao finalizar pedido. Tente novamente.';
        Alpine.store('toasts').error(this.error);
      } finally { this.loading = false; }
    },

    fmtMoney: fmtMoney,
  };
};

/**
 * WishlistPage()
 * Página de favoritos.
 *
 * Uso mínimo:
 *   <div x-data="WishlistPage()" x-init="init()">
 *     <div x-show="!loading && products.length===0">Nenhum favorito ainda.</div>
 *     <template x-for="p in products" :key="p.pid">
 *       <div>
 *         <a :href="'/produto/'+p.slug" x-text="p.title"></a>
 *         <button @click="remove(p.pid)">Remover</button>
 *       </div>
 *     </template>
 *   </div>
 */
window.WishlistPage = function() {
  return {
    products: [],
    loading:  false,
    async init() {
      const pids = StoreSDK.wishlist.all();
      if (!pids.length) return;
      this.loading = true;
      try {
        const results = await Promise.allSettled(pids.map(function(pid){ return StoreSDK.products.get(pid); }));
        this.products = results.filter(function(r){ return r.status === 'fulfilled'; }).map(function(r){ return r.value; });
      } finally { this.loading = false; }
    },
    remove(pid) {
      Alpine.store('wishlist').toggle(pid);
      this.products = this.products.filter(function(p){ return p.pid !== pid; });
    },
    fmtMoney: fmtMoney,
  };
};

/**
 * OrderDetail(pid)
 * Detalhe de um pedido (pós-compra ou histórico).
 *
 * Uso mínimo:
 *   <div x-data="OrderDetail('{{ order_pid }}')" x-init="init()">
 *     <p x-text="order?.order_number"></p>
 *     <span x-text="statusLabel(order?.status)"></span>
 *     <template x-for="item in (order?.items || [])" :key="item.pid">
 *       <p x-text="item.quantity + 'x ' + item.title + ' — ' + fmtMoney(item.total)"></p>
 *     </template>
 *     <strong x-text="fmtMoney(order?.total)"></strong>
 *   </div>
 */
window.OrderDetail = function(pid) {
  return {
    order:   null,
    loading: false,
    async init() {
      this.loading = true;
      try { this.order = await StoreSDK.orders.get(pid); }
      catch(e) { Alpine.store('toasts').error('Pedido não encontrado'); }
      finally { this.loading = false; }
    },
    statusLabel: statusLabel,
    statusClass: statusClass,
    fmtMoney:    fmtMoney,
    fmtDate:     fmtDate,
    fmtDateTime: fmtDateTime,
  };
};
