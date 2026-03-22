---
name: mobile-app-design
description: Mobile app UX design including touch targets, navigation patterns, platform conventions, and accessibility. Use when designing mobile interfaces, reviewing mobile UX, or adapting web designs for mobile.
---

# Mobile App Design

## When to Use

When the user is designing mobile interfaces, asks about navigation patterns,
touch interactions, platform conventions, or says "design this for mobile"
or "review my mobile UX" or "adapt this for iOS and Android".

## Instructions

### 1. Touch Target Sizing

- Minimum 44x44 points (iOS) / 48x48 dp (Android) for all interactive elements
- Spacing between targets: at least 8pt gap to prevent mis-taps
- Primary actions (CTA buttons) should be larger: 48-56pt height
- Thumbs reach the lower two-thirds of the screen easily — place frequent actions there
- Avoid placing destructive actions near frequent-tap zones
- Floating action buttons: 56dp standard, 40dp mini

### 2. Navigation Patterns

- **Tab bar** (bottom): 3-5 top-level destinations, always visible, best for peer sections
- **Navigation drawer** (hamburger): 5+ destinations or infrequent sections
- **Stack navigation**: push/pop for hierarchical content (detail screens)
- **Modal sheets**: temporary tasks, filters, confirmations — dismiss with swipe or button
- Keep navigation depth shallow (3 levels max) — deep nesting confuses users
- Show a back button or swipe-to-go-back gesture on all non-root screens
- Persistent bottom nav: highlight the active tab, do not reset tab state on re-tap

### 3. Responsive Layouts

- Design for the smallest supported screen first (375pt width for iOS, 360dp for Android)
- Use a 4pt/8pt spacing grid for consistent alignment
- Support landscape orientation unless the content is strongly portrait-oriented
- Adapt to tablets: use split-view or increase content width with max-width constraints
- Support Dynamic Type (iOS) and font scaling (Android) — never use fixed font sizes
- Test with system font size set to maximum to catch overflow issues

### 4. iOS vs Android Conventions

- **Back navigation**: iOS swipe-from-edge or top-left button; Android system back button/gesture
- **Tabs**: iOS bottom tab bar; Android bottom navigation bar or top tabs
- **Buttons**: iOS plain text or rounded rect; Android filled/outlined Material buttons
- **Alerts**: iOS centered alerts with stacked buttons; Android dialogs with side-by-side buttons
- **Typography**: iOS uses SF Pro; Android uses Roboto (or system font)
- **Icons**: iOS SF Symbols (outline style); Android Material Icons (filled/outlined)
- When in doubt, follow platform conventions — users expect familiar behavior

### 5. Gesture Patterns

- **Tap**: primary interaction — keep feedback instant (<100ms)
- **Long press**: secondary actions, context menus — always provide a visible alternative
- **Swipe**: list actions (delete, archive), sheet dismissal, page navigation
- **Pull to refresh**: standard for list content — show a spinner indicator
- **Pinch to zoom**: images, maps — provide zoom controls as alternative
- Make swipe actions discoverable with hints (peek animation on first use)
- Never make gestures the only way to perform an action

### 6. Accessibility

- Support screen readers: add labels to all interactive elements and images
- Ensure 4.5:1 contrast ratio for text, 3:1 for UI components
- Support Dynamic Type / font scaling without layout breakage
- Provide captions or transcripts for audio/video content
- Test with VoiceOver (iOS) and TalkBack (Android)
- Touch targets must remain accessible at all accessibility zoom levels
- Respect reduced motion preferences — provide static alternatives to animations

### 7. Offline-First Design

- Show cached content immediately while fetching updates in background
- Display clear "offline" indicators — do not silently fail
- Queue user actions when offline, sync when connected
- Use optimistic UI: show the result immediately, reconcile later
- Store critical data locally — users expect apps to work without signal
- Design conflict resolution for collaborative data

### 8. Push Notifications

- Request permission at a contextually relevant moment, not on first launch
- Explain the value before asking: "Get notified when your order ships"
- Group related notifications to avoid overwhelming the user
- Deep link from notifications directly to relevant content
- Respect notification settings — provide in-app controls to customize
- Never use notifications for marketing on first install — earn trust first

### 9. Onboarding Flows

- Keep onboarding to 3-5 screens maximum — users want to use the app
- Show value immediately — let users explore before requiring sign-up
- Use progressive disclosure: teach features when the user first encounters them
- Skip onboarding for returning users — check for existing data
- Provide a "Skip" option on every onboarding screen
- Use contextual tooltips and coach marks instead of upfront tutorials

## References

- `references/platform-guidelines.md` — iOS HIG vs Material Design comparison
- [Apple Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/)
- [Material Design 3](https://m3.material.io/)

## Examples

**User:** "Design the navigation for a banking app"
**Agent:** Recommends a bottom tab bar with 4 tabs (Accounts, Transfers, Cards, More),
stack navigation for account details and transaction history, a modal bottom sheet
for quick transfer, and biometric authentication before sensitive actions. Places
the primary CTA ("Send Money") in the thumb-reachable zone.

**User:** "Review the UX of this mobile screen"
**Agent:** Checks touch target sizes (flags any below 44pt), evaluates navigation
consistency, verifies accessibility labels and contrast ratios, tests the layout
at 320pt width and with maximum font scaling, and suggests specific improvements
for each issue found.

**User:** "Adapt this web dashboard for mobile"
**Agent:** Replaces the sidebar with a bottom tab bar, converts the data table to
a card-based list, moves filters to a bottom sheet, makes charts scrollable with
one visible at a time, and increases all touch targets to meet platform minimums.
