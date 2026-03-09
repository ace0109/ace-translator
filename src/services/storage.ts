// src/services/storage.ts
export function saveToStorage(key: string, value: any) {
  localStorage.setItem(key, JSON.stringify(value));
}

export function getFromStorage(key: string): any | null {
  const item = localStorage.getItem(key);
  return item ? JSON.parse(item) : null;
}