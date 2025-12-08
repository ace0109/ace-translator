#!/usr/bin/env node
/**
 * 版本同步脚本
 * 用法: node scripts/bump-version.mjs <version>
 * 示例: node scripts/bump-version.mjs 0.3.0
 *
 * 会同步更新以下文件的版本号:
 * - package.json
 * - src-tauri/Cargo.toml
 * - src-tauri/tauri.conf.json
 */

import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const rootDir = path.resolve(__dirname, '..')

const newVersion = process.argv[2]

if (!newVersion) {
  console.error('请提供版本号，例如: node scripts/bump-version.mjs 0.3.0')
  process.exit(1)
}

// 验证版本号格式
if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(newVersion)) {
  console.error('版本号格式无效，应为: x.y.z 或 x.y.z-suffix')
  process.exit(1)
}

console.log(`正在更新版本号到 ${newVersion}...\n`)

// 1. 更新 package.json
const packageJsonPath = path.join(rootDir, 'package.json')
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'))
const oldPackageVersion = packageJson.version
packageJson.version = newVersion
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n')
console.log(`✓ package.json: ${oldPackageVersion} -> ${newVersion}`)

// 2. 更新 src-tauri/Cargo.toml
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml')
let cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8')
const cargoVersionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)
const oldCargoVersion = cargoVersionMatch ? cargoVersionMatch[1] : 'unknown'
cargoToml = cargoToml.replace(/^(version\s*=\s*)"[^"]+"/m, `$1"${newVersion}"`)
fs.writeFileSync(cargoTomlPath, cargoToml)
console.log(`✓ src-tauri/Cargo.toml: ${oldCargoVersion} -> ${newVersion}`)

// 3. 更新 src-tauri/tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json')
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'))
const oldTauriVersion = tauriConf.version
tauriConf.version = newVersion
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n')
console.log(`✓ src-tauri/tauri.conf.json: ${oldTauriVersion} -> ${newVersion}`)

console.log(`\n版本已更新为 ${newVersion}`)
