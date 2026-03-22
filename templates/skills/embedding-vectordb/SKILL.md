---
name: embedding-vectordb
description: Vector embeddings and vector database patterns including model selection, similarity metrics, and index tuning. Use when building semantic search, choosing vector stores, or optimizing embedding pipelines.
---

# Embedding and Vector Database Patterns

## When to Use

- Choosing an embedding model for a new project
- Selecting or migrating between vector databases
- Optimizing semantic search quality or performance
- Implementing hybrid search (dense + sparse)
- Tuning vector index parameters for latency or recall
- Building or scaling an embedding pipeline

## Instructions

### 1. Embedding Model Selection

Choose based on your data type, quality needs, and constraints.

**Commercial models:**
| Model | Dims | Max Tokens | Strengths |
|-------|------|-----------|-----------|
| OpenAI `text-embedding-3-small` | 1536 | 8191 | Good default, low cost |
| OpenAI `text-embedding-3-large` | 3072 | 8191 | Higher quality, Matryoshka (dim reduction) |
| Cohere `embed-v3` | 1024 | 512 | Strong multilingual, search/classify modes |
| Voyage `voyage-3` | 1024 | 32000 | Long context, strong on code |

**Open-source models:**
| Model | Dims | Max Tokens | Strengths |
|-------|------|-----------|-----------|
| `nomic-embed-text-v1.5` | 768 | 8192 | Strong MTEB, Matryoshka support |
| `bge-large-en-v1.5` | 1024 | 512 | Good for English, well-tested |
| `e5-mistral-7b-instruct` | 4096 | 32768 | Best open-source quality, high compute |
| `all-MiniLM-L6-v2` | 384 | 256 | Tiny, fast, good for prototyping |

Selection criteria:
- **Domain match:** MTEB leaderboard scores are general; benchmark on YOUR data
- **Dimensionality:** Higher dims = better quality but more storage and slower search
- **Context window:** Must fit your chunk size
- **Latency:** Smaller models (MiniLM) for real-time; larger models (e5-mistral) for batch
- **Cost:** Open-source is free to run but needs GPU infrastructure

### 2. Dimensionality and Matryoshka Embeddings

Matryoshka embeddings (supported by OpenAI text-embedding-3 and nomic-embed) allow truncating dimensions without retraining:
- 3072 dims -> truncate to 1024 or 512 for faster search with modest quality loss
- Test quality at each dimension on your eval set before deciding

Rule of thumb: 256-512 dims is sufficient for most retrieval tasks. 1024+ dims for fine-grained similarity.

### 3. Similarity Metrics

| Metric | Formula | Range | Best For |
|--------|---------|-------|----------|
| Cosine similarity | dot(a,b) / (norm(a) * norm(b)) | [-1, 1] | Normalized embeddings (most common) |
| Dot product | dot(a,b) | (-inf, inf) | When magnitude matters (relevance scoring) |
| Euclidean (L2) | sqrt(sum((a-b)^2)) | [0, inf) | Spatial clustering |

Default choice: **cosine similarity**. Most embedding models are trained with cosine. If embeddings are already normalized (unit length), cosine = dot product.

### 4. Vector Database Selection

| Database | Architecture | Best For | Scaling |
|----------|-------------|----------|---------|
| **FAISS** | In-memory library | Prototyping, batch search, <10M vectors | Single machine |
| **pgvector** | Postgres extension | Postgres shops, need joins + filtering | Vertical + read replicas |
| **Chroma** | Embedded DB | Local dev, quick experiments | Single machine, <1M practical |
| **Qdrant** | Rust-based, client-server | Production workloads, advanced filtering | Horizontal sharding |
| **Weaviate** | Go-based, built-in vectorizers | Multimodal, auto-vectorization | Horizontal sharding |
| **Pinecone** | Managed SaaS | Zero ops, serverless option | Fully managed |
| **Milvus** | Distributed, cloud-native | Large scale (billions of vectors) | Horizontal, cloud-native |

Decision flow:
1. Prototyping? Use Chroma or FAISS locally.
2. Already using Postgres? Start with pgvector.
3. Need advanced filtering + production reliability? Qdrant or Weaviate.
4. Want zero ops? Pinecone.
5. Billions of vectors? Milvus.

### 5. Index Types and Tuning

**HNSW (Hierarchical Navigable Small Worlds):**
- Default choice for most vector DBs. Good balance of speed and recall.
- Key parameters: `M` (connections per node, default 16), `ef_construction` (build quality, default 200), `ef_search` (query quality, default 100).
- Higher M and ef = better recall but more memory and slower build.
- Start with defaults. Tune ef_search up if recall is too low, down if latency is too high.

**IVF (Inverted File Index):**
- Clusters vectors, then searches only relevant clusters.
- Parameter: `nlist` (number of clusters), `nprobe` (clusters to search at query time).
- Rule of thumb: nlist = sqrt(N), nprobe = nlist/10 to nlist/5.
- Faster than HNSW for very large datasets (>100M vectors).

**PQ (Product Quantization):**
- Compresses vectors for memory efficiency. Slight quality loss.
- Use when dataset does not fit in memory.
- Often combined with IVF: IVF-PQ.

### 6. Hybrid Search

Combine dense (semantic) and sparse (keyword) retrieval for best results:

- Dense retrieval catches semantic matches ("automobile" matches "car")
- Sparse retrieval (BM25) catches exact matches (product IDs, acronyms, proper nouns)
- Fusion: Reciprocal Rank Fusion (RRF) is the standard method
  - RRF score = sum(1 / (k + rank_i)) across retrievers, k typically 60
- Almost always outperforms either method alone

Implementation: Qdrant and Weaviate have built-in hybrid search. For others, run BM25 (Elasticsearch/OpenSearch) + vector search separately and fuse results.

### 7. Metadata Filtering and Multi-Tenancy

- Store metadata alongside vectors: source, date, category, tenant_id
- Pre-filter (filter before vector search) is more efficient than post-filter
- For multi-tenancy: use metadata filter on tenant_id, or separate collections per tenant
- Index metadata fields used in filters for performance

### 8. Embedding Pipeline Best Practices

- **Batch processing:** Embed in batches of 100-500 for API efficiency
- **Caching:** Cache embeddings by content hash to avoid re-embedding unchanged content
- **Normalization:** Normalize to unit length if using cosine similarity
- **Chunking alignment:** Chunk text before embedding; do not embed then chunk
- **Monitoring:** Track embedding latency, cache hit rate, and index size over time

## Examples

### Choosing a Stack for Semantic Search
User wants to add semantic search to a documentation site. Recommend `text-embedding-3-small` for cost-effective embedding, pgvector if they already use Postgres (otherwise Qdrant). Implement hybrid search: BM25 for exact term matches, dense retrieval for semantic. Chunk docs by section headers at 512 tokens. Set up an eval set of 50 queries with expected results to tune retrieval.

### Optimizing Vector Search Performance
User reports vector search is too slow (>500ms at 5M vectors). Check current index type and parameters. If using flat index, switch to HNSW. If HNSW, tune ef_search down (trade recall for speed). Consider Matryoshka dimension reduction (3072 -> 1024). Enable metadata pre-filtering to reduce search space. Benchmark each change against the eval set for recall and latency.

### Migrating Between Vector Databases
User wants to move from Chroma to Qdrant for production. Export all vectors and metadata from Chroma. Create Qdrant collection with matching distance metric and dimensions. Batch-upload vectors with metadata. Verify record count matches. Run the eval query set against both and compare recall and latency. Set up Qdrant with replication for high availability.
