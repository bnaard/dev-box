---
name: reflex-python
description: Reflex Python web framework for building full-stack apps in pure Python. Components, state management, and deployment. Use when building Reflex apps, designing component hierarchies, or managing app state.
allowed-tools: Bash(reflex:*) Bash(python:*) Read Write
---

# Reflex Python

## When to Use

When the user is building web apps with Reflex, asks about component design, state
management, routing in Reflex, or says "create a Reflex app" or "add a page" or
"manage state in Reflex".

## Instructions

### 1. App Structure

- Initialize with `reflex init` -- creates `rxconfig.py` and app directory
- Entry point is `app/app.py` with `app = rx.App()`
- Each page is a function decorated with `@rx.page(route="/path", title="Page Title")`
- File-based routing: files in `app/pages/` map to routes automatically
- Configuration in `rxconfig.py`: `rx.Config(app_name="myapp", db_url=...)`
- Static assets go in `assets/` directory, accessible at `/filename`

### 2. Components

- All UI is built from `rx.*` components -- Python wrappers around React components
- Layout components: `rx.box`, `rx.flex`, `rx.grid`, `rx.stack`, `rx.center`
- Display: `rx.text`, `rx.heading`, `rx.image`, `rx.badge`, `rx.code_block`
- Input: `rx.input`, `rx.text_area`, `rx.select`, `rx.checkbox`, `rx.slider`, `rx.switch`
- Feedback: `rx.alert`, `rx.toast`, `rx.progress`, `rx.spinner`, `rx.skeleton`
- Navigation: `rx.link`, `rx.breadcrumb`, `rx.tabs`
- Components accept props as keyword arguments: `rx.button("Click", color_scheme="blue")`
- Nest components as positional arguments: `rx.box(rx.text("Hello"), rx.button("Go"))`

### 3. State Management

- Define state as a class inheriting `rx.State`
- State vars are class attributes with type annotations: `count: int = 0`
- Event handlers are methods that modify state: `def increment(self): self.count += 1`
- Bind state to UI: `rx.text(State.count)` -- auto-updates on change
- Computed vars use the `@rx.var` decorator for derived values
- State is per-session -- each browser tab gets its own instance
- Substates for modularity: `class FormState(State): ...`

### 4. Event Handlers

- Attach handlers to events: `rx.button("Add", on_click=State.increment)`
- Handlers receive event data: `def on_change(self, value: str): self.query = value`
- Use `rx.input(value=State.query, on_change=State.set_query)` for two-way binding
- `set_<var>` is auto-generated for each state var
- Background tasks with `@rx.event(background=True)` for long-running operations
- Yield from handlers to send intermediate UI updates
- Chain events: return `State.other_handler` from a handler

### 5. Styling

- Inline styles as props: `rx.box(padding="4", bg="blue.500", border_radius="md")`
- Uses Radix UI design tokens: color scales (`blue.1` to `blue.12`), spacing, radii
- Responsive props with list: `rx.box(columns=["1", "2", "3"])` (mobile, tablet, desktop)
- Global styles in `rxconfig.py` or `rx.App(style={...})`
- Use `rx.color_mode.toggle` and `rx.color_mode_cond()` for light/dark themes
- CSS class support: `rx.box(class_name="custom-class")`

### 6. Routing and Navigation

- Decorator-based: `@rx.page(route="/users/[id]", title="User Profile")`
- Dynamic segments via brackets: access with `self.router.page.params["id"]`
- Navigate programmatically: `return rx.redirect("/dashboard")`
- Use `rx.link("Home", href="/")` for navigation links
- On-load events: `@rx.page(on_load=State.load_data)`
- 404 handling: define a catch-all page with `@rx.page(route="/[...catchall]")`

### 7. Database Integration

- Built-in SQLModel integration: define models as `class User(rx.Model): ...`
- Query in event handlers: `with rx.session() as session: users = session.exec(...)`
- Migrations handled automatically with `reflex db migrate`
- Default SQLite for development; configure PostgreSQL for production in `rxconfig.py`

### 8. Deployment

- `reflex deploy` for Reflex Cloud (managed hosting)
- Self-host: `reflex export --frontend-only` produces static frontend + API backend
- Docker: use `reflex export` then serve frontend via nginx, backend via uvicorn
- Set `REFLEX_ENV=prod` for production optimizations

### 9. Common Mistakes

- Mutating state outside event handlers -- state changes must occur inside handlers
- Forgetting type annotations on state vars -- Reflex needs them for serialization
- Using Python-only objects in state -- all vars must be JSON-serializable
- Blocking the event loop -- use background tasks for CPU/IO-heavy work

## References

- `references/component-reference.md` -- Common components, state patterns, and examples
- [Reflex Documentation](https://reflex.dev/docs/)
- [Component Library](https://reflex.dev/docs/library/)

## Examples

**User:** "Create a todo app with Reflex"
**Agent:** Defines a `TodoState` with a list of todos and input field, creates event
handlers for add/delete/toggle, builds UI with `rx.input`, `rx.button`, and
`rx.foreach(TodoState.todos, render_todo)` to render the list dynamically.

**User:** "Add authentication to my Reflex app"
**Agent:** Creates a `LoginState` substate with username/password vars, implements
`login` and `logout` handlers that validate credentials, uses `rx.cond(State.logged_in,
dashboard(), login_page())` for conditional rendering, and redirects on auth state change.

**User:** "How do I fetch data from an API in Reflex?"
**Agent:** Creates an event handler decorated with `@rx.event(background=True)` that
uses `httpx` to fetch data, stores results in a state var, and triggers the handler
with `on_load` on the page decorator for automatic loading.
