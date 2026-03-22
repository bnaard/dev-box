---
name: rag-engineering
description: Retrieval-Augmented Generation pipeline design including document ingestion, chunking, embedding, vector stores, retrieval strategies, and evaluation. Use when building RAG systems, optimizing retrieval quality, or debugging RAG pipelines.
allowed-tools: Bash Read Write
---

# RAG Engineering

## When to Use

- Building a new RAG pipeline from scratch
- Choosing chunking strategies, embedding models, or vector stores
- Retrieval quality is poor (irrelevant contexts, missed answers)
- Evaluating or benchmarking an existing RAG system
- Debugging hallucinations or unfaithful answers in RAG output
- Scaling a RAG system or migrating vector stores

## Instructions

### 1. RAG Architecture Overview

A RAG pipeline has two phases: **indexing** (offline) and **retrieval + generation** (online).

**Indexing:** Documents -> Parse -> Chunk -> Embed -> Store in vector DB
**Query:** User query -> Embed -> Retrieve top-k -> Construct prompt -> LLM generates answer

Design decisions cascade: bad chunking ruins retrieval no matter how good your embeddings are. Start from the data and work forward.

### 2. Document Ingestion

Parse source documents into clean text with metadata preserved.

- **PDF:** Use `pymupdf4llm` or `unstructured`. Preserve headings, tables, page numbers. OCR fallback for scanned docs via `tesseract`.
- **HTML:** Strip boilerplate with `trafilatura` or `readability-lxml`. Keep structural elements (headings, lists, code blocks).
- **Code:** Use tree-sitter for AST-aware parsing. Keep function/class boundaries intact. Preserve file paths and language metadata.
- **Metadata extraction:** Always capture: source URL/path, document title, section heading, page number, last-modified date. Metadata enables filtering at retrieval time.

### 3. Chunking Strategies

Choose based on document structure and query patterns. See `references/chunking-strategies.md`.

- **Fixed-size with overlap** — Simple baseline. 512-1024 tokens, 10-20% overlap.
- **Sentence-based** — Better boundary preservation. Group 3-5 sentences.
- **Semantic chunking** — Split where embedding similarity drops. Best for heterogeneous docs.
- **Recursive character** — LangChain default. Splits on `\n\n`, then `\n`, then space.
- **Document-structure-aware** — Split on headings, code blocks, sections. Best when structure is reliable.

Rule of thumb: chunk size should match expected answer granularity. FAQ -> small chunks. Technical explanations -> larger chunks.

### 4. Embedding Models

Selection criteria: domain match, dimensionality, speed, cost.

| Model | Dims | Context | Notes |
|-------|------|---------|-------|
| `text-embedding-3-small` | 1536 | 8191 | Good default, cheap |
| `text-embedding-3-large` | 3072 | 8191 | Better quality, 2x cost |
| `nomic-embed-text` | 768 | 8192 | Open-source, strong MTEB |
| `bge-large-en-v1.5` | 1024 | 512 | Open-source, good for code |
| `voyage-code-2` | 1536 | 16000 | Best for code retrieval |

Always benchmark on your actual queries before committing. MTEB leaderboard scores do not predict domain-specific performance.

Fine-tuning embeddings: generate synthetic query-passage pairs from your corpus, then fine-tune with contrastive loss. Even 1000 pairs helps.

### 5. Vector Stores

Choose based on scale, deployment constraints, and feature needs. Comparison:

- **FAISS** — In-memory, fastest for <10M vectors. No filtering. Use for prototyping or read-heavy workloads.
- **pgvector** — Postgres extension. Use when you already have Postgres and need joins/filtering. HNSW index for performance.
- **Chroma** — Embedded DB, easy local dev. Good DX, limited scale (<1M vectors practical).
- **Qdrant** — Rust-based, strong filtering, good hybrid search. Self-host or cloud. Production-ready.
- **Pinecone** — Fully managed, scales well. Use when you want zero ops. Expensive at scale.
- **Weaviate** — Built-in vectorization, multimodal. Heavier but feature-rich.

For most projects: start with Chroma locally, move to Qdrant or pgvector for production.

### 6. Retrieval Strategies

See `references/retrieval-patterns.md` for architecture details.

- **Dense retrieval** — Embed query, find nearest neighbors. Default approach.
- **Sparse retrieval (BM25)** — Term matching. Better for exact terms, acronyms, IDs.
- **Hybrid search** — Combine dense + sparse with reciprocal rank fusion. Almost always better than either alone.
- **Reranking** — Retrieve top-50 with fast method, rerank to top-5 with cross-encoder. Use `bge-reranker-v2-m3` or Cohere Rerank.
- **MMR (Maximal Marginal Relevance)** — Diversify results to reduce redundancy. Lambda 0.5-0.7 is typical.
- **Parent-document retrieval** — Embed small chunks, retrieve the parent section. Combines precision with context.
- **Multi-query** — LLM generates 3-5 query variants, retrieve for each, deduplicate. Helps with ambiguous queries.

### 7. Prompt Construction

The retrieval-to-generation handoff is where many RAG systems fail.

- **Context ordering:** Place most relevant chunks first. LLMs attend more to start and end of context.
- **Context window budget:** Reserve 30-40% of context for retrieved content, rest for system prompt + conversation.
- **Citation:** Number each chunk `[1]`, `[2]`, etc. Instruct the LLM to cite sources. Verify citations in post-processing.
- **Chunk deduplication:** Remove near-duplicate chunks before inserting into prompt. Cosine similarity > 0.95 = duplicate.
- **No-context handling:** If retrieval returns low-similarity results (below threshold), tell the LLM explicitly that no relevant information was found rather than stuffing bad context.

### 8. Evaluation

See `references/evaluation.md` for metrics and methodology.

Key metrics: **context precision** (are retrieved chunks relevant?), **context recall** (are all needed chunks retrieved?), **faithfulness** (is the answer grounded in context?), **answer relevance** (does it address the question?).

Use the RAGAS framework for automated evaluation. Build a golden dataset of 50-100 question/answer/context triples from your actual documents.

Common failure modes:
- **Hallucination** — Low faithfulness. Fix: better retrieval, explicit grounding instructions.
- **Incomplete answers** — Low context recall. Fix: smaller chunks, multi-query retrieval.
- **Wrong documents retrieved** — Low context precision. Fix: hybrid search, metadata filtering, reranking.
- **Correct retrieval, bad generation** — Model issue. Fix: better prompt template, stronger model.

## Examples

### Basic RAG Setup
User asks to build a RAG system over internal docs. Start with recursive chunking at 512 tokens, `text-embedding-3-small`, Chroma for storage. Implement basic dense retrieval with top-5. Build a simple eval set from real user questions. Iterate from this baseline.

### Optimizing Retrieval Quality
User reports RAG answers miss relevant information. Diagnose: check context recall by examining what was retrieved vs. what should have been. Try hybrid search (BM25 + dense), add reranking with a cross-encoder, experiment with smaller chunk sizes. Test each change against the eval set.

### Evaluating a RAG Pipeline
User wants to benchmark their RAG system. Help build a golden dataset: sample 50 questions, manually identify ideal context passages and reference answers. Run RAGAS metrics. Identify the weakest metric and focus optimization there. Set up automated eval in CI to catch regressions.
