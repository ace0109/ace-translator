// src/services/zhipu.ts
export function translateWithZhipu(text: string): Promise<string> {
  console.log('Translating with Zhipu:', text);
  return Promise.resolve(`Translated: ${text}`);
}