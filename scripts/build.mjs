#!/usr/bin/env node
/**
 * Tauri 构建脚本
 * 自动设置签名环境变量后执行 tauri build
 */
import { execSync } from 'child_process'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'
import { platform } from 'os'

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = resolve(__dirname, '..')

// 签名密钥（用于 Tauri 更新签名）
const SIGNING_KEY = 'dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5YkRpUTE2SE5EMXdLZ01ZbmFKdkpuQnBIYlQ1MGhETmRHRzRhVkhFNGJPUUFBQkFBQUFBQUFBQUFBQUlBQUFBQURhaklQelhMeTVBdGVTRzFYOWVyeE5hRkJrdGMySmJWTXl0OUl2RFB1Tzhjdk5rVUtsOTVPRmlDSjdldjd3L3pFTEYyRGNJblZjbFRvWmErcFBjMUhPYVpIRElrenFEaU4xOUIremRIRFFVQTFpUU56eUdKbDA4eEFSMEw4ak5TdklmRHNQYnEwd289Cg=='
const SIGNING_PASSWORD = 'Caiyuan0109...'

// 获取额外参数
let args = process.argv.slice(2)

// 如果在 macOS 上构建 Windows 目标，自动添加 --runner cargo-xwin
const isWindows = platform() === 'win32'
const isBuildingWindows = args.some(arg => arg.includes('windows') || arg.includes('pc-windows'))
if (!isWindows && isBuildingWindows) {
  args.push('--runner', 'cargo-xwin')
  console.log('🍎 检测到 macOS 环境构建 Windows，已自动添加 --runner cargo-xwin')
}

console.log('🔐 已设置 Tauri 签名环境变量')
console.log('🔨 开始构建...\n')

try {
  execSync(`pnpm tauri build ${args.join(' ')}`, {
    cwd: rootDir,
    stdio: 'inherit',
    env: {
      ...process.env,
      TAURI_SIGNING_PRIVATE_KEY: SIGNING_KEY,
      TAURI_SIGNING_PRIVATE_KEY_PASSWORD: SIGNING_PASSWORD,
    },
  })
} catch (error) {
  process.exit(1)
}
