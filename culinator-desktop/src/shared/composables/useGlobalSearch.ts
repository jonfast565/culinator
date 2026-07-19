type SearchHandler = () => void;

const handlers = new Map<string, SearchHandler>();

export function registerSearchHandler(id: string, handler: SearchHandler): () => void {
  handlers.set(id, handler);
  return () => handlers.delete(id);
}

export function triggerSearch(id: string): boolean {
  const handler = handlers.get(id);
  if (!handler) return false;
  handler();
  return true;
}

export function isSearchShortcut(event: KeyboardEvent): boolean {
  return (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k";
}
