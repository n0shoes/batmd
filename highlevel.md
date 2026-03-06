
We are going to build and app that is like batcat crossed with vim.
The idea is it's a command line app that renders markdown docs.
The user can press "e" to enter edit mode (similar to vim i for insert)
When in insert mode the content is raw markdown.
The user presses esc to exit edit mode and return to the rendered view.
It shoudl also support basic emacs bindings like ctrl-k to cut the current line.
ctrl-a and ctrl-e (not sure if this would all work by default or if it has to be coded).

We are going to use a Rust and build it as a TUI using the ratatui, crossterm and syntect crates for batcat-like rendering.

the user launches it with the batmd (or something command) or maybe mdcat as that's easier for my to type? eg.

mdcat filename
I should be able to move around in the text with up/down/left/right keys.
When I press 'i', let me type to change the text.
When I press 'Esc', go back to View."

Changes should be automatically saved.
If editing and the file is updated in the background eg. By claude code then we should handle that with a dirty flag aand a reload option. If there are any gotchas we need to discuss ensure we do so.


Probably Use a file watcher (like the notify crate in Rust).
If a change is detected while you're in Edit Mode, show a small red dot or an "!" in the corner.
Don't force a reload—let the user hit R to pull in Claude’s changes when they're ready.

In a TUI, if the text "reflows" (wraps differently) when you switch from View (hidden symbols) to Edit (visible symbols), your cursor might visually jump.
Solution: Keep the line-wrapping logic identical in both modes, simply replacing hidden characters with "empty space" or using a consistent padding so the text doesn't shift horizontally.




when editing I should be editing raw markdown syntax
when not editing i should be viewing rendered markdown - this should be very much like the batcat viewing experience.

external links and images should just be rendered as text but highlighted rather than
rendered. (ask the user if this is confusing)

We should try to use nice colors that pop - perhaps these are defined in the libraries we will use or a batcat thheme we can discuss but it must look slick and polished.


The "Bat" DNA: Since bat is written in Rust, you are using the same native tools. If you want to see how bat handles a specific theme, you can literally read its source code and copy the logic.



To keep the UX as lean as possible, here is a blueprint for the logic:
State 1: View (The "Bat" Side)
Visuals: Use Glow-style rendering. Headers are distinct colors, lists use bullets, and Markdown symbols (#, *) are hidden.
Controls: arrows or maybe j/k to scroll, / to search, i to enter Edit mode.

State 2: Edit (The "Batcat" Side)
Visuals: Use Syntect (the bat engine). All Markdown symbols reappear so you can precisely place your syntax.
Controls: Standard typing. Esc returns to View mode.

The Autosave Engine:
Every time the user pauses typing for ~500ms or hits Esc, the buffer writes to disk. This prevents the "Did I save?" anxiety while keeping performance high.

The Crates (Libraries):
Ratatui: For the UI.
Crossterm: To handle the terminal "raw mode" (so i works instantly without hitting Enter).
Syntect: For the bat syntax highlighting.

if we need to we can probably download the source for batcat. please let me know.