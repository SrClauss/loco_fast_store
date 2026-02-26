//! CLI: create-template
//!
//! Cria páginas HTML com Tailwind + Alpine.js + store-sdk já configurados.
//!
//! # Uso
//!
//!   cargo run --bin create-template -- --name busca-produtos --type store
//!   cargo run --bin create-template -- --name produto-detalhe --type store --component ProductDetail
//!   cargo run --bin create-template -- --name relatorio       --type admin
//!   cargo run --bin create-template -- --name envios-lista    --type painel
//!   cargo run --bin create-template -- -n home -t store -c ProductList -o assets/views/store
//!
//! # Tipos disponíveis
//!
//!   store  — páginas de loja (Alpine + store-sdk.js)
//!   admin  — painel administrativo (Alpine + app.js admin)
//!   painel — painel de colaboradores/envios
//!
//! # Componentes Alpine pré-preenchidos (--type store)
//!
//!   ProductList, ProductDetail, CategoryList, CollectionDetail,
//!   CartPage, CartDrawer, SearchBar, CustomerAuth, CustomerAccount,
//!   CheckoutForm, WishlistPage, OrderDetail

use std::{fs, path::PathBuf, process};

// ── Argumentos ────────────────────────────────────────────────────────────────

struct Args {
    name: String,
    kind: TemplateKind,
    component: Option<String>,
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TemplateKind {
    Store,
    Admin,
    Painel,
}

impl TemplateKind {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "store" | "loja" => Some(Self::Store),
            "admin" => Some(Self::Admin),
            "painel" | "panel" => Some(Self::Painel),
            _ => None,
        }
    }

    fn default_output(&self) -> &'static str {
        match self {
            Self::Store => "assets/views/store",
            Self::Admin => "assets/views/admin",
            Self::Painel => "assets/views/painel",
        }
    }
}

// ── Ponto de entrada ──────────────────────────────────────────────────────────

fn main() {
    let args = parse_args();
    let dir = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(args.kind.default_output()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| {
        die(&format!(
            "Não foi possível criar diretório {:?}: {}",
            dir, e
        ))
    });

    let filename = slug(&args.name) + ".html";
    let dest = dir.join(&filename);

    if dest.exists() {
        eprintln!("⚠️  Arquivo já existe: {}", dest.display());
        eprintln!("   Escolha um nome diferente ou remova o arquivo existente.");
        process::exit(1);
    }

    let html = match args.kind {
        TemplateKind::Store => gen_store(&args),
        TemplateKind::Admin => gen_admin(&args),
        TemplateKind::Painel => gen_painel(&args),
    };

    fs::write(&dest, html)
        .unwrap_or_else(|e| die(&format!("Erro ao escrever {}: {}", dest.display(), e)));

    println!("✅ Template criado: {}", dest.display());
    println!();
    match args.kind {
        TemplateKind::Store => {
            println!("  Estende : store/layouts/base.html  (ou crie seu próprio layout)");
            if let Some(ref c) = args.component {
                println!("  Componente Alpine pré-carregado: {}()", c);
                println!();
                println!("  Docs do componente:");
                print_component_docs(c);
            }
        }
        TemplateKind::Admin => {
            println!("  Estende : admin/layouts/base.html");
        }
        TemplateKind::Painel => {
            println!("  Estende : painel/layouts/base.html");
        }
    }
    println!();
    println!("  Próximos passos:");
    println!("  1. Edite {}", dest.display());
    println!("  2. Implemente o bloco content");
    println!("  3. Adicione a rota em src/controllers/");
}

// ── Geradores de HTML ─────────────────────────────────────────────────────────

fn gen_store(args: &Args) -> String {
    let title = title_case(&args.name);
    let slug = slug(&args.name);
    let comp = args.component.as_deref().unwrap_or("ProductList");
    let comp_i = component_init(comp);
    let comp_hint = component_placeholder(comp);

    format!(
        r#"{{% extends "store/layouts/base.html" %}}
{{% block title %}}{title}{{% endblock %}}

{{#-
  Página: {slug}.html  (tipo: store)
  Gerada por: create-template
  Componente Alpine: {comp}

  Documentação do componente — consulte store-sdk.js ou docs/ALPINE_SDK.md

  Exemplo de include no roteador (src/controllers/):
    format::render().view(&v, "store/{slug}.html", serde_json::json!({{ "store": {{ "pid": store.pid.to_string() }} }}))
-#}}

{{% block content %}}
<div x-data="{comp_i}" x-init="init()">

  {{#- ═══════════════════════════════════════════════════════════════════
       Implemente o conteúdo da página aqui usando os dados do componente.
       Todos os métodos e propriedades do componente estão disponíveis.

       Stores globais:
         $store.cart      — carrinho (count, total, items, addItem, open)
         $store.customer  — cliente logado (ok, data, login, logout)
         $store.toasts    — notificações (success, error, warning, info)
         $store.wishlist  — favoritos (has, toggle, count)

       Utilitários:
         fmtMoney(cents)    — "R$ 19,90"
         fmtDate(iso)       — "25 jan. 2026"
         fmtDateTime(iso)   — "25/01/2026 14:30"
         statusLabel(s)     — "Pendente", "Enviado"…
         statusClass(s)     — classes Tailwind de cor para badges
  ═══════════════════════════════════════════════════════════════════ -#}}

{comp_hint}

</div>
{{% endblock %}}

{{% block scripts %}}
<script>
// Lógica específica desta página (se necessário).
// Prefira usar os componentes de store-sdk.js em vez de duplicar lógica aqui.
</script>
{{% endblock %}}
"#,
        title = title,
        slug = slug,
        comp = comp,
        comp_i = comp_i,
        comp_hint = comp_hint,
    )
}

fn gen_admin(args: &Args) -> String {
    let title = title_case(&args.name);
    let slug = slug(&args.name);

    format!(
        r#"{{% extends "admin/layouts/base.html" %}}
{{% block title %}}{title}{{% endblock %}}
{{% block page_title %}}{title}{{% endblock %}}

{{#-
  Página: {slug}.html  (tipo: admin)
  Gerada por: create-template

  Variáveis disponíveis no contexto (injetar no controller):
    current_page — identifica o item ativo no sidebar

  API helper disponível: window.api (ver assets/static/js/app.js)
  Utilitários: window.formatCurrency, window.formatDate, window.formatDateTime
-#}}

{{% block content %}}
<div x-data="{slug}Data()" x-init="init()">

  {{#- Cabeçalho da página -#}}
  <div class="flex items-center justify-between mb-6">
    <div>
      <h2 class="text-2xl font-bold text-gray-900">{title}</h2>
      <p class="text-sm text-gray-500 mt-1">Descrição da página</p>
    </div>
    <div class="flex items-center gap-3">
      {{#- Botões de ação -#}}
    </div>
  </div>

  {{#- Conteúdo principal -#}}
  <div class="card">
    <div class="card-body">
      <p class="text-gray-500">Implemente o conteúdo aqui.</p>
    </div>
  </div>

</div>
{{% endblock %}}

{{% block scripts %}}
<script>
function {slug}Data() {{
  return {{
    loading: false,

    async init() {{
      // Inicialização da página
    }},
  }};
}}
</script>
{{% endblock %}}
"#,
        title = title,
        slug = slug,
    )
}

fn gen_painel(args: &Args) -> String {
    let title = title_case(&args.name);
    let slug = slug(&args.name);

    format!(
        r#"{{% extends "painel/layouts/base.html" %}}
{{% block title %}}{title}{{% endblock %}}
{{% block page_title %}}{title}{{% endblock %}}
{{% set current_page = "{slug}" %}}

{{#-
  Página: {slug}.html  (tipo: painel de colaboradores)
  Gerada por: create-template

  Helper de API disponível: window.papi (ver painel/layouts/base.html)
    papi.get('/api/painel/{{ store.pid }}/pedidos')
    papi.put('/api/painel/{{ store.pid }}/pedidos/PID/status', {{ status:'shipped' }})

  Utilitários: fmt(cents), fmtDate(iso), statusLabel(s), statusClass(s)
-#}}

{{% block content %}}
<div x-data="{slug}Data()" x-init="init()">

  {{#- Loading -#}}
  <div x-show="loading" class="flex items-center justify-center py-16 text-gray-400">
    <i class="fas fa-spinner fa-spin text-2xl mr-3"></i>
    <span>Carregando…</span>
  </div>

  {{#- Conteúdo -#}}
  <div x-show="!loading">
    <p class="text-gray-500">Implemente o conteúdo aqui.</p>
  </div>

</div>
{{% endblock %}}

{{% block scripts %}}
<script>
function {slug}Data() {{
  return {{
    loading: false,

    async init() {{
      this.loading = true;
      try {{
        // const data = await papi.get('/api/painel/{{ store.pid }}/pedidos');
      }} catch(e) {{
        console.error(e);
      }} finally {{
        this.loading = false;
      }}
    }},
  }};
}}
</script>
{{% endblock %}}
"#,
        title = title,
        slug = slug,
    )
}

// ── Helpers de conteúdo ───────────────────────────────────────────────────────

/// Retorna a chamada `x-data` certa para o componente.
fn component_init(comp: &str) -> String {
    match comp {
        "ProductDetail" => "ProductDetail('{{ product.pid }}')".into(),
        "CollectionDetail" => "CollectionDetail('{{ collection.pid }}')".into(),
        "OrderDetail" => "OrderDetail('{{ order_pid }}')".into(),
        "CustomerAuth" => "CustomerAuth({ redirectOnLogin: '/minha-conta' })".into(),
        "SearchBar" => "SearchBar({ limit: 8 })".into(),
        "CartPage" => "CartPage()".into(),
        "CartDrawer" => "CartDrawer()".into(),
        "CustomerAccount" => "CustomerAccount()".into(),
        "CheckoutForm" => "CheckoutForm()".into(),
        "WishlistPage" => "WishlistPage()".into(),
        "CategoryList" => "CategoryList()".into(),
        _ => "ProductList({ limit: 12 })".to_string(),
    }
}

/// Retorna um placeholder HTML comentado com o uso básico do componente.
fn component_placeholder(comp: &str) -> String {
    match comp {
        "ProductList" => r#"
  {{#- Filtros -#}}
  <div class="flex gap-3 mb-6">
    <input x-model="filters.q"
           @input.debounce.400ms="search($event.target.value)"
           placeholder="Buscar produtos…"
           class="border rounded-lg px-4 py-2 w-full max-w-sm">
    <select x-model="filters.category_id" @change="filter('category_id',$event.target.value)"
            class="border rounded-lg px-3 py-2">
      <option value="">Todas categorias</option>
    </select>
  </div>

  {{#- Loading skeleton -#}}
  <div x-show="loading && products.length === 0"
       class="grid grid-cols-2 md:grid-cols-4 gap-6">
    <template x-for="i in 8" :key="i">
      <div class="bg-gray-100 animate-pulse rounded-xl h-64"></div>
    </template>
  </div>

  {{#- Grade de produtos -#}}
  <div class="grid grid-cols-2 md:grid-cols-4 gap-6">
    <template x-for="p in products" :key="p.pid">
      <a :href="'/produto/'+p.slug" class="group block">
        <div class="aspect-square bg-gray-100 rounded-xl overflow-hidden mb-3">
          <img :src="p.image_url || '/static/images/placeholder.png'"
               :alt="p.title"
               class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300">
        </div>
        <p class="font-medium text-gray-900 text-sm truncate" x-text="p.title"></p>
        <p class="text-gray-500 text-xs mb-1" x-text="p.category_name"></p>
        <p class="font-bold text-gray-900" x-text="fmtMoney(p.price)"></p>
      </a>
    </template>
  </div>

  {{#- Sem resultados -#}}
  <div x-show="!loading && products.length === 0" class="py-16 text-center text-gray-400">
    <i class="fas fa-search text-4xl mb-4"></i>
    <p class="font-medium">Nenhum produto encontrado</p>
    <p class="text-sm mt-1">Tente outros termos ou remova os filtros.</p>
  </div>

  {{#- Carregar mais -#}}
  <div class="text-center mt-8">
    <button @click="loadMore()"
            x-show="hasMore"
            :disabled="loading"
            class="px-8 py-3 border border-gray-300 rounded-full text-sm font-medium hover:bg-gray-50 disabled:opacity-50">
      <span x-show="!loading">Carregar mais</span>
      <span x-show="loading"><i class="fas fa-spinner fa-spin mr-2"></i>Carregando…</span>
    </button>
  </div>"#.into(),

        "ProductDetail" => r#"
  <div class="grid grid-cols-1 md:grid-cols-2 gap-12">
    {{#- Galeria -#}}
    <div>
      <div class="aspect-square bg-gray-100 rounded-2xl overflow-hidden mb-4">
        <img :src="mainImage" :alt="product?.title"
             class="w-full h-full object-cover">
      </div>
      <div class="flex gap-3 overflow-x-auto">
        <template x-for="(img, i) in images" :key="i">
          <img :src="img"
               @click="mainImage = img"
               :class="mainImage === img ? 'ring-2 ring-black' : 'ring-1 ring-gray-200'"
               class="w-20 h-20 object-cover rounded-lg cursor-pointer flex-shrink-0">
        </template>
      </div>
    </div>

    {{#- Informações -#}}
    <div class="space-y-6">
      <div>
        <h1 class="text-3xl font-bold text-gray-900" x-text="product?.title"></h1>
        <p class="text-gray-500 mt-1" x-text="product?.category_name"></p>
      </div>

      <p class="text-3xl font-bold" x-text="selectedPrice"></p>

      {{#- Opções (Cor, Tamanho…) -#}}
      <template x-for="opt in options" :key="opt.name">
        <div>
          <p class="text-sm font-semibold text-gray-700 mb-2" x-text="opt.name"></p>
          <div class="flex flex-wrap gap-2">
            <template x-for="val in opt.values" :key="val">
              <button @click="selectOption(opt.name, val)"
                      :class="selectedOptions[opt.name] === val
                        ? 'bg-black text-white border-black'
                        : 'bg-white text-gray-700 border-gray-300 hover:border-gray-500'"
                      class="px-4 py-2 border rounded-lg text-sm font-medium transition-colors"
                      x-text="val">
              </button>
            </template>
          </div>
        </div>
      </template>

      {{#- Quantidade -#}}
      <div>
        <p class="text-sm font-semibold text-gray-700 mb-2">Quantidade</p>
        <div class="flex items-center gap-3 border border-gray-300 rounded-xl w-fit px-4 py-2">
          <button @click="qty > 1 && qty--" class="text-xl font-bold w-6 text-center">-</button>
          <span x-text="qty" class="text-lg font-semibold w-8 text-center"></span>
          <button @click="qty++" class="text-xl font-bold w-6 text-center">+</button>
        </div>
      </div>

      {{#- Ações -#}}
      <div class="flex gap-3">
        <button @click="addToCart()"
                :disabled="!inStock || $store.cart.loading"
                class="flex-1 py-4 bg-black text-white rounded-xl font-semibold text-lg disabled:opacity-50 hover:bg-gray-800 transition-colors">
          <span x-show="inStock">Adicionar ao carrinho</span>
          <span x-show="!inStock">Sem estoque</span>
        </button>
        <button @click="toggleWishlist()"
                :class="inWishlist ? 'text-red-500 border-red-300' : 'text-gray-500 border-gray-300'"
                class="px-4 py-4 border rounded-xl hover:bg-gray-50 transition-colors text-xl">
          <span x-show="inWishlist">♥</span>
          <span x-show="!inWishlist">♡</span>
        </button>
      </div>

      {{#- Descrição -#}}
      <div class="prose prose-sm max-w-none text-gray-600"
           x-html="product?.description || ''">
      </div>
    </div>
  </div>"#.into(),

        "SearchBar" => r#"
  <div class="relative" @keydown.escape="close()">
    <div class="relative">
      <i class="fas fa-search absolute left-4 top-1/2 -translate-y-1/2 text-gray-400"></i>
      <input x-model="query"
             @input.debounce.300ms="search()"
             @focus="query.length >= 2 && search()"
             placeholder="Buscar produtos…"
             class="w-full pl-12 pr-4 py-4 border border-gray-300 rounded-2xl text-lg focus:outline-none focus:ring-2 focus:ring-black">
      <button x-show="loading" class="absolute right-4 top-1/2 -translate-y-1/2">
        <i class="fas fa-spinner fa-spin text-gray-400"></i>
      </button>
    </div>

    <div x-show="open" @click.outside="close()"
         class="absolute top-full left-0 right-0 mt-2 bg-white border border-gray-200 rounded-2xl shadow-xl z-50 overflow-hidden">
      <template x-for="p in results" :key="p.pid">
        <a :href="'/produto/'+p.slug"
           @click="close()"
           class="flex items-center gap-4 px-5 py-4 hover:bg-gray-50 transition-colors">
          <img :src="p.image_url || '/static/images/placeholder.png'"
               :alt="p.title"
               class="w-12 h-12 object-cover rounded-lg flex-shrink-0">
          <div class="flex-1 min-w-0">
            <p class="font-medium text-gray-900 truncate" x-text="p.title"></p>
            <p class="text-sm text-gray-500" x-text="fmtMoney(p.price)"></p>
          </div>
          <i class="fas fa-chevron-right text-gray-400 text-sm"></i>
        </a>
      </template>
    </div>
  </div>"#.into(),

        "CartPage" => r#"
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
    {{#- Itens -#}}
    <div class="lg:col-span-2 space-y-4">
      <div x-show="$store.cart.count === 0" class="py-16 text-center text-gray-400">
        <i class="fas fa-shopping-cart text-5xl mb-4"></i>
        <p class="font-medium">Carrinho vazio</p>
        <a href="/" class="mt-4 inline-block text-black underline">Continuar comprando</a>
      </div>
      <template x-for="item in $store.cart.items" :key="item.pid">
        <div class="flex gap-4 p-4 bg-white rounded-xl border border-gray-200">
          <img :src="item.image_url || '/static/images/placeholder.png'"
               :alt="item.title"
               class="w-24 h-24 object-cover rounded-lg flex-shrink-0">
          <div class="flex-1">
            <p class="font-semibold text-gray-900" x-text="item.title"></p>
            <p class="text-sm text-gray-500" x-text="item.variant_title"></p>
            <div class="flex items-center justify-between mt-3">
              <div class="flex items-center gap-2 border border-gray-300 rounded-lg px-3 py-1">
                <button @click="dec(item)" class="font-bold text-lg w-5 text-center">-</button>
                <span x-text="item.quantity" class="w-6 text-center"></span>
                <button @click="inc(item)" class="font-bold text-lg w-5 text-center">+</button>
              </div>
              <p class="font-bold text-lg" x-text="fmtMoney(item.total)"></p>
            </div>
          </div>
          <button @click="$store.cart.removeItem(item.id)"
                  class="text-gray-400 hover:text-red-500 transition-colors self-start">
            <i class="fas fa-times"></i>
          </button>
        </div>
      </template>
    </div>

    {{#- Resumo -#}}
    <div class="bg-white rounded-xl border border-gray-200 p-6 h-fit space-y-4">
      <h3 class="font-bold text-lg text-gray-900">Resumo do pedido</h3>
      <div class="flex justify-between text-sm text-gray-600">
        <span>Subtotal (<span x-text="$store.cart.count"></span> itens)</span>
        <span x-text="fmtMoney($store.cart.total)"></span>
      </div>
      <div class="border-t pt-4 flex justify-between font-bold text-lg">
        <span>Total</span>
        <span x-text="fmtMoney($store.cart.total)"></span>
      </div>
      <a href="/checkout"
         class="block w-full text-center bg-black text-white py-4 rounded-xl font-bold text-lg hover:bg-gray-800 transition-colors">
        Finalizar compra
      </a>
      <a href="/" class="block text-center text-sm text-gray-500 hover:text-gray-700">
        Continuar comprando
      </a>
    </div>
  </div>"#.into(),

        _ => "  <!-- Implemente o conteúdo aqui usando os dados do componente Alpine -->".into(),
    }
}

fn print_component_docs(comp: &str) {
    let docs: &[(&str, &str)] = &[
        ("ProductList",      "products[] hasMore loading cursor filters{q status category_id limit}\n          init() loadMore() search(q) filter(k,v) refresh() fmtMoney(c)"),
        ("ProductDetail",    "product variants images mainImage selectedVariant selectedOptions qty inStock inWishlist selectedPrice\n          init() selectVariant(v) selectOption(k,v) addToCart() toggleWishlist() options fmtMoney(c)"),
        ("CategoryList",     "categories[] loading\n          init()"),
        ("CollectionDetail", "collection products[] loading\n          init() fmtMoney(c)"),
        ("CartPage",         "init() inc(item) dec(item) fmtMoney(c)\n          + $store.cart.{items,count,total,removeItem,updateItem}"),
        ("CartDrawer",       "init() inc(item) dec(item) fmtMoney(c)\n          + $store.cart.{open,items,count,total}"),
        ("SearchBar",        "query results[] loading open limit\n          search() close() fmtMoney(c)"),
        ("CustomerAuth",     "mode('login'|'register') loading errors form{email password first_name last_name phone}\n          init() login() register()"),
        ("CustomerAccount",  "tab profile addresses[] orders[] loading\n          init() logout() addAddress(p) fmtMoney(c) fmtDate(s) statusLabel(s)"),
        ("CheckoutForm",     "step(1-4) loading order error form{email first_name last_name addr payment notes}\n          init() nextStep() prevStep() lookupCep() submit() fmtMoney(c)"),
        ("WishlistPage",     "products[] loading\n          init() remove(pid) fmtMoney(c)"),
        ("OrderDetail",      "order loading\n          init() statusLabel(s) statusClass(s) fmtMoney(c) fmtDate(s)"),
    ];

    for (name, doc) in docs {
        if *name == comp {
            for line in doc.lines() {
                println!("      {}", line.trim());
            }
            return;
        }
    }
}

// ── Parser de args ─────────────────────────────────────────────────────────────

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().collect();

    if raw.len() < 2 || raw.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        process::exit(0);
    }

    let mut name = None::<String>;
    let mut kind = None::<TemplateKind>;
    let mut component = None::<String>;
    let mut output = None::<PathBuf>;

    let mut i = 1usize;
    while i < raw.len() {
        match raw[i].as_str() {
            "--name" | "-n" => {
                i += 1;
                name = raw.get(i).cloned();
            }
            "--type" | "-t" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    kind = TemplateKind::from_str(v).or_else(|| {
                        eprintln!("Tipo inválido: {v}");
                        process::exit(1);
                    });
                }
            }
            "--component" | "-c" => {
                i += 1;
                component = raw.get(i).cloned();
            }
            "--output" | "-o" => {
                i += 1;
                output = raw.get(i).map(PathBuf::from);
            }
            other => {
                eprintln!("Argumento desconhecido: {other}");
                print_help();
                process::exit(1);
            }
        }
        i += 1;
    }

    let name = name.unwrap_or_else(|| {
        eprintln!("--name é obrigatório");
        process::exit(1);
    });
    let kind = kind.unwrap_or_else(|| {
        eprintln!("--type é obrigatório (store|admin|painel)");
        process::exit(1);
    });

    Args {
        name,
        kind,
        component,
        output,
    }
}

fn print_help() {
    eprintln!(
        r#"
create-template — Gerador de páginas HTML para Loco Fast Store

USO:
  cargo run --bin create-template -- --name <nome> --type <tipo> [opções]

ARGUMENTOS OBRIGATÓRIOS:
  -n, --name <nome>       Nome da página (ex.: busca-produtos, checkout, home)
  -t, --type <tipo>       Tipo do template:
                            store  — página de loja (Alpine + store-sdk.js)
                            admin  — painel administrativo
                            painel — painel de colaboradores/envios

ARGUMENTOS OPCIONAIS:
  -c, --component <nome>  Componente Alpine pré-preenchido (apenas --type store):
                            ProductList | ProductDetail | CategoryList
                            CollectionDetail | CartPage | CartDrawer | SearchBar
                            CustomerAuth | CustomerAccount | CheckoutForm
                            WishlistPage | OrderDetail
  -o, --output <dir>      Diretório de saída (padrão: assets/views/<tipo>/)
  -h, --help              Exibe esta ajuda

EXEMPLOS:
  cargo run --bin create-template -- -n busca          -t store -c ProductList
  cargo run --bin create-template -- -n produto        -t store -c ProductDetail
  cargo run --bin create-template -- -n carrinho       -t store -c CartPage
  cargo run --bin create-template -- -n checkout       -t store -c CheckoutForm
  cargo run --bin create-template -- -n minha-conta    -t store -c CustomerAccount
  cargo run --bin create-template -- -n login          -t store -c CustomerAuth
  cargo run --bin create-template -- -n favoritos      -t store -c WishlistPage
  cargo run --bin create-template -- -n relatorio      -t admin
  cargo run --bin create-template -- -n envios-lista   -t painel
  cargo run --bin create-template -- -n minha-pagina   -t store -o assets/views/minha-loja
"#
    );
}

// ── Utilidades de string ──────────────────────────────────────────────────────

/// Converte "busca-produtos" → "Busca Produtos"
fn title_case(s: &str) -> String {
    s.split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Converte "Busca Produtos" → "busca-produtos"
fn slug(s: &str) -> String {
    s.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn die(msg: &str) -> ! {
    eprintln!("Erro: {msg}");
    process::exit(1);
}
