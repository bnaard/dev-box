---
name: fastapi-patterns
description: FastAPI patterns including dependency injection, Pydantic models, async endpoints, middleware, and testing. Use when building FastAPI applications, designing API endpoints, or reviewing FastAPI code.
allowed-tools: Bash(python:*) Bash(uvicorn:*) Read Write
---

# FastAPI Patterns

## When to Use

When the user is building APIs with FastAPI, asks about endpoint design, dependency
injection, authentication, or says "create an API" or "add an endpoint" or "test
this FastAPI app".

## Instructions

### 1. Route Definitions

- Use HTTP method decorators: `@app.get`, `@app.post`, `@app.put`, `@app.delete`, `@app.patch`
- Path parameters with types: `@app.get("/users/{user_id}")` + `user_id: int`
- Query parameters as function args with defaults: `skip: int = 0, limit: int = 100`
- Use `APIRouter` to organize routes into modules: `router = APIRouter(prefix="/users")`
- Add tags for OpenAPI grouping: `@router.get("/", tags=["users"])`
- Use `status_code` parameter: `@app.post("/users", status_code=201)`

### 2. Pydantic Models (Request/Response)

- Define request bodies as Pydantic `BaseModel` subclasses
- Use `model_config = ConfigDict(from_attributes=True)` for ORM compatibility
- Separate Create, Update, and Response schemas: `UserCreate`, `UserUpdate`, `UserResponse`
- Use `response_model` to filter output: `@app.get("/users/{id}", response_model=UserResponse)`
- Optional fields with `field: str | None = None` for PATCH operations
- Validate with `Field()`: `name: str = Field(min_length=1, max_length=100)`
- Nest models for complex structures: `address: Address` inside `User`

### 3. Dependency Injection

- Define dependencies as functions or classes: `def get_db(): yield session`
- Inject with `Depends()`: `def read_users(db: Session = Depends(get_db))`
- Chain dependencies: auth depends on token, which depends on header extraction
- Use `Annotated` for reusable dependencies: `DbDep = Annotated[Session, Depends(get_db)]`
- App-level dependencies apply to all routes: `app = FastAPI(dependencies=[Depends(verify_api_key)])`
- Yield-based dependencies handle cleanup: `yield session; session.close()`

### 4. Async Endpoints

- Use `async def` for I/O-bound operations (database, HTTP calls, file I/O)
- Use plain `def` for CPU-bound work — FastAPI runs it in a thread pool automatically
- Pair with async libraries: `httpx.AsyncClient`, `asyncpg`, `aiofiles`
- Avoid blocking calls inside `async def` — they block the event loop
- Use `asyncio.gather()` for concurrent async operations within a handler

### 5. Middleware

- CORS: `app.add_middleware(CORSMiddleware, allow_origins=[...], allow_methods=["*"])`
- Custom middleware with `@app.middleware("http")` or `BaseHTTPMiddleware`
- Request timing: measure `time.perf_counter()` before/after `await call_next(request)`
- Trusted hosts: `TrustedHostMiddleware` to prevent host header attacks
- GZip: `GZipMiddleware(minimum_size=1000)` for response compression
- Middleware order matters — last added runs first (outermost)

### 6. Authentication

- OAuth2 with password flow: `OAuth2PasswordBearer(tokenUrl="token")`
- Decode JWT in a dependency; raise `HTTPException(401)` on failure
- Scopes for permission levels: `Security(get_current_user, scopes=["admin"])`
- API key via header: `api_key: str = Depends(APIKeyHeader(name="X-API-Key"))`
- Always hash passwords with bcrypt; never store plaintext

### 7. Background Tasks

- Inject `BackgroundTasks` and add work: `background_tasks.add_task(send_email, to=email)`
- Tasks run after the response is sent — do not block the response
- For heavy work, use Celery or ARQ instead of FastAPI background tasks
- Background tasks share the same process — failures do not crash the request

### 8. WebSocket Support

- Define with `@app.websocket("/ws")` + `async def ws(websocket: WebSocket)`
- Accept with `await websocket.accept()`
- Send/receive: `await websocket.receive_text()`, `await websocket.send_json(data)`
- Handle disconnects with `WebSocketDisconnect` exception
- Use a connection manager class for broadcasting to multiple clients

### 9. Error Handling

- Raise `HTTPException(status_code=404, detail="Not found")` for client errors
- Custom exception handlers: `@app.exception_handler(ValueError)`
- Validation errors return 422 automatically with detailed field information
- Use `RequestValidationError` handler to customize validation error format
- Return consistent error schema: `{"detail": "message", "code": "ERROR_CODE"}`

### 10. Testing

- Use `TestClient` from Starlette: `client = TestClient(app)`
- Override dependencies in tests: `app.dependency_overrides[get_db] = mock_db`
- Async tests with `httpx.AsyncClient`: `async with AsyncClient(app=app) as client:`
- Test WebSockets: `with client.websocket_connect("/ws") as ws:`
- Always clean up dependency overrides after tests

## References

- `references/endpoint-patterns.md` — CRUD, pagination, file upload, and auth patterns
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Pydantic V2 Documentation](https://docs.pydantic.dev/)

## Examples

**User:** "Create a REST API for a blog with posts and comments"
**Agent:** Creates Pydantic schemas for `PostCreate`, `PostResponse`, `CommentCreate`,
defines APIRouter modules for `/posts` and `/posts/{id}/comments`, implements CRUD
handlers with SQLAlchemy dependency injection, and adds pagination to list endpoints.

**User:** "Add JWT authentication to my FastAPI app"
**Agent:** Installs `python-jose` and `passlib`, creates `OAuth2PasswordBearer` scheme,
implements `/token` endpoint that validates credentials and returns JWT, creates
`get_current_user` dependency that decodes and verifies the token, and protects
routes with `Depends(get_current_user)`.

**User:** "Write tests for my FastAPI endpoints"
**Agent:** Creates a test file with `TestClient`, overrides the database dependency
with an in-memory SQLite session, writes tests for each endpoint covering success
and error cases, and verifies response status codes and body schemas.
