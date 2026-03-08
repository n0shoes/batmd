---
# batmde-4hjy
title: Remove unused syntect dependency
status: completed
type: task
priority: normal
created_at: 2026-03-08T04:25:39Z
updated_at: 2026-03-08T04:25:58Z
parent: batmde-2wte
---

syntect is in Cargo.toml but no longer used anywhere. Remove it to slim down deps and speed up builds.