import { describe, expect, it, vi } from "vitest";
import { LatestWriteQueue } from "./latestWrite";

describe("LatestWriteQueue", () => {
  it("coalesces rapid values and commits only the newest response", async () => {
    vi.useFakeTimers();
    const writes: number[] = [];
    const commits: number[] = [];
    const queue = new LatestWriteQueue(
      async (value: number) => { writes.push(value); return value; },
      (value) => commits.push(value),
      () => undefined,
      300,
    );
    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(3);
    await vi.advanceTimersByTimeAsync(300);
    expect(writes).toEqual([3]);
    expect(commits).toEqual([3]);
    queue.dispose();
    vi.useRealTimers();
  });

  it("rolls back only the latest failed write", async () => {
    vi.useFakeTimers();
    const failures: string[] = [];
    const queue = new LatestWriteQueue(
      async () => { throw new Error("failed"); },
      () => undefined,
      (error) => failures.push(String(error)),
      10,
    );
    queue.enqueue("latest");
    await vi.advanceTimersByTimeAsync(10);
    expect(failures).toHaveLength(1);
    vi.useRealTimers();
  });

  it("can restart after a Strict Mode cleanup cycle", async () => {
    vi.useFakeTimers();
    const writes: number[] = [];
    const queue = new LatestWriteQueue(
      async (value: number) => { writes.push(value); return value; },
      () => undefined,
      () => undefined,
      10,
    );

    queue.start();
    queue.dispose();
    queue.start();
    queue.enqueue(7);
    await vi.advanceTimersByTimeAsync(10);

    expect(writes).toEqual([7]);
    queue.dispose();
    vi.useRealTimers();
  });
});
