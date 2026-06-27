import path from "path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { createConnectionMock } = vi.hoisted(() => ({
  createConnectionMock: vi.fn(),
}));

vi.mock("net", () => ({
  default: {
    createConnection: createConnectionMock,
  },
}));

import { queryState, resolveSocketPath } from "./ipc";

type MockSocket = {
  setTimeout: ReturnType<typeof vi.fn>;
  write: ReturnType<typeof vi.fn>;
  destroy: ReturnType<typeof vi.fn>;
  once: (event: string, handler: (...args: unknown[]) => void) => void;
  on: (event: string, handler: (...args: unknown[]) => void) => void;
  emitOnce: (event: string, ...args: unknown[]) => void;
  emit: (event: string, ...args: unknown[]) => void;
};

function createMockSocket(): MockSocket {
  const onceHandlers = new Map<string, (...args: unknown[]) => void>();
  const handlers = new Map<string, Array<(...args: unknown[]) => void>>();

  return {
    setTimeout: vi.fn(),
    write: vi.fn(),
    destroy: vi.fn(),
    once(event, handler) {
      onceHandlers.set(event, handler);
    },
    on(event, handler) {
      const list = handlers.get(event) ?? [];
      list.push(handler);
      handlers.set(event, list);
    },
    emitOnce(event, ...args) {
      const handler = onceHandlers.get(event);
      if (handler) {
        onceHandlers.delete(event);
        handler(...args);
      }
    },
    emit(event, ...args) {
      this.emitOnce(event, ...args);
      for (const handler of handlers.get(event) ?? []) {
        handler(...args);
      }
    },
  };
}

function mockSuccessfulResponse(response: string): MockSocket {
  const socket = createMockSocket();
  createConnectionMock.mockReturnValue(socket);

  const promise = queryState();
  socket.emitOnce("connect");
  socket.emit("data", Buffer.from(response));
  socket.emitOnce("end");

  return { socket, promise };
}

describe("resolveSocketPath", () => {
  const previous = process.env["SEMAPHORE_SOCKET"];

  afterEach(() => {
    if (previous === undefined) {
      delete process.env["SEMAPHORE_SOCKET"];
    } else {
      process.env["SEMAPHORE_SOCKET"] = previous;
    }
    vi.unstubAllEnvs();
  });

  it("prefers SEMAPHORE_SOCKET when set", () => {
    vi.stubEnv("SEMAPHORE_SOCKET", "/custom/semaphore.sock");
    expect(resolveSocketPath()).toBe("/custom/semaphore.sock");
  });

  it("uses the Windows named pipe on win32", () => {
    vi.stubEnv("SEMAPHORE_SOCKET", "");
    vi.stubGlobal("process", { ...process, platform: "win32" });
    expect(resolveSocketPath()).toBe("\\\\.\\pipe\\semaphore");
  });

  it("uses XDG_RUNTIME_DIR on Unix when set", () => {
    vi.stubEnv("SEMAPHORE_SOCKET", "");
    vi.stubEnv("XDG_RUNTIME_DIR", "/run/user/1000");
    vi.stubGlobal("process", {
      ...process,
      platform: "linux",
      getuid: () => 1000,
    });
    expect(resolveSocketPath()).toBe(path.join("/run/user/1000", "semaphore.sock"));
  });

  it("falls back to /tmp/semaphore-{uid}.sock on Unix", () => {
    vi.stubEnv("SEMAPHORE_SOCKET", "");
    vi.stubEnv("XDG_RUNTIME_DIR", "");
    vi.stubGlobal("process", {
      ...process,
      platform: "darwin",
      getuid: () => 501,
    });
    expect(resolveSocketPath()).toBe("/tmp/semaphore-501.sock");
  });
});

describe("queryState", () => {
  beforeEach(() => {
    createConnectionMock.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it.each([
    ["green", '{"state":"green"}\n'],
    ["yellow", '{"state":"yellow"}\n'],
    ["red", '{"state":"red"}\n'],
  ] as const)("returns %s for a valid status response", async (state, response) => {
    const { socket, promise } = mockSuccessfulResponse(response);

    await expect(promise).resolves.toBe(state);
    expect(createConnectionMock).toHaveBeenCalledOnce();
    expect(socket.write).toHaveBeenCalledWith('{"cmd":"status"}\n');
    expect(socket.setTimeout).toHaveBeenCalledWith(1000);
  });

  it("returns unknown for unrecognized state values", async () => {
    const { promise } = mockSuccessfulResponse('{"state":"blue"}\n');
    await expect(promise).resolves.toBe("unknown");
  });

  it("returns unknown for malformed JSON", async () => {
    const { promise } = mockSuccessfulResponse("not-json\n");
    await expect(promise).resolves.toBe("unknown");
  });

  it("returns unknown when the socket cannot connect", async () => {
    const socket = createMockSocket();
    createConnectionMock.mockReturnValue(socket);

    const promise = queryState();
    socket.emitOnce("error");

    await expect(promise).resolves.toBe("unknown");
  });

  it("returns unknown on timeout", async () => {
    vi.useFakeTimers();

    const socket = createMockSocket();
    createConnectionMock.mockReturnValue(socket);

    const promise = queryState();
    socket.emitOnce("connect");
    socket.emitOnce("timeout");

    await expect(promise).resolves.toBe("unknown");
    expect(socket.destroy).toHaveBeenCalled();
  });
});
