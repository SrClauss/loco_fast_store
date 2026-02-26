# Loco Fast Store :train:

Plataforma de e-commerce SaaS construída com [Loco](https://loco.rs) — framework web de alta performance em Rust — e painel administrativo em HTML/Alpine.js com Tailwind CSS.

---

## Início Rápido

```sh
cargo loco start
```

Saída esperada:

```
Finished dev [unoptimized + debuginfo] target(s) in 21.63s
    Running `target/debug/loco_fast_store-cli start`

environment: development
   database: automigrate
     logger: debug
compilation: debug
      modes: server

listening on http://localhost:5150
```

---

## Estrutura do Projeto

```
src/
├── controllers/     # Rotas e handlers HTTP
├── models/          # Entidades e lógica de negócio
├── dto/             # Objetos de transferência de dados
├── workers/         # Tarefas em background
├── tasks/           # Tarefas CLI
└── views/           # Templates Tera (SSR)

assets/
├── views/admin/     # Painel administrativo (HTML + Alpine.js)
└── static/          # CSS, JS e imagens estáticas

migration/           # Migrações do banco de dados (SeaORM)
```

---

## Descrição do Banco de Dados

O banco de dados é gerenciado pelo [SeaORM](https://www.sea-ql.org/SeaORM/) e suporta **PostgreSQL** e **SQLite**. Abaixo está a descrição completa de todas as tabelas, seus campos, tipos, relacionamentos e índices.

---

### Diagrama de Relacionamentos (resumido)

```
users ──< stores ──< categories
                 └──< products ──< product_variants ──< prices
                 │              └──< product_images
                 ├──< collections >──< collection_products >──< products
                 ├──< customers ──< addresses
                 ├──< carts ──< cart_items
                 └──< orders ──< order_items
```

---

### Tabela: `users`

Usuários administrativos do sistema. Gerada pelo Loco como parte do starter SaaS.

| Coluna                        | Tipo                        | Nulo | Padrão | Descrição                                    |
|-------------------------------|-----------------------------|------|--------|----------------------------------------------|
| `id`                          | INTEGER (PK, auto)          | Não  | —      | Identificador interno                        |
| `pid`                         | UUID (único)                | Não  | —      | Identificador público (UUID v4)              |
| `email`                       | VARCHAR (único)             | Não  | —      | E-mail de login (único)                      |
| `password`                    | VARCHAR                     | Não  | —      | Hash da senha                                |
| `api_key`                     | VARCHAR (único)             | Não  | —      | Chave de API (único)                         |
| `name`                        | VARCHAR                     | Não  | —      | Nome completo do usuário                     |
| `reset_token`                 | VARCHAR                     | Sim  | NULL   | Token para redefinição de senha              |
| `reset_sent_at`               | TIMESTAMPTZ                 | Sim  | NULL   | Data/hora do envio do token de reset         |
| `email_verification_token`    | VARCHAR                     | Sim  | NULL   | Token de verificação de e-mail               |
| `email_verification_sent_at`  | TIMESTAMPTZ                 | Sim  | NULL   | Data/hora do envio da verificação            |
| `email_verified_at`           | TIMESTAMPTZ                 | Sim  | NULL   | Data/hora da confirmação do e-mail           |
| `magic_link_token`            | VARCHAR                     | Sim  | NULL   | Token de acesso via magic link               |
| `magic_link_expiration`       | TIMESTAMPTZ                 | Sim  | NULL   | Expiração do magic link                      |
| `created_at`                  | TIMESTAMPTZ                 | Não  | now()  | Data de criação                              |
| `updated_at`                  | TIMESTAMPTZ                 | Não  | now()  | Data da última atualização                   |

**Relacionamentos:**
- Um `user` pode ser dono de muitas `stores`.
- Um `user` pode estar associado a um `customer` (acesso loja).

---

### Tabela: `stores`

Representa cada loja dentro da plataforma SaaS multi-tenant.

| Coluna             | Tipo                   | Nulo | Padrão    | Descrição                                        |
|--------------------|------------------------|------|-----------|--------------------------------------------------|
| `id`               | INTEGER (PK, auto)     | Não  | —         | Identificador interno                            |
| `pid`              | UUID (único)           | Não  | —         | Identificador público (UUID v4)                  |
| `slug`             | VARCHAR(128) (único)   | Não  | —         | Slug único da loja (URL-friendly)                |
| `name`             | VARCHAR(256)           | Não  | —         | Nome da loja                                     |
| `domain`           | VARCHAR(256)           | Sim  | NULL      | Domínio customizado da loja                      |
| `default_currency` | CHAR(3)                | Não  | `"BRL"`   | Moeda padrão (código ISO 4217)                   |
| `config`           | JSONB                  | Não  | `{}`      | Configurações gerais da loja (JSON)              |
| `status`           | VARCHAR(20)            | Não  | `"draft"` | Status: `draft`, `active`, `suspended`           |
| `metadata`         | JSONB                  | Não  | `{}`      | Metadados adicionais (JSON)                      |
| `owner_id`         | INTEGER (FK → users)   | Não  | —         | Dono da loja                                     |
| `created_at`       | TIMESTAMPTZ            | Não  | now()     | Data de criação                                  |
| `updated_at`       | TIMESTAMPTZ            | Não  | now()     | Data da última atualização                       |
| `deleted_at`       | TIMESTAMPTZ            | Sim  | NULL      | Soft delete (NULL = ativo)                       |

**Índices:**
- `idx_stores_slug` — em `slug`

**Relacionamentos:**
- `owner_id` → `users.id` (CASCADE DELETE)
- Uma `store` possui muitas `categories`, `products`, `collections`, `customers`, `carts` e `orders`.

---

### Tabela: `categories`

Categorias hierárquicas para organização dos produtos.

| Coluna        | Tipo                    | Nulo | Padrão | Descrição                                         |
|---------------|-------------------------|------|--------|---------------------------------------------------|
| `id`          | INTEGER (PK, auto)      | Não  | —      | Identificador interno                             |
| `pid`         | UUID (único)            | Não  | —      | Identificador público (UUID v4)                   |
| `store_id`    | INTEGER (FK → stores)   | Não  | —      | Loja à qual pertence                              |
| `name`        | VARCHAR(256)            | Não  | —      | Nome da categoria                                 |
| `slug`        | VARCHAR(256)            | Não  | —      | Slug da categoria                                 |
| `description` | TEXT                    | Sim  | NULL   | Descrição da categoria                            |
| `parent_id`   | INTEGER (FK → self)     | Sim  | NULL   | Categoria pai (para hierarquia)                   |
| `image_url`   | VARCHAR(512)            | Sim  | NULL   | URL da imagem de capa da categoria                |
| `sort_order`  | INTEGER                 | Não  | `0`    | Ordem de exibição                                 |
| `created_at`  | TIMESTAMPTZ             | Não  | now()  | Data de criação                                   |
| `updated_at`  | TIMESTAMPTZ             | Não  | now()  | Data da última atualização                        |
| `deleted_at`  | TIMESTAMPTZ             | Sim  | NULL   | Soft delete (NULL = ativa)                        |

**Índices:**
- `idx_categories_store_slug` — em `(store_id, slug)`, único

**Relacionamentos:**
- `store_id` → `stores.id` (CASCADE DELETE)
- `parent_id` → `categories.id` (SET NULL) — hierarquia de categorias
- Uma `category` pode ter muitos `products`.

---

### Tabela: `products`

Produto cadastrado em uma loja.

| Coluna            | Tipo                       | Nulo | Padrão       | Descrição                                              |
|-------------------|----------------------------|------|--------------|--------------------------------------------------------|
| `id`              | INTEGER (PK, auto)         | Não  | —            | Identificador interno                                  |
| `pid`             | UUID (único)               | Não  | —            | Identificador público (UUID v4)                        |
| `store_id`        | INTEGER (FK → stores)      | Não  | —            | Loja à qual pertence                                   |
| `title`           | VARCHAR(512)               | Não  | —            | Título do produto                                      |
| `slug`            | VARCHAR(512)               | Não  | —            | Slug único por loja (anti-colisão automática)          |
| `description`     | TEXT                       | Não  | `""`         | Descrição completa do produto                          |
| `handle`          | VARCHAR(256)               | Não  | —            | Handle de URL (padrão = slug)                          |
| `status`          | VARCHAR(20)                | Não  | `"draft"`    | Status: `draft`, `active`, `archived`                  |
| `product_type`    | VARCHAR(20)                | Não  | `"physical"` | Tipo: `physical`, `digital`, `service`                 |
| `category_id`     | INTEGER (FK → categories)  | Sim  | NULL         | Categoria do produto                                   |
| `tags`            | JSONB                      | Não  | `[]`         | Lista de tags (array JSON)                             |
| `metadata`        | JSONB                      | Não  | `{}`         | Metadados adicionais                                   |
| `seo_title`       | VARCHAR(256)               | Sim  | NULL         | Título para SEO                                        |
| `seo_description` | VARCHAR(512)               | Sim  | NULL         | Descrição para SEO                                     |
| `weight`          | DECIMAL(10,2)              | Sim  | NULL         | Peso do produto (kg)                                   |
| `dimensions`      | JSONB                      | Sim  | NULL         | Dimensões: `{width, height, depth}`                    |
| `featured`        | BOOLEAN                    | Não  | `false`      | Se está em destaque                                    |
| `created_at`      | TIMESTAMPTZ                | Não  | now()        | Data de criação                                        |
| `updated_at`      | TIMESTAMPTZ                | Não  | now()        | Data da última atualização                             |
| `deleted_at`      | TIMESTAMPTZ                | Sim  | NULL         | Soft delete (NULL = ativo)                             |

**Índices:**
- `idx_products_store_slug` — em `(store_id, slug)`, único
- `idx_products_store_status` — em `(store_id, status)`
- `idx_products_featured` — em `(store_id, featured)`

**Relacionamentos:**
- `store_id` → `stores.id` (CASCADE DELETE)
- `category_id` → `categories.id` (SET NULL)
- Um `product` possui muitas `product_variants`, `product_images` e pode integrar muitas `collections`.

> **Nota sobre slugs:** O sistema gera slugs automaticamente a partir do título e garante unicidade dentro da loja. Em caso de colisão, um sufixo numérico é adicionado automaticamente (ex.: `meu-produto` → `meu-produto1` → `meu-produto2`).

---

### Tabela: `product_variants`

Variantes de um produto (tamanho, cor, etc.). Todo produto deve ter ao menos uma variante.

| Coluna               | Tipo                     | Nulo | Padrão  | Descrição                                         |
|----------------------|--------------------------|------|---------|---------------------------------------------------|
| `id`                 | INTEGER (PK, auto)       | Não  | —       | Identificador interno                             |
| `pid`                | UUID (único)             | Não  | —       | Identificador público (UUID v4)                   |
| `product_id`         | INTEGER (FK → products)  | Não  | —       | Produto ao qual pertence                          |
| `sku`                | VARCHAR(128) (único)     | Não  | —       | Código SKU único                                  |
| `title`              | VARCHAR(256)             | Não  | —       | Título da variante (ex.: "G / Azul")              |
| `option_values`      | JSONB                    | Não  | `{}`    | Valores das opções (ex.: `{"tamanho":"G"}`)       |
| `inventory_quantity` | INTEGER                  | Não  | `0`     | Quantidade em estoque                             |
| `allow_backorder`    | BOOLEAN                  | Não  | `false` | Permitir pedido mesmo sem estoque                 |
| `weight`             | DECIMAL(10,2)            | Sim  | NULL    | Peso da variante (sobrescreve o do produto)       |
| `dimensions`         | JSONB                    | Sim  | NULL    | Dimensões da variante                             |
| `metadata`           | JSONB                    | Não  | `{}`    | Metadados adicionais                              |
| `sort_order`         | INTEGER                  | Não  | `0`     | Ordem de exibição                                 |
| `created_at`         | TIMESTAMPTZ              | Não  | now()   | Data de criação                                   |
| `updated_at`         | TIMESTAMPTZ              | Não  | now()   | Data da última atualização                        |
| `deleted_at`         | TIMESTAMPTZ              | Sim  | NULL    | Soft delete (NULL = ativa)                        |

**Índices:**
- `idx_variants_product` — em `product_id`
- `idx_variants_sku` — em `sku`, único

**Relacionamentos:**
- `product_id` → `products.id` (CASCADE DELETE)
- Uma `product_variant` possui muitos `prices` e pode ter muitas `product_images`.

---

### Tabela: `prices`

Preços de uma variante, com suporte a múltiplas moedas, regiões e faixas de quantidade (preço por volume).

| Coluna         | Tipo                              | Nulo | Padrão  | Descrição                                          |
|----------------|-----------------------------------|------|---------|----------------------------------------------------|
| `id`           | INTEGER (PK, auto)                | Não  | —       | Identificador interno                              |
| `pid`          | UUID (único)                      | Não  | —       | Identificador público (UUID v4)                    |
| `variant_id`   | INTEGER (FK → product_variants)   | Não  | —       | Variante à qual pertence                           |
| `amount`       | BIGINT                            | Não  | —       | Valor em centavos (ex.: 1990 = R$ 19,90)           |
| `currency`     | CHAR(3)                           | Não  | `"BRL"` | Moeda (código ISO 4217)                            |
| `region`       | VARCHAR(10)                       | Sim  | NULL    | Região de aplicação (ex.: `"SP"`, `"US"`)          |
| `min_quantity`  | INTEGER                          | Não  | `1`     | Quantidade mínima para aplicar este preço          |
| `max_quantity`  | INTEGER                          | Sim  | NULL    | Quantidade máxima (NULL = sem limite)              |
| `starts_at`    | TIMESTAMPTZ                       | Sim  | NULL    | Início da vigência do preço                        |
| `ends_at`      | TIMESTAMPTZ                       | Sim  | NULL    | Fim da vigência do preço                           |
| `created_at`   | TIMESTAMPTZ                       | Não  | now()   | Data de criação                                    |
| `updated_at`   | TIMESTAMPTZ                       | Não  | now()   | Data da última atualização                         |

**Índices:**
- `idx_prices_variant` — em `variant_id`

**Relacionamentos:**
- `variant_id` → `product_variants.id` (CASCADE DELETE)

---

### Tabela: `product_images`

Imagens associadas a um produto ou variante específica.

| Coluna       | Tipo                              | Nulo | Padrão | Descrição                                         |
|--------------|-----------------------------------|------|--------|---------------------------------------------------|
| `id`         | INTEGER (PK, auto)                | Não  | —      | Identificador interno                             |
| `pid`        | UUID (único)                      | Não  | —      | Identificador público (UUID v4)                   |
| `product_id` | INTEGER (FK → products)           | Não  | —      | Produto ao qual pertence                          |
| `variant_id` | INTEGER (FK → product_variants)   | Sim  | NULL   | Variante específica (NULL = imagem geral)         |
| `url`        | VARCHAR(1024)                     | Não  | —      | URL da imagem                                     |
| `alt_text`   | VARCHAR(256)                      | Não  | `""`   | Texto alternativo (acessibilidade/SEO)            |
| `sort_order` | INTEGER                           | Não  | `0`    | Ordem de exibição                                 |
| `created_at` | TIMESTAMPTZ                       | Não  | now()  | Data de criação                                   |
| `updated_at` | TIMESTAMPTZ                       | Não  | now()  | Data da última atualização                        |

**Relacionamentos:**
- `product_id` → `products.id` (CASCADE DELETE)
- `variant_id` → `product_variants.id` (SET NULL)

---

### Tabela: `collections`

Coleções são agrupamentos curados de produtos (ex.: "Promoção de Verão", "Mais Vendidos").

| Coluna        | Tipo                    | Nulo | Padrão  | Descrição                                        |
|---------------|-------------------------|------|---------|--------------------------------------------------|
| `id`          | INTEGER (PK, auto)      | Não  | —       | Identificador interno                            |
| `pid`         | UUID (único)            | Não  | —       | Identificador público (UUID v4)                  |
| `store_id`    | INTEGER (FK → stores)   | Não  | —       | Loja à qual pertence                             |
| `title`       | VARCHAR(256)            | Não  | —       | Título da coleção                                |
| `slug`        | VARCHAR(256)            | Não  | —       | Slug único por loja                              |
| `description` | TEXT                    | Não  | `""`    | Descrição da coleção                             |
| `image_url`   | VARCHAR(1024)           | Sim  | NULL    | URL da imagem de capa                            |
| `published`   | BOOLEAN                 | Não  | `false` | Se está publicada e visível na loja              |
| `sort_order`  | INTEGER                 | Não  | `0`     | Ordem de exibição                                |
| `metadata`    | JSONB                   | Não  | `{}`    | Metadados adicionais                             |
| `created_at`  | TIMESTAMPTZ             | Não  | now()   | Data de criação                                  |
| `updated_at`  | TIMESTAMPTZ             | Não  | now()   | Data da última atualização                       |
| `deleted_at`  | TIMESTAMPTZ             | Sim  | NULL    | Soft delete (NULL = ativa)                       |

**Índices:**
- `idx_collections_store_slug` — em `(store_id, slug)`, único

**Relacionamentos:**
- `store_id` → `stores.id` (CASCADE DELETE)
- Uma `collection` contém muitos `products` através de `collection_products`.

---

### Tabela: `collection_products`

Tabela de junção N:N entre coleções e produtos.

| Coluna          | Tipo                         | Nulo | Padrão | Descrição                           |
|-----------------|------------------------------|------|--------|-------------------------------------|
| `id`            | INTEGER (PK, auto)           | Não  | —      | Identificador interno               |
| `collection_id` | INTEGER (FK → collections)   | Não  | —      | Coleção                             |
| `product_id`    | INTEGER (FK → products)      | Não  | —      | Produto                             |
| `sort_order`    | INTEGER                      | Não  | `0`    | Ordem do produto dentro da coleção  |
| `created_at`    | TIMESTAMPTZ                  | Não  | now()  | Data de criação                     |

**Índices:**
- `idx_colprod_unique` — em `(collection_id, product_id)`, único

**Relacionamentos:**
- `collection_id` → `collections.id` (CASCADE DELETE)
- `product_id` → `products.id` (CASCADE DELETE)

---

### Tabela: `customers`

Clientes das lojas. Podem ou não ter conta de usuário associada.

| Coluna                | Tipo                    | Nulo | Padrão  | Descrição                                           |
|-----------------------|-------------------------|------|---------|-----------------------------------------------------|
| `id`                  | INTEGER (PK, auto)      | Não  | —       | Identificador interno                               |
| `pid`                 | UUID (único)            | Não  | —       | Identificador público (UUID v4)                     |
| `store_id`            | INTEGER (FK → stores)   | Não  | —       | Loja à qual pertence                                |
| `email`               | VARCHAR(256)            | Não  | —       | E-mail do cliente (único por loja)                  |
| `first_name`          | VARCHAR(128)            | Não  | `""`    | Primeiro nome                                       |
| `last_name`           | VARCHAR(128)            | Não  | `""`    | Sobrenome                                           |
| `phone`               | VARCHAR(32)             | Sim  | NULL    | Telefone de contato                                 |
| `has_account`         | BOOLEAN                 | Não  | `false` | Possui conta de acesso à loja                       |
| `user_id`             | INTEGER (FK → users)    | Sim  | NULL    | Usuário vinculado (opcional)                        |
| `metadata`            | JSONB                   | Não  | `{}`    | Metadados adicionais                                |
| `marketing_consent`   | BOOLEAN                 | Não  | `false` | Consentimento para marketing                        |
| `analytics_session_id`| VARCHAR(128)            | Sim  | NULL    | ID de sessão para analytics                         |
| `last_seen_at`        | TIMESTAMPTZ             | Sim  | NULL    | Última atividade registrada                         |
| `created_at`          | TIMESTAMPTZ             | Não  | now()   | Data de criação                                     |
| `updated_at`          | TIMESTAMPTZ             | Não  | now()   | Data da última atualização                          |
| `deleted_at`          | TIMESTAMPTZ             | Sim  | NULL    | Soft delete (NULL = ativo)                          |

**Índices:**
- `idx_customers_store_email` — em `(store_id, email)`, único

**Relacionamentos:**
- `store_id` → `stores.id` (CASCADE DELETE)
- `user_id` → `users.id` (SET NULL)
- Um `customer` pode ter muitos `addresses`, `carts` e `orders`.

---

### Tabela: `addresses`

Endereços de entrega e cobrança dos clientes.

| Coluna               | Tipo                       | Nulo | Padrão | Descrição                                    |
|----------------------|----------------------------|------|--------|----------------------------------------------|
| `id`                 | INTEGER (PK, auto)         | Não  | —      | Identificador interno                        |
| `pid`                | UUID (único)               | Não  | —      | Identificador público (UUID v4)              |
| `customer_id`        | INTEGER (FK → customers)   | Não  | —      | Cliente ao qual pertence                     |
| `first_name`         | VARCHAR(128)               | Não  | `""`   | Nome do destinatário                         |
| `last_name`          | VARCHAR(128)               | Não  | `""`   | Sobrenome do destinatário                    |
| `company`            | VARCHAR(256)               | Sim  | NULL   | Nome da empresa                              |
| `address_line1`      | VARCHAR(512)               | Não  | —      | Logradouro e número                          |
| `address_line2`      | VARCHAR(512)               | Sim  | NULL   | Complemento                                  |
| `city`               | VARCHAR(128)               | Não  | —      | Cidade                                       |
| `state`              | VARCHAR(64)                | Não  | —      | Estado / Província                           |
| `postal_code`        | VARCHAR(20)                | Não  | —      | CEP / Código postal                          |
| `country`            | CHAR(2)                    | Não  | `"BR"` | País (código ISO 3166-1 alpha-2)             |
| `phone`              | VARCHAR(32)                | Sim  | NULL   | Telefone para entrega                        |
| `is_default_shipping`| BOOLEAN                    | Não  | `false`| Endereço padrão de entrega                   |
| `is_default_billing` | BOOLEAN                    | Não  | `false`| Endereço padrão de cobrança                  |
| `created_at`         | TIMESTAMPTZ                | Não  | now()  | Data de criação                              |
| `updated_at`         | TIMESTAMPTZ                | Não  | now()  | Data da última atualização                   |

**Relacionamentos:**
- `customer_id` → `customers.id` (CASCADE DELETE)

---

### Tabela: `carts`

Carrinhos de compra, ativos ou abandonados.

| Coluna            | Tipo                       | Nulo | Padrão     | Descrição                                          |
|-------------------|----------------------------|------|------------|----------------------------------------------------|
| `id`              | INTEGER (PK, auto)         | Não  | —          | Identificador interno                              |
| `pid`             | UUID (único)               | Não  | —          | Identificador público (UUID v4)                    |
| `store_id`        | INTEGER (FK → stores)      | Não  | —          | Loja à qual pertence                               |
| `customer_id`     | INTEGER (FK → customers)   | Sim  | NULL       | Cliente (NULL = carrinho anônimo)                  |
| `session_id`      | VARCHAR(256)               | Não  | —          | ID de sessão do visitante                          |
| `status`          | VARCHAR(20)                | Não  | `"active"` | Status: `active`, `abandoned`, `completed`         |
| `email`           | VARCHAR(256)               | Sim  | NULL       | E-mail capturado (para recuperação de carrinho)    |
| `currency`        | CHAR(3)                    | Não  | `"BRL"`    | Moeda do carrinho                                  |
| `subtotal`        | BIGINT                     | Não  | `0`        | Subtotal em centavos                               |
| `tax`             | BIGINT                     | Não  | `0`        | Impostos em centavos                               |
| `shipping`        | BIGINT                     | Não  | `0`        | Frete em centavos                                  |
| `total`           | BIGINT                     | Não  | `0`        | Total em centavos                                  |
| `metadata`        | JSONB                      | Não  | `{}`       | Metadados adicionais                               |
| `expires_at`      | TIMESTAMPTZ                | Sim  | NULL       | Expiração do carrinho                              |
| `completed_at`    | TIMESTAMPTZ                | Sim  | NULL       | Data de conclusão (conversão em pedido)            |
| `last_activity_at`| TIMESTAMPTZ                | Não  | now()      | Última atividade (usado para detectar abandono)    |
| `recovery_token`  | VARCHAR(128)               | Sim  | NULL       | Token para e-mail de recuperação de carrinho       |
| `created_at`      | TIMESTAMPTZ                | Não  | now()      | Data de criação                                    |
| `updated_at`      | TIMESTAMPTZ                | Não  | now()      | Data da última atualização                         |

**Índices:**
- `idx_carts_session` — em `(store_id, session_id)`
- `idx_carts_abandoned` — em `(status, last_activity_at)` (para job de carrinhos abandonados)

**Relacionamentos:**
- `store_id` → `stores.id` (CASCADE DELETE)
- `customer_id` → `customers.id` (SET NULL)
- Um `cart` possui muitos `cart_items`.

---

### Tabela: `cart_items`

Itens dentro de um carrinho de compra.

| Coluna       | Tipo                              | Nulo | Padrão | Descrição                              |
|--------------|-----------------------------------|------|--------|----------------------------------------|
| `id`         | INTEGER (PK, auto)                | Não  | —      | Identificador interno                  |
| `pid`        | UUID (único)                      | Não  | —      | Identificador público (UUID v4)        |
| `cart_id`    | INTEGER (FK → carts)              | Não  | —      | Carrinho ao qual pertence              |
| `variant_id` | INTEGER (FK → product_variants)   | Não  | —      | Variante do produto                    |
| `quantity`   | INTEGER                           | Não  | `1`    | Quantidade                             |
| `unit_price` | BIGINT                            | Não  | —      | Preço unitário em centavos             |
| `total`      | BIGINT                            | Não  | —      | Total do item em centavos              |
| `metadata`   | JSONB                             | Não  | `{}`   | Metadados adicionais                   |
| `created_at` | TIMESTAMPTZ                       | Não  | now()  | Data de criação                        |
| `updated_at` | TIMESTAMPTZ                       | Não  | now()  | Data da última atualização             |

**Índices:**
- `idx_cart_items_cart` — em `cart_id`

**Relacionamentos:**
- `cart_id` → `carts.id` (CASCADE DELETE)
- `variant_id` → `product_variants.id` (CASCADE DELETE)

---

### Tabela: `orders`

Pedidos realizados na loja.

| Coluna               | Tipo                        | Nulo | Padrão      | Descrição                                                |
|----------------------|-----------------------------|------|-------------|----------------------------------------------------------|
| `id`                 | INTEGER (PK, auto)          | Não  | —           | Identificador interno                                    |
| `pid`                | UUID (único)                | Não  | —           | Identificador público (UUID v4)                          |
| `store_id`           | INTEGER (FK → stores)       | Não  | —           | Loja à qual pertence                                     |
| `customer_id`        | INTEGER (FK → customers)    | Não  | —           | Cliente que realizou o pedido                            |
| `cart_id`            | INTEGER (FK → carts)        | Sim  | NULL        | Carrinho de origem (referência histórica)                |
| `order_number`       | VARCHAR(64) (único)         | Não  | —           | Número do pedido visível ao cliente (ex.: `#1001`)       |
| `status`             | VARCHAR(20)                 | Não  | `"pending"` | Status: `pending`, `confirmed`, `processing`, `shipped`, `delivered`, `cancelled` |
| `payment_status`     | VARCHAR(20)                 | Não  | `"pending"` | Status financeiro: `pending`, `paid`, `refunded`, `failed` |
| `fulfillment_status` | VARCHAR(30)                 | Não  | `"pending"` | Status de entrega: `pending`, `partial`, `fulfilled`     |
| `currency`           | CHAR(3)                     | Não  | `"BRL"`     | Moeda do pedido                                          |
| `subtotal`           | BIGINT                      | Não  | `0`         | Subtotal em centavos                                     |
| `tax`                | BIGINT                      | Não  | `0`         | Impostos em centavos                                     |
| `shipping`           | BIGINT                      | Não  | `0`         | Frete em centavos                                        |
| `discount`           | BIGINT                      | Não  | `0`         | Desconto aplicado em centavos                            |
| `total`              | BIGINT                      | Não  | `0`         | Total final em centavos                                  |
| `shipping_address_id`| INTEGER (FK → addresses)    | Sim  | NULL        | Endereço de entrega                                      |
| `billing_address_id` | INTEGER (FK → addresses)    | Sim  | NULL        | Endereço de cobrança                                     |
| `payment_method`     | VARCHAR(32)                 | Sim  | NULL        | Método de pagamento (ex.: `"pix"`, `"credit_card"`)      |
| `payment_provider`   | VARCHAR(32)                 | Sim  | NULL        | Provedor de pagamento (ex.: `"asaas"`, `"stripe"`)       |
| `payment_data`       | JSONB                       | Não  | `{}`        | Dados adicionais do pagamento (JSON)                     |
| `notes`              | TEXT                        | Sim  | NULL        | Observações do cliente ou do admin                       |
| `metadata`           | JSONB                       | Não  | `{}`        | Metadados adicionais                                     |
| `canceled_at`        | TIMESTAMPTZ                 | Sim  | NULL        | Data de cancelamento                                     |
| `paid_at`            | TIMESTAMPTZ                 | Sim  | NULL        | Data de confirmação do pagamento                         |
| `created_at`         | TIMESTAMPTZ                 | Não  | now()       | Data de criação                                          |
| `updated_at`         | TIMESTAMPTZ                 | Não  | now()       | Data da última atualização                               |

**Índices:**
- `idx_orders_store_created` — em `(store_id, created_at)`
- `idx_orders_customer` — em `customer_id`

**Relacionamentos:**
- `store_id` → `stores.id` (CASCADE DELETE)
- `customer_id` → `customers.id` (RESTRICT — pedidos não são excluídos com o cliente)
- `cart_id` → `carts.id` (SET NULL)
- `shipping_address_id` → `addresses.id` (SET NULL)
- `billing_address_id` → `addresses.id` (SET NULL)
- Um `order` possui muitos `order_items`.

---

### Tabela: `order_items`

Itens de um pedido. Os dados são desnormalizados para preservar o histórico mesmo que o produto seja alterado posteriormente.

| Coluna       | Tipo                              | Nulo | Padrão | Descrição                                              |
|--------------|-----------------------------------|------|--------|--------------------------------------------------------|
| `id`         | INTEGER (PK, auto)                | Não  | —      | Identificador interno                                  |
| `pid`        | UUID (único)                      | Não  | —      | Identificador público (UUID v4)                        |
| `order_id`   | INTEGER (FK → orders)             | Não  | —      | Pedido ao qual pertence                                |
| `variant_id` | INTEGER (FK → product_variants)   | Sim  | NULL   | Variante (pode ficar NULL se o produto for excluído)   |
| `title`      | VARCHAR(512)                      | Não  | —      | Título do produto no momento da compra                 |
| `sku`        | VARCHAR(128)                      | Não  | —      | SKU da variante no momento da compra                   |
| `quantity`   | INTEGER                           | Não  | —      | Quantidade                                             |
| `unit_price` | BIGINT                            | Não  | —      | Preço unitário em centavos no momento da compra        |
| `total`      | BIGINT                            | Não  | —      | Total do item em centavos                              |
| `metadata`   | JSONB                             | Não  | `{}`   | Metadados adicionais                                   |
| `created_at` | TIMESTAMPTZ                       | Não  | now()  | Data de criação                                        |
| `updated_at` | TIMESTAMPTZ                       | Não  | now()  | Data da última atualização                             |

**Índices:**
- `idx_order_items_order` — em `order_id`

**Relacionamentos:**
- `order_id` → `orders.id` (CASCADE DELETE)
- `variant_id` → `product_variants.id` (SET NULL)

---

## Convenções do Banco de Dados

| Convenção              | Detalhe                                                                 |
|------------------------|-------------------------------------------------------------------------|
| **Soft Delete**        | Registros deletados recebem valor em `deleted_at` em vez de serem removidos |
| **UUID público**       | Toda tabela possui `pid` (UUID v4) exposto nas APIs; `id` é interno     |
| **Valores monetários** | Armazenados em **centavos** como `BIGINT` para evitar erros de ponto flutuante |
| **Timestamps**         | Todas as tabelas possuem `created_at` e `updated_at` com fuso horário   |
| **Slugs únicos**       | Slugs são únicos dentro do escopo da loja, com geração anti-colisão automática |
| **JSON/JSONB**         | Campos flexíveis (`metadata`, `config`, `tags`) usam JSONB              |

---

## Executando as Migrações

```sh
# Aplicar todas as migrações
cargo loco db migrate

# Reverter a última migração
cargo loco db down

# Ver status das migrações
cargo loco db status
```

---

## Ajuda

Consulte a [documentação do Loco](https://loco.rs/docs/getting-started/guide/) para mais informações sobre configuração, deploy e extensão do framework.

