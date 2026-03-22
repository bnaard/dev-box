---
name: flutter-development
description: Flutter/Dart development including widget architecture, state management, navigation, and cross-platform patterns. Use when building Flutter apps, choosing state management, or designing responsive mobile layouts.
allowed-tools: Bash(flutter:*) Bash(dart:*) Read Write
---

# Flutter Development

## When to Use

When the user is building mobile or cross-platform apps with Flutter, asks about
widget design, state management, navigation, or says "create a Flutter screen"
or "manage state" or "add navigation".

## Instructions

### 1. Widget Architecture

- Everything is a widget — compose small, focused widgets into larger screens
- Use `StatelessWidget` when the widget has no mutable state
- Use `StatefulWidget` when the widget manages local state (animations, form input)
- Keep `build()` methods lightweight — extract sub-widgets into separate classes
- Prefer composition over inheritance: wrap widgets, do not extend them
- Use `const` constructors where possible to enable widget tree optimization
- Name widgets by purpose: `UserProfileCard`, not `Card1`

### 2. Layout System

- `Row` and `Column` are the primary axis-based layout widgets
- `Expanded` and `Flexible` control how children share available space
- `Stack` for overlapping widgets; use `Positioned` for absolute placement
- `ListView` and `GridView` for scrollable lists — use `.builder()` for large lists
- `SizedBox` for fixed dimensions, `Spacer` for flexible gaps
- `Padding` and `Container` for spacing and decoration
- Use `LayoutBuilder` and `MediaQuery` for responsive designs
- `ConstrainedBox` and `IntrinsicHeight` to solve common sizing issues

### 3. State Management

- **Local state**: `StatefulWidget` + `setState()` for simple, widget-scoped state
- **Provider**: lightweight DI and state — good for most apps
- **Riverpod**: type-safe, compile-time checked, no BuildContext dependency
- **BLoC**: event-driven, separates business logic, good for complex flows
- Choose the simplest approach that fits the complexity:
  - Simple form toggle → `StatefulWidget`
  - Shared state across a few screens → Provider/Riverpod
  - Complex async flows with many events → BLoC
- Avoid putting all state in one global provider — split by domain

### 4. Navigation (GoRouter)

- Define routes declaratively: `GoRoute(path: '/users/:id', builder: ...)`
- Navigate with `context.go('/users/42')` (replace) or `context.push(...)` (stack)
- Nested navigation with `ShellRoute` for bottom nav / tab layouts
- Redirect guards for authentication: `redirect: (context, state) => ...`
- Use typed routes with `GoRouterState` for type-safe parameter access
- Deep linking works automatically — test with `adb` and `xcrun`

### 5. Theming and Styling

- Define theme in `MaterialApp(theme: ThemeData(...))` — use `ColorScheme.fromSeed()`
- Access theme values: `Theme.of(context).colorScheme.primary`
- Use Material 3: `useMaterial3: true` in `ThemeData`
- Support dark mode: provide `darkTheme` in `MaterialApp`
- Custom text styles via `TextTheme` — avoid hardcoded font sizes
- Extract reusable style constants into a `theme.dart` file

### 6. Networking and Data

- Use `http` or `dio` package for REST APIs
- Parse JSON with `json_serializable` + `build_runner` for type safety
- Use `FutureBuilder` for one-shot async data, `StreamBuilder` for real-time
- Implement a repository pattern: `UserRepository` wraps API + cache logic
- Handle loading/error/data states explicitly — never show blank screens

### 7. Platform Channels

- Use `MethodChannel` for one-off calls to native code (iOS/Android)
- Use `EventChannel` for continuous data streams from native
- Prefer existing packages (camera, geolocator, etc.) over custom channels
- Platform-specific code goes in `android/` and `ios/` directories
- Use `Platform.isIOS` / `Platform.isAndroid` for platform checks in Dart

### 8. Testing

- **Unit tests**: pure Dart logic, no Flutter dependency — `test()`
- **Widget tests**: render widgets in isolation — `testWidgets()`, `find.byType()`
- **Integration tests**: full app on device/emulator — `integration_test` package
- Use `pumpWidget()` to render, `pump()` to advance frames
- Mock dependencies with `mocktail` or `mockito`
- Golden tests for visual regression: `matchesGoldenFile('snapshot.png')`
- Aim for widget tests on all screens, unit tests on all business logic

### 9. Performance

- Use `const` widgets to skip unnecessary rebuilds
- `ListView.builder` for long lists — only builds visible items
- Avoid `setState()` high in the tree — it rebuilds all descendants
- Profile with Flutter DevTools: check rebuild counts and frame times
- Use `RepaintBoundary` to isolate expensive painting areas
- Minimize `Opacity` widget usage — it forces off-screen rendering
- Cache images with `CachedNetworkImage` package

### 10. Common Mistakes

- Nesting `Scaffold` inside `Scaffold` — leads to double app bars and broken navigation
- Using `setState` after dispose — check `mounted` or cancel timers/subscriptions
- Unbounded widgets in `Row`/`Column` — wrap with `Expanded` or `Flexible`
- Ignoring `dispose()` — always cancel controllers, streams, and animation controllers
- Hardcoding strings — use `intl` or `easy_localization` for i18n

## References

- `references/widget-catalog.md` — Common widgets by category with constructor patterns
- [Flutter Documentation](https://docs.flutter.dev/)
- [Flutter Widget Catalog](https://docs.flutter.dev/ui/widgets)

## Examples

**User:** "Create a product list screen with search and pull-to-refresh"
**Agent:** Creates a `StatefulWidget` with a search `TextField`, uses `ListView.builder`
for efficient rendering, implements `RefreshIndicator` for pull-to-refresh, fetches
products from a repository, and shows loading/error/empty states.

**User:** "Set up Riverpod state management for my app"
**Agent:** Adds `flutter_riverpod` dependency, wraps the app in `ProviderScope`, creates
providers for API client and repositories, defines `StateNotifier` or `AsyncNotifier`
for screen state, and uses `ConsumerWidget` to watch providers in the UI.

**User:** "Add bottom navigation with nested routes"
**Agent:** Defines a `ShellRoute` in GoRouter with a `Scaffold` containing
`NavigationBar`, creates child routes for each tab, preserves tab state with
`StatefulShellRoute`, and handles deep links to specific tabs.
