export interface ServiceBootstrap {
  endpoint: string;
  websocketUrl: string;
  token: string;
}

export type ConnectionStatus = "connecting" | "connected" | "reconnecting" | "disconnected";
export interface ServiceEvent<T = unknown> {
  event: string;
  payload: T;
}

declare global {
  interface Window {
    __CULINATOR_SERVICE__?: ServiceBootstrap;
    __TAURI_INTERNALS__?: unknown;
  }
}

interface RpcResponse<T = unknown> {
  id: string;
  ok: boolean;
  result?: T;
  error?: { code: string; message: string };
}

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
  timeout: number;
}

let bootstrapPromise: Promise<ServiceBootstrap> | undefined;
let clientPromise: Promise<WebSocketServiceClient> | undefined;
let currentStatus: ConnectionStatus = "connecting";
const statusListeners = new Set<(status: ConnectionStatus) => void>();
const eventListeners = new Set<(event: ServiceEvent) => void>();

export function hasConfiguredService(): boolean {
  return (
    Boolean(import.meta.env.VITE_CULINATOR_WS_URL || import.meta.env.VITE_CULINATOR_API_URL) ||
    "__TAURI_INTERNALS__" in window
  );
}

export function onConnectionStatus(listener: (status: ConnectionStatus) => void): () => void {
  statusListeners.add(listener);
  listener(currentStatus);
  return () => statusListeners.delete(listener);
}

export function onServiceEvent(listener: (event: ServiceEvent) => void): () => void {
  eventListeners.add(listener);
  return () => eventListeners.delete(listener);
}

function emitStatus(status: ConnectionStatus): void {
  currentStatus = status;
  for (const listener of statusListeners) listener(status);
}

async function serviceBootstrap(): Promise<ServiceBootstrap> {
  const configuredWs = import.meta.env.VITE_CULINATOR_WS_URL as string | undefined;
  const configuredHttp = import.meta.env.VITE_CULINATOR_API_URL as string | undefined;
  const configuredToken = import.meta.env.VITE_CULINATOR_API_TOKEN as string | undefined;
  if (configuredWs || configuredHttp) {
    if (!configuredToken) throw new Error("VITE_CULINATOR_API_TOKEN is required");
    const endpoint = (
      configuredHttp ?? configuredWs!.replace(/^ws/, "http").replace(/\/ws$/, "")
    ).replace(/\/$/, "");
    return {
      endpoint,
      websocketUrl: configuredWs ?? `${endpoint.replace(/^http/, "ws")}/ws`,
      token: configuredToken,
    };
  }

  if (window.__CULINATOR_SERVICE__) return window.__CULINATOR_SERVICE__;
  bootstrapPromise ??= new Promise<ServiceBootstrap>((resolve, reject) => {
    const timeout = window.setTimeout(
      () => reject(new Error("Tauri did not provide the local service bootstrap")),
      10_000,
    );
    window.addEventListener(
      "culinator:service-ready",
      (event) => {
        window.clearTimeout(timeout);
        resolve((event as CustomEvent<ServiceBootstrap>).detail);
      },
      { once: true },
    );
  });
  return bootstrapPromise;
}

class WebSocketServiceClient {
  private socket?: WebSocket;
  private readonly pending = new Map<string, PendingRequest>();
  private reconnectAttempt = 0;
  private closed = false;
  private connectPromise?: Promise<void>;

  constructor(private readonly bootstrap: ServiceBootstrap) {}

  connect(): Promise<void> {
    if (this.socket?.readyState === WebSocket.OPEN) return Promise.resolve();
    if (this.connectPromise) return this.connectPromise;
    this.connectPromise = this.openSocket().finally(() => {
      this.connectPromise = undefined;
    });
    return this.connectPromise;
  }

  async request<T>(method: string, params: Record<string, unknown> = {}): Promise<T> {
    await this.connect();
    const socket = this.socket;
    if (!socket || socket.readyState !== WebSocket.OPEN)
      throw new Error("Service socket is unavailable");
    const id = crypto.randomUUID();
    return new Promise<T>((resolve, reject) => {
      const timeout = window.setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Service request timed out: ${method}`));
      }, 15_000);
      this.pending.set(id, {
        resolve: (value) => resolve(value as T),
        reject,
        timeout,
      });
      socket.send(JSON.stringify({ id, method, params }));
    });
  }

  close(): void {
    this.closed = true;
    this.socket?.close(1000, "Application closing");
    this.rejectPending(new Error("Service connection closed"));
  }

  private openSocket(): Promise<void> {
    emitStatus(this.reconnectAttempt > 0 ? "reconnecting" : "connecting");
    return new Promise((resolve, reject) => {
      const socket = new WebSocket(this.bootstrap.websocketUrl, [
        "culinator.v1",
        `culinator.auth.${this.bootstrap.token}`,
      ]);
      this.socket = socket;
      const timeout = window.setTimeout(() => {
        socket.close();
        reject(new Error("Timed out connecting to Culinator service"));
      }, 10_000);

      socket.addEventListener(
        "open",
        () => {
          window.clearTimeout(timeout);
          this.reconnectAttempt = 0;
          emitStatus("connected");
          resolve();
        },
        { once: true },
      );
      socket.addEventListener("message", (event) => this.handleMessage(event.data));
      socket.addEventListener(
        "error",
        () => {
          window.clearTimeout(timeout);
          reject(new Error("Could not connect to Culinator service"));
        },
        { once: true },
      );
      socket.addEventListener("close", () => {
        this.socket = undefined;
        emitStatus("disconnected");
        this.rejectPending(new Error("Service connection was interrupted"));
        if (!this.closed) this.scheduleReconnect();
      });
    });
  }

  private handleMessage(data: unknown): void {
    if (typeof data !== "string") return;
    const message = JSON.parse(data) as RpcResponse | ServiceEvent;
    if ("event" in message) {
      for (const listener of eventListeners) listener(message);
      return;
    }
    const pending = this.pending.get(message.id);
    if (!pending) return;
    window.clearTimeout(pending.timeout);
    this.pending.delete(message.id);
    if (message.ok) pending.resolve(message.result);
    else pending.reject(new Error(message.error?.message ?? "Service operation failed"));
  }

  private scheduleReconnect(): void {
    this.reconnectAttempt += 1;
    const delay = Math.min(250 * 2 ** (this.reconnectAttempt - 1), 5_000);
    emitStatus("reconnecting");
    window.setTimeout(() => void this.connect().catch(() => undefined), delay);
  }

  private rejectPending(error: Error): void {
    for (const pending of this.pending.values()) {
      window.clearTimeout(pending.timeout);
      pending.reject(error);
    }
    this.pending.clear();
  }
}

async function serviceClient(): Promise<WebSocketServiceClient> {
  clientPromise ??= serviceBootstrap().then(async (bootstrap) => {
    const client = new WebSocketServiceClient(bootstrap);
    await client.connect();
    return client;
  });
  return clientPromise;
}

export async function serviceRpc<T>(method: string, params?: Record<string, unknown>): Promise<T> {
  return (await serviceClient()).request<T>(method, params ?? {});
}

export async function serviceRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const method = (init?.method ?? "GET").toUpperCase();
  const body = init?.body ? (JSON.parse(String(init.body)) as Record<string, unknown>) : {};
  const recipe = path.match(/^\/api\/v1\/recipes\/([^/]+)$/);
  const recipeBookMove = path.match(/^\/api\/v1\/recipes\/([^/]+)\/book$/);
  const recipeTries = path.match(/^\/api\/v1\/recipes\/([^/]+)\/tries$/);
  const recipeHaccp = path.match(/^\/api\/v1\/recipes\/([^/]+)\/haccp$/);
  const book = path.match(/^\/api\/v1\/books\/([^/]+)$/);
  const recipeExport = path.match(/^\/api\/v1\/recipes\/([^/]+)\/export$/);
  const recipeFormulas = path.match(/^\/api\/v1\/recipes\/([^/]+)\/formulas$/);
  const recipeTry = path.match(/^\/api\/v1\/tries\/([^/]+)$/);
  const haccpPlan = path.match(/^\/api\/v1\/haccp\/([^/]+)$/);
  const haccpRecord = path.match(/^\/api\/v1\/haccp\/ccps\/([^/]+)\/records$/);
  if (path === "/api/v1/recipes" && method === "GET") return serviceRpc("recipes.list");
  if (path === "/api/v1/recipes" && method === "POST") return serviceRpc("recipes.create", body);
  if (path === "/api/v1/books" && method === "GET") return serviceRpc("books.list");
  if (path === "/api/v1/books" && method === "POST") return serviceRpc("books.create", body);
  if (book && method === "PUT")
    return serviceRpc("books.update", { id: decodeURIComponent(book[1]), ...body });
  if (book && method === "DELETE")
    return serviceRpc("books.delete", { id: decodeURIComponent(book[1]) });
  if (recipeBookMove && method === "PUT")
    return serviceRpc("recipes.move", { id: decodeURIComponent(recipeBookMove[1]), ...body });
  if (recipeExport && method === "POST")
    return serviceRpc("recipes.export", { id: decodeURIComponent(recipeExport[1]), ...body });
  if (recipe && method === "GET")
    return serviceRpc("recipes.get", { id: decodeURIComponent(recipe[1]) });
  if (recipe && method === "PUT")
    return serviceRpc("recipes.save", { id: decodeURIComponent(recipe[1]), ...body });
  if (recipe && method === "DELETE")
    return serviceRpc("recipes.delete", { id: decodeURIComponent(recipe[1]) });
  if (path === "/api/v1/validation") return serviceRpc("recipes.validate", body);
  if (path === "/api/v1/formulas/calculate") return serviceRpc("formulas.calculate", body);
  if (path === "/api/v1/formulas/percentages") return serviceRpc("formulas.percentages", body);
  if (path === "/api/v1/formulas" && method === "PUT")
    return serviceRpc("formulas.save", { formula: body });
  if (recipeFormulas)
    return serviceRpc("formulas.list", { recipeId: decodeURIComponent(recipeFormulas[1]) });
  if (recipeTries && method === "GET")
    return serviceRpc("tries.list", { recipeId: decodeURIComponent(recipeTries[1]) });
  if (recipeTry && method === "GET")
    return serviceRpc("tries.get", { tryId: decodeURIComponent(recipeTry[1]) });
  if (recipeTry && method === "DELETE")
    return serviceRpc("tries.delete", { tryId: decodeURIComponent(recipeTry[1]) });
  if (recipeHaccp && method === "GET")
    return serviceRpc("haccp.list", { recipeId: decodeURIComponent(recipeHaccp[1]) });
  if (recipeHaccp && method === "POST")
    return serviceRpc("haccp.create", { recipeId: decodeURIComponent(recipeHaccp[1]), ...body });
  if (haccpPlan && method === "GET")
    return serviceRpc("haccp.get", { planId: decodeURIComponent(haccpPlan[1]) });
  if (haccpPlan && method === "PUT")
    return serviceRpc("haccp.save", { planId: decodeURIComponent(haccpPlan[1]), ...body });
  if (haccpPlan && method === "DELETE")
    return serviceRpc("haccp.delete", { planId: decodeURIComponent(haccpPlan[1]) });
  if (haccpRecord && method === "POST")
    return serviceRpc("haccp.record", { ccpId: decodeURIComponent(haccpRecord[1]), ...body });
  throw new Error(`No WebSocket RPC mapping for ${method} ${path}`);
}
