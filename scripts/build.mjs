#!/usr/bin/env node
/**
 * Tauri 构建脚本
 * 自动设置签名环境变量后执行 tauri build
 * 构建完成后将安装包和签名文件移动到 target/{version}/ 目录
 */
import { execSync } from 'child_process'
import { resolve, dirname, basename } from 'path'
import { fileURLToPath } from 'url'
import { platform } from 'os'
import { readFileSync, mkdirSync, copyFileSync, readdirSync, existsSync, rmSync } from 'fs'

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = resolve(__dirname, '..')
const tauriDir = resolve(rootDir, 'src-tauri')
const targetDir = resolve(tauriDir, 'target')

// 读取版本号
const tauriConf = JSON.parse(readFileSync(resolve(tauriDir, 'tauri.conf.json'), 'utf-8'))
const version = tauriConf.version

// 签名密钥（用于 Tauri 更新签名）
const SIGNING_KEY = '***REMOVED_UPDATE_SIGNING_PRIVATE_KEY***'
const SIGNING_PASSWORD = '***REMOVED_UPDATE_SIGNING_PASSWORD***'

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
console.log(`📦 当前版本: v${version}`)
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

  // 构建成功后，收集安装包到版本目录
  console.log('\n📁 正在收集构建产物...')

  const outputDir = resolve(targetDir, version)
  mkdirSync(outputDir, { recursive: true })

  // 根据构建参数确定 bundle 目录位置
  // 从 args 中提取 --target 参数
  const targetIndex = args.findIndex(arg => arg === '--target' || arg === '-t')
  const buildTarget = targetIndex !== -1 && args[targetIndex + 1] ? args[targetIndex + 1] : null

  let bundleDir
  if (buildTarget) {
    bundleDir = resolve(targetDir, buildTarget, 'release', 'bundle')
  } else {
    bundleDir = resolve(targetDir, 'release', 'bundle')
  }

  console.log(`📂 Bundle 目录: ${bundleDir}`)

  let collectedFiles = []

  // 简化文件名：去掉版本号和架构信息
  const simplifyName = (filename) => {
    return filename
      .replace(/_[\d.]+/, '')           // 去掉 _1.0.1
      .replace(/_x64|_aarch64|_x86/, '') // 去掉架构
      .replace(/-setup/, '')             // 去掉 -setup
  }

  // 固定 macOS DMG 命名
  const dmgOutputName = 'ace-translator.dmg'

  if (existsSync(bundleDir)) {
    // 收集 NSIS 安装包 (Windows)
    const nsisDir = resolve(bundleDir, 'nsis')
    if (existsSync(nsisDir)) {
      const files = readdirSync(nsisDir).filter(f => f.endsWith('.exe') || f.endsWith('.sig'))
      for (const file of files) {
        const newName = simplifyName(file)
        copyFileSync(resolve(nsisDir, file), resolve(outputDir, newName))
        collectedFiles.push(newName)
      }
    }

    // 收集 DMG (macOS)
    const dmgDir = resolve(bundleDir, 'dmg')
    if (existsSync(dmgDir)) {
      const files = readdirSync(dmgDir).filter(f => f.endsWith('.dmg'))
      for (const file of files) {
        const newName = dmgOutputName
        copyFileSync(resolve(dmgDir, file), resolve(outputDir, newName))
        collectedFiles.push(newName)
      }
    }

    // 收集 macOS 更新包 (.tar.gz 和 .sig)
    const macosDir = resolve(bundleDir, 'macos')
    if (existsSync(macosDir)) {
      const files = readdirSync(macosDir).filter(f => f.endsWith('.tar.gz') || f.endsWith('.sig'))
      for (const file of files) {
        const newName = simplifyName(file)
        copyFileSync(resolve(macosDir, file), resolve(outputDir, newName))
        collectedFiles.push(newName)
      }
    } else {
      console.log('ℹ️ 未找到 macos 更新包目录（tar.gz / sig），仅收集 DMG。')
    }
  }

  if (collectedFiles.length > 0) {
    console.log(`\n✅ 构建产物已收集到: target/${version}/`)
    collectedFiles.forEach(f => console.log(`   - ${f}`))

    // 清空原产物目录
    console.log('\n🧹 正在清理原产物目录...')
    const nsisDir = resolve(bundleDir, 'nsis')
    const dmgDir = resolve(bundleDir, 'dmg')
    const macosDir = resolve(bundleDir, 'macos')

    if (existsSync(nsisDir)) {
      rmSync(nsisDir, { recursive: true, force: true })
      console.log(`   - 已清理 ${nsisDir}`)
    }
    if (existsSync(dmgDir)) {
      rmSync(dmgDir, { recursive: true, force: true })
      console.log(`   - 已清理 ${dmgDir}`)
    }
    if (existsSync(macosDir)) {
      rmSync(macosDir, { recursive: true, force: true })
      console.log(`   - 已清理 ${macosDir}`)
    }
  } else {
    console.log('\n⚠️  未找到构建产物')
  }

} catch (error) {
  process.exit(1)
}
