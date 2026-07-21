export class LatestWriteQueue<T, R> {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private pending: { value: T; token: number } | null = null;
  private inFlight = false;
  private token = 0;
  private disposed = false;

  constructor(
    private readonly write: (value: T) => Promise<R>,
    private readonly commit: (result: R) => void,
    private readonly rollback: (error: unknown) => void,
    private readonly delay = 300,
  ) {}

  start() {
    this.disposed = false;
  }

  enqueue(value: T) {
    if (this.disposed) return;
    this.pending = { value, token: ++this.token };
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.flush(), this.delay);
  }

  async flush() {
    if (this.disposed || this.inFlight || !this.pending) return;
    const next = this.pending;
    this.pending = null;
    this.inFlight = true;
    try {
      const result = await this.write(next.value);
      if (!this.disposed && next.token === this.token) this.commit(result);
    } catch (error) {
      if (!this.disposed && next.token === this.token) this.rollback(error);
    } finally {
      this.inFlight = false;
      if (this.pending && !this.disposed) {
        if (this.timer) clearTimeout(this.timer);
        this.timer = setTimeout(() => void this.flush(), 0);
      }
    }
  }

  dispose() {
    this.disposed = true;
    this.pending = null;
    if (this.timer) clearTimeout(this.timer);
  }
}
