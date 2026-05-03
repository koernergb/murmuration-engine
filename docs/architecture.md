# Architecture Notes

The workspace is split into simulation, rendering, and application shells so behavior and platform concerns can evolve independently.

Initial contracts to keep stable:

- simulation bird layout
- simulation parameter surface
- renderer-facing bird data
- preset loading schema

