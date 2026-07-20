# Agent and Contributor Guide

Begin every task by reading [manifest.md](manifest.md), then follow the linked product requirements, notices, and lessons relevant to the work.

## Required working practices

- Treat `PRD.md` as the product outcome and scope authority.
- Update `notices.md` when compatibility, licensing, installation, security, or user-facing risk changes.
- Append to `lessons.md` whenever work reveals a durable constraint, failed assumption, or reusable solution.
- Keep the Rust core independent of GPUI until the backend Ableton smoke test is complete.
- Keep version-sensitive Live Browser behavior isolated behind a bridge adapter.
- Bind local services to loopback only and authenticate every state-changing request.
- Preserve headless CLI coverage for integration behavior.
- Do not edit vendor SDK contents or reference repositories unless a task explicitly requires it.
- Do not treat code under `refs/` as project-owned code; record copied or adapted material and licensing implications.

## Validation expectations

- Run focused tests while iterating and full workspace validation before declaring a milestone complete.
- Test protocol changes on both sides of the Rust/Python boundary.
- Document manual Ableton verification steps for behavior that cannot be automated safely.
- Never claim compatibility with a Live version that has not been exercised.
