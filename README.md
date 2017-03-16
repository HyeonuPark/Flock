# Flock

Flock is a parallel runner for asynchronous tasks.

## What is this?

Imagine JavaScript, with lots of ES2017 async functions. If we can assume that every values shared between function calls are immutable, those functions can effectively be executed in separate thread so we can highly utilize modern multicore processors for (almost) free.

This is what exactly I want to achieve with Flock. Coroutines(tasks) are distributed across fixed amount of worker threads, and single central event loop feed them all. Tasks can be moved between workers while paused, so all workers can always stay busy while active tasks remain.

## Current state

v0.2

Every core concepts are implemented.

## Milestones

- [ ] Implement cancelable stream

- [ ] Implement main thread bound task

- [ ] Gather some user feedback
