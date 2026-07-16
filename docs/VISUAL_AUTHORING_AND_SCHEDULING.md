# Visual authoring and Gantt scheduling

The Vue editor now offers a source-preserving visual authoring panel. It edits only the selected declaration range and leaves unrelated comments and formatting intact. The DSL remains canonical; visual forms are projections over the lossless document.

The `culinator-scheduler` crate implements `RecipeScheduler`. Its critical-path scheduler honors dependency types, minimum lags, operation durations, and exclusive equipment/container/labor bindings. Independent tasks are placed in parallel. The result is available through `recipes.schedule` on the authenticated WebSocket API and rendered by the Vue Gantt panel.

Future scheduler adapters can implement optimization for multiple cooks, capacities, deadlines, preferred start times, and batch production without changing the application or UI contracts.
