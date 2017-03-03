# Flock

Flock is a parallel runner for asynchronous tasks.

## What is this?

Imagine JavaScript, with lots of ES2017 async functions. If we can assume that every values shared between function calls are immutable, those functions can effectively be executed in separate thread so we can highly utilize modern multicore processors for (almost) free.

This is what exactly I want to achieve with Flock. Coroutines(tasks) are distributed across fixed amount of worker threads, and single central event loop feed them all. Tasks can be moved between workers while paused, so all workers can always stay busy while active tasks remain.

## Current state

v0.1

Work-stealing is not implemented, so all workers except first one keep idle.

As it's just identical scenario with single-threaded situation, implementation of work-stealing should not affect program semantics.

## Possible todos

- Cancelable stream
- Prioritize actors before execute to maximize cache hit
- Specialize worker-local task to not emit events to kernel
