import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LightState } from "./ipc.js";

const { queryStateMock } = vi.hoisted(() => ({
  queryStateMock: vi.fn<() => Promise<LightState>>(),
}));

vi.mock("./ipc.js", () => ({
  queryState: queryStateMock,
}));

import { StatePoller } from "./state-poller.js";

describe("StatePoller", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    queryStateMock.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("delivers the last state immediately on subscribe", () => {
    const poller = new StatePoller();
    const listener = vi.fn();

    poller.subscribe(listener);

    expect(listener).toHaveBeenCalledTimes(1);
    expect(listener).toHaveBeenCalledWith("unknown");
  });

  it("notifies when polled state changes", async () => {
    queryStateMock.mockResolvedValueOnce("green");

    const poller = new StatePoller();
    const listener = vi.fn();
    poller.subscribe(listener);
    listener.mockClear();

    await vi.runOnlyPendingTimersAsync();

    expect(listener).toHaveBeenCalledWith("green");
  });

  it("skips listener notification when state is unchanged", async () => {
    queryStateMock.mockResolvedValue("green");

    const poller = new StatePoller();
    const listener = vi.fn();
    poller.subscribe(listener);

    await vi.runOnlyPendingTimersAsync();
    listener.mockClear();

    await vi.advanceTimersByTimeAsync(500);

    expect(listener).not.toHaveBeenCalled();
  });

  it("shares one poll interval across subscribers", async () => {
    queryStateMock.mockResolvedValue("yellow");

    const poller = new StatePoller();
    poller.subscribe(vi.fn());

    await vi.runOnlyPendingTimersAsync();
    queryStateMock.mockClear();

    poller.subscribe(vi.fn());
    await vi.advanceTimersByTimeAsync(500);

    expect(queryStateMock).toHaveBeenCalledTimes(1);
  });

  it("stops polling when the last subscriber unsubscribes", async () => {
    queryStateMock.mockResolvedValue("red");

    const poller = new StatePoller();
    const unsubscribe = poller.subscribe(vi.fn());

    await vi.runOnlyPendingTimersAsync();
    queryStateMock.mockClear();

    unsubscribe();
    await vi.advanceTimersByTimeAsync(1500);

    expect(queryStateMock).not.toHaveBeenCalled();
  });
});
